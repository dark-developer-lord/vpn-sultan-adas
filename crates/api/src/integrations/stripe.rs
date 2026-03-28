use stripe::{Client, CreatePaymentIntent, CreateCustomer, ListPaymentMethods, PaymentIntent, Customer};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ==================== TYPES ====================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StripeConfig {
    pub api_key: String,
    pub webhook_secret: String,
    pub price_ids: PriceIds,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PriceIds {
    pub starter: String,    // $5/month or price_id
    pub professional: String, // $15/month
    pub enterprise: String, // $49/month or contact sales
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionTierResponse {
    pub tier: String,
    pub price: i64,        // in cents
    pub currency: String,
    pub billing_period: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePaymentResponse {
    pub payment_intent_id: String,
    pub client_secret: String,
    pub amount: i64,
    pub currency: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub id: String,
    pub object: String,
    pub data: WebhookData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebhookData {
    pub object: String,
}

// ==================== SUBSCRIPTION PLANS ====================

pub struct SubscriptionPlans;

impl SubscriptionPlans {
    pub const STARTER_MONTHLY: &'static str = "starter_monthly";
    pub const PROFESSIONAL_MONTHLY: &'static str = "professional_monthly";
    pub const ENTERPRISE_MONTHLY: &'static str = "enterprise_monthly";
}

// ==================== STRIPE SERVICE ====================

pub struct StripeService {
    client: Client,
    config: StripeConfig,
}

impl StripeService {
    pub fn new(config: StripeConfig) -> Self {
        let client = Client::new(config.api_key.clone());
        Self { client, config }
    }

    /// Get all available subscription tiers
    pub fn get_subscription_tiers() -> Vec<SubscriptionTierResponse> {
        vec![
            SubscriptionTierResponse {
                tier: "starter".to_string(),
                price: 500,       // $5
                currency: "usd".to_string(),
                billing_period: "monthly".to_string(),
            },
            SubscriptionTierResponse {
                tier: "professional".to_string(),
                price: 1500,      // $15
                currency: "usd".to_string(),
                billing_period: "monthly".to_string(),
            },
            SubscriptionTierResponse {
                tier: "enterprise".to_string(),
                price: 4900,      // $49
                currency: "usd".to_string(),
                billing_period: "monthly".to_string(),
            },
        ]
    }

    /// Create or get Stripe customer
    pub async fn get_or_create_customer(
        &self,
        user_id: &str,
        email: &str,
        name: &str,
        db: &Database,
    ) -> Result<String> {
        // Check if customer already exists
        let existing = db
            .query_one(
                "SELECT stripe_customer_id FROM users WHERE id = $1",
                &[user_id],
            )
            .await?;

        if let Ok(Some(row)) = existing {
            let stripe_id: String = row.get(0);
            if !stripe_id.is_empty() {
                return Ok(stripe_id);
            }
        }

        // Create new customer
        let customer_params = CreateCustomer::new()
            .email(email)
            .name(name);

        let customer = stripe::Customer::create(&self.client, customer_params)
            .await?;

        let stripe_customer_id = customer.id.to_string();

        // Save to database
        db.execute(
            "UPDATE users SET stripe_customer_id = $1 WHERE id = $2",
            &[&stripe_customer_id, user_id],
        )
        .await?;

        Ok(stripe_customer_id)
    }

    /// Create payment intent for one-time charge
    pub async fn create_payment_intent(
        &self,
        amount: i64, // in cents
        currency: &str,
        user_id: &str,
        description: &str,
        db: &Database,
    ) -> Result<CreatePaymentResponse> {
        let payload = CreatePaymentIntent::new(amount, stripe::CurrencyCode::Usd)
            .description(description)
            .metadata("user_id", user_id);

        let intent = stripe::PaymentIntent::create(&self.client, payload).await?;

        let payment_intent_id = intent.id.to_string();
        let client_secret = intent.client_secret.unwrap_or_default().to_string();

        // Store in database for tracking
        db.execute(
            "INSERT INTO stripe_payments (payment_intent_id, user_id, amount, currency, status, created_at)
             VALUES ($1, $2, $3, $4, 'pending', NOW())",
            &[
                &payment_intent_id,
                user_id,
                &amount.to_string(),
                &currency,
            ],
        )
        .await?;

        Ok(CreatePaymentResponse {
            payment_intent_id,
            client_secret,
            amount,
            currency: currency.to_string(),
        })
    }

    /// Subscribe user to a plan
    pub async fn create_subscription(
        &self,
        user_id: &str,
        email: &str,
        name: &str,
        tier: &str,
        db: &Database,
    ) -> Result<String> {
        // Get or create Stripe customer
        let customer_id = self.get_or_create_customer(user_id, email, name, db).await?;

        // Get price ID for tier
        let price_id = match tier {
            "starter" => self.config.price_ids.starter.clone(),
            "professional" => self.config.price_ids.professional.clone(),
            "enterprise" => self.config.price_ids.enterprise.clone(),
            _ => return Err(AppError::BadRequest("Invalid tier".to_string())),
        };

        // Create subscription
        let subscription = stripe::Subscription::create(
            &self.client,
            stripe::CreateSubscription::new(customer_id.parse()?)
                .add_item(stripe::CreateSubscriptionItems::new().set_price(price_id.parse()?)),
        )
        .await?;

        let subscription_id = subscription.id.to_string();

        // Update database
        db.execute(
            "UPDATE users SET 
             subscription_tier = $1, 
             subscription_id = $2, 
             stripe_customer_id = $3,
             subscription_started_at = NOW()
             WHERE id = $4",
            &[&tier, &subscription_id, &customer_id.parse::<String>()?, user_id],
        )
        .await?;

        log_audit_event(
            user_id,
            "SUBSCRIPTION_CREATED",
            &format!("Subscribed to {} tier", tier),
            db,
        )
        .await?;

        Ok(subscription_id)
    }

    /// Cancel subscription
    pub async fn cancel_subscription(
        &self,
        user_id: &str,
        db: &Database,
    ) -> Result<()> {
        let subscription = db
            .query_one(
                "SELECT subscription_id FROM users WHERE id = $1",
                &[user_id],
            )
            .await?
            .ok_or(AppError::NotFound("No subscription found".to_string()))?;

        let subscription_id: String = subscription.get(0);

        // Cancel in Stripe
        let cancel_params = stripe::CancelSubscription::new();
        stripe::Subscription::cancel(&self.client, subscription_id.parse()?, cancel_params)
            .await?;

        // Update database
        db.execute(
            "UPDATE users SET 
             subscription_tier = 'free', 
             subscription_id = NULL,
             subscription_cancelled_at = NOW()
             WHERE id = $1",
            &[user_id],
        )
        .await?;

        log_audit_event(
            user_id,
            "SUBSCRIPTION_CANCELLED",
            "User cancelled subscription",
            db,
        )
        .await?;

        Ok(())
    }

    /// Handle Stripe webhook event
    pub async fn handle_webhook(
        &self,
        payload: &str,
        signature: &str,
        db: &Database,
    ) -> Result<()> {
        // Verify webhook signature
        self.verify_webhook_signature(payload, signature)?;

        // Parse webhook payload
        let event: serde_json::Value = serde_json::from_str(payload)?;
        let event_type = event["type"].as_str().unwrap_or("");

        match event_type {
            "payment_intent.succeeded" => {
                let payment_id = event["data"]["object"]["id"].as_str().unwrap_or("");
                self.handle_payment_succeeded(payment_id, db).await?;
            }
            "payment_intent.payment_failed" => {
                let payment_id = event["data"]["object"]["id"].as_str().unwrap_or("");
                self.handle_payment_failed(payment_id, db).await?;
            }
            "customer.subscription.updated" => {
                let subscription_id = event["data"]["object"]["id"].as_str().unwrap_or("");
                self.handle_subscription_updated(subscription_id, db).await?;
            }
            "invoice.payment_failed" => {
                self.handle_invoice_payment_failed(&event["data"]["object"], db).await?;
            }
            _ => {
                println!("Unhandled webhook event: {}", event_type);
            }
        }

        Ok(())
    }

    fn verify_webhook_signature(&self, payload: &str, signature: &str) -> Result<()> {
        // Verify Stripe webhook signature using HMAC
        let secret = self.config.webhook_secret.as_bytes();
        let expected_sig = hmac_sha256(secret, payload.as_bytes());

        if expected_sig.ne(signature) {
            return Err(AppError::Unauthorized);
        }

        Ok(())
    }

    async fn handle_payment_succeeded(
        &self,
        payment_id: &str,
        db: &Database,
    ) -> Result<()> {
        db.execute(
            "UPDATE stripe_payments SET status = 'succeeded', updated_at = NOW() WHERE payment_intent_id = $1",
            &[payment_id],
        )
        .await?;

        Ok(())
    }

    async fn handle_payment_failed(
        &self,
        payment_id: &str,
        db: &Database,
    ) -> Result<()> {
        db.execute(
            "UPDATE stripe_payments SET status = 'failed', updated_at = NOW() WHERE payment_intent_id = $1",
            &[payment_id],
        )
        .await?;

        Ok(())
    }

    async fn handle_subscription_updated(
        &self,
        subscription_id: &str,
        db: &Database,
    ) -> Result<()> {
        // Fetch subscription from Stripe
        let subscription = stripe::Subscription::retrieve(&self.client, subscription_id.parse()?, vec![])
            .await?;

        // Update in database
        let status = format!("{:?}", subscription.status);
        db.execute(
            "UPDATE users SET subscription_status = $1 WHERE subscription_id = $2",
            &[&status, subscription_id],
        )
        .await?;

        Ok(())
    }

    async fn handle_invoice_payment_failed(
        &self,
        invoice_data: &serde_json::Value,
        db: &Database,
    ) -> Result<()> {
        // Handle failed invoice payment (send email, retry, etc.)
        let customer_id = invoice_data["customer"].as_str().unwrap_or("");
        
        // Find user by Stripe customer ID
        let user = db
            .query_one(
                "SELECT id, email FROM users WHERE stripe_customer_id = $1",
                &[customer_id],
            )
            .await?;

        if let Ok(Some(row)) = user {
            let user_id: String = row.get(0);
            let email: String = row.get(1);

            // Send payment failure email
            send_payment_failure_email(&email).await?;

            log_audit_event(
                &user_id,
                "PAYMENT_FAILED",
                "Invoice payment failed",
                db,
            )
            .await?;
        }

        Ok(())
    }
}

// ==================== HANDLERS ====================

/// Get subscription tiers
#[get("/billing/tiers")]
pub async fn get_subscription_tiers() -> Result<Json<Vec<SubscriptionTierResponse>>> {
    Ok(Json(StripeService::get_subscription_tiers()))
}

/// Create payment intent
#[post("/billing/payment-intent")]
pub async fn create_payment_intent(
    auth: AuthLayer,
    db: Database,
    Json(payload): Json<CreatePaymentIntentRequest>,
) -> Result<Json<CreatePaymentResponse>> {
    let config = StripeConfig::from_env()?;
    let service = StripeService::new(config);

    let response = service
        .create_payment_intent(payload.amount, &payload.currency, &auth.user_id, &payload.description, &db)
        .await?;

    Ok(Json(response))
}

/// Subscribe to plan
#[post("/billing/subscribe")]
pub async fn subscribe_to_plan(
    auth: AuthLayer,
    db: Database,
    Json(payload): Json<SubscribeRequest>,
) -> Result<StatusCode> {
    let config = StripeConfig::from_env()?;
    let service = StripeService::new(config);

    service
        .create_subscription(
            &auth.user_id,
            &auth.user.email,
            &auth.user.name,
            &payload.tier,
            &db,
        )
        .await?;

    Ok(StatusCode::OK)
}

/// Cancel subscription
#[post("/billing/cancel")]
pub async fn cancel_subscription(
    auth: AuthLayer,
    db: Database,
) -> Result<StatusCode> {
    let config = StripeConfig::from_env()?;
    let service = StripeService::new(config);

    service.cancel_subscription(&auth.user_id, &db).await?;

    Ok(StatusCode::OK)
}

/// Stripe webhook
#[post("/webhooks/stripe")]
pub async fn stripe_webhook(
    db: Database,
    headers: HeaderMap,
    body: String,
) -> Result<StatusCode> {
    let signature = headers
        .get("stripe-signature")
        .and_then(|h| h.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    let config = StripeConfig::from_env()?;
    let service = StripeService::new(config);

    service.handle_webhook(&body, signature, &db).await?;

    Ok(StatusCode::OK)
}

// ==================== DATABASE SCHEMA ====================

pub const STRIPE_TABLES: &str = r#"
ALTER TABLE users ADD COLUMN IF NOT EXISTS stripe_customer_id TEXT;
ALTER TABLE users ADD COLUMN IF NOT EXISTS subscription_tier TEXT DEFAULT 'free';
ALTER TABLE users ADD COLUMN IF NOT EXISTS subscription_id TEXT;
ALTER TABLE users ADD COLUMN IF NOT EXISTS subscription_started_at TIMESTAMP WITH TIME ZONE;
ALTER TABLE users ADD COLUMN IF NOT EXISTS subscription_cancelled_at TIMESTAMP WITH TIME ZONE;
ALTER TABLE users ADD COLUMN IF NOT EXISTS subscription_status TEXT;

CREATE TABLE IF NOT EXISTS stripe_payments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    payment_intent_id TEXT NOT NULL UNIQUE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    amount BIGINT NOT NULL,
    currency TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_stripe_payments_user_id ON stripe_payments(user_id);
CREATE INDEX IF NOT EXISTS idx_stripe_payments_status ON stripe_payments(status);
"#;

// ==================== REQUEST TYPES ====================

#[derive(Debug, Deserialize)]
pub struct CreatePaymentIntentRequest {
    pub amount: i64,
    pub currency: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct SubscribeRequest {
    pub tier: String,
}

// ==================== HELPERS ====================

fn hmac_sha256(secret: &[u8], data: &[u8]) -> String {
    use sha2::Sha256;
    use hmac::{Hmac, Mac};
    
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(secret).unwrap();
    mac.update(data);
    format!("{:x}", mac.finalize().into_bytes())
}

async fn send_payment_failure_email(email: &str) -> Result<()> {
    // TODO: Integrate with email service
    Ok(())
}
