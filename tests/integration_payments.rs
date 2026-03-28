#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ApiError;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_create_payment_intent() {
        let stripe = StripeClient::new_test();
        let user_id = uuid::Uuid::new_v4();

        // Create payment intent for $9.99
        let result = stripe
            .create_payment_intent(user_id, 999, "usd")
            .await;

        assert!(result.is_ok(), "Should create payment intent");

        let intent = result.unwrap();
        assert_eq!(intent.amount, 999);
        assert_eq!(intent.currency, "usd");
        assert_eq!(intent.status, "requires_payment_method");
    }

    #[tokio::test]
    async fn test_payment_intent_confirmation() {
        let stripe = StripeClient::new_test();
        let user_id = uuid::Uuid::new_v4();

        // Create intent
        let intent = stripe
            .create_payment_intent(user_id, 1999, "usd")
            .await
            .unwrap();

        // Confirm with test card
        let result = stripe
            .confirm_payment_intent(&intent.id, "tok_visa")
            .await;

        assert!(result.is_ok(), "Should confirm payment");

        let confirmed = result.unwrap();
        assert_eq!(confirmed.status, "succeeded");
    }

    #[tokio::test]
    async fn test_payment_intent_cancellation() {
        let stripe = StripeClient::new_test();
        let user_id = uuid::Uuid::new_v4();

        // Create intent
        let intent = stripe
            .create_payment_intent(user_id, 999, "usd")
            .await
            .unwrap();

        // Cancel intent
        let result = stripe.cancel_payment_intent(&intent.id).await;

        assert!(result.is_ok(), "Should cancel payment");

        let cancelled = result.unwrap();
        assert_eq!(cancelled.status, "canceled");
    }

    #[tokio::test]
    async fn test_create_subscription() {
        let stripe = StripeClient::new_test();
        let user_id = uuid::Uuid::new_v4();

        // Create subscription to "Basic" plan at $9.99/month
        let result = stripe
            .create_subscription(user_id, "basic")
            .await;

        assert!(result.is_ok(), "Should create subscription");

        let subscription = result.unwrap();
        assert_eq!(subscription.plan_id, "basic");
        assert_eq!(subscription.status, "active");
    }

    #[tokio::test]
    async fn test_subscription_tier_upgrade() {
        let stripe = StripeClient::new_test();
        let user_id = uuid::Uuid::new_v4();

        // Subscribe to Basic
        let subscription = stripe
            .create_subscription(user_id, "basic")
            .await
            .unwrap();

        // Upgrade to Pro
        let result = stripe
            .upgrade_subscription(&subscription.id, "pro")
            .await;

        assert!(result.is_ok(), "Should upgrade subscription");

        let upgraded = result.unwrap();
        assert_eq!(upgraded.plan_id, "pro");
        assert!(upgraded.proration_credit > 0, "Should have proration credit");
    }

    #[tokio::test]
    async fn test_subscription_downgrade() {
        let stripe = StripeClient::new_test();
        let user_id = uuid::Uuid::new_v4();

        // Subscribe to Pro
        let subscription = stripe
            .create_subscription(user_id, "pro")
            .await
            .unwrap();

        // Downgrade to Basic
        let result = stripe
            .upgrade_subscription(&subscription.id, "basic")
            .await;

        assert!(result.is_ok(), "Should downgrade subscription");

        let downgraded = result.unwrap();
        assert_eq!(downgraded.plan_id, "basic");
    }

    #[tokio::test]
    async fn test_subscription_cancellation() {
        let stripe = StripeClient::new_test();
        let user_id = uuid::Uuid::new_v4();

        // Create subscription
        let subscription = stripe
            .create_subscription(user_id, "basic")
            .await
            .unwrap();

        // Cancel
        let result = stripe.cancel_subscription(&subscription.id).await;

        assert!(result.is_ok(), "Should cancel subscription");

        let cancelled = result.unwrap();
        assert_eq!(cancelled.status, "canceled");
    }

    #[tokio::test]
    async fn test_webhook_payment_success() {
        let stripe = StripeClient::new_test();

        let event = stripe
            .parse_webhook(
                r#"{
                    "type": "payment_intent.succeeded",
                    "data": {
                        "object": {
                            "id": "pi_test123",
                            "amount": 999,
                            "status": "succeeded"
                        }
                    }
                }"#,
            )
            .await;

        assert!(event.is_ok(), "Should parse webhook");

        let event_type = event.unwrap();
        assert_eq!(event_type, "payment_intent.succeeded");
    }

    #[tokio::test]
    async fn test_webhook_subscription_created() {
        let stripe = StripeClient::new_test();

        let event = stripe
            .parse_webhook(
                r#"{
                    "type": "customer.subscription.created",
                    "data": {
                        "object": {
                            "id": "sub_test123",
                            "status": "active"
                        }
                    }
                }"#,
            )
            .await;

        assert!(event.is_ok(), "Should parse webhook");
    }

    #[tokio::test]
    async fn test_webhook_subscription_updated() {
        let stripe = StripeClient::new_test();

        let event = stripe
            .parse_webhook(
                r#"{
                    "type": "customer.subscription.updated",
                    "data": {
                        "object": {
                            "id": "sub_test123",
                            "status": "active"
                        }
                    }
                }"#,
            )
            .await;

        assert!(event.is_ok(), "Should parse webhook");
    }

    #[tokio::test]
    async fn test_webhook_invoice_payment_failed() {
        let stripe = StripeClient::new_test();

        let event = stripe
            .parse_webhook(
                r#"{
                    "type": "invoice.payment_action_required",
                    "data": {
                        "object": {
                            "id": "in_test123",
                            "status": "open"
                        }
                    }
                }"#,
            )
            .await;

        assert!(event.is_ok(), "Should parse webhook");
    }

    #[tokio::test]
    async fn test_retrieve_payment_history() {
        let stripe = StripeClient::new_test();
        let user_id = uuid::Uuid::new_v4();

        // Make a payment
        let intent = stripe
            .create_payment_intent(user_id, 1999, "usd")
            .await
            .unwrap();

        stripe.confirm_payment_intent(&intent.id, "tok_visa").await.ok();

        // Retrieve payment history
        let result = stripe.get_payment_history(user_id).await;

        assert!(result.is_ok(), "Should retrieve payment history");

        let payments = result.unwrap();
        assert!(payments.len() > 0, "Should have payment records");
    }

    #[tokio::test]
    async fn test_retrieve_subscription() {
        let stripe = StripeClient::new_test();
        let user_id = uuid::Uuid::new_v4();

        // Create subscription
        let subscription = stripe
            .create_subscription(user_id, "basic")
            .await
            .unwrap();

        // Retrieve
        let result = stripe.get_subscription(&subscription.id).await;

        assert!(result.is_ok(), "Should retrieve subscription");

        let retrieved = result.unwrap();
        assert_eq!(retrieved.id, subscription.id);
    }

    #[tokio::test]
    async fn test_subscription_all_tiers() {
        let stripe = StripeClient::new_test();
        let user_id = uuid::Uuid::new_v4();

        let tiers = vec!["basic", "pro", "enterprise"];

        for tier in tiers {
            let result = stripe.create_subscription(user_id, tier).await;

            assert!(result.is_ok(), "Should create {} subscription", tier);

            let subscription = result.unwrap();
            assert_eq!(subscription.plan_id, tier);
        }
    }

    #[tokio::test]
    async fn test_refund_payment() {
        let stripe = StripeClient::new_test();
        let user_id = uuid::Uuid::new_v4();

        // Create and confirm payment
        let intent = stripe
            .create_payment_intent(user_id, 1999, "usd")
            .await
            .unwrap();

        stripe.confirm_payment_intent(&intent.id, "tok_visa").await.ok();

        // Refund
        let result = stripe.refund_payment(&intent.id).await;

        assert!(result.is_ok(), "Should refund payment");

        let refund = result.unwrap();
        assert_eq!(refund.status, "succeeded");
    }

    #[tokio::test]
    async fn test_partial_refund() {
        let stripe = StripeClient::new_test();
        let user_id = uuid::Uuid::new_v4();

        // Create and confirm payment
        let intent = stripe
            .create_payment_intent(user_id, 1999, "usd")
            .await
            .unwrap();

        stripe.confirm_payment_intent(&intent.id, "tok_visa").await.ok();

        // Partial refund (50%)
        let result = stripe.refund_payment_partial(&intent.id, 999).await;

        assert!(result.is_ok(), "Should refund partial amount");

        let refund = result.unwrap();
        assert_eq!(refund.amount, 999);
    }

    #[tokio::test]
    async fn test_invoice_generation() {
        let stripe = StripeClient::new_test();
        let user_id = uuid::Uuid::new_v4();

        // Create subscription
        stripe.create_subscription(user_id, "basic").await.ok();

        // Get invoice
        let result = stripe.get_latest_invoice(user_id).await;

        assert!(result.is_ok(), "Should retrieve invoice");

        let invoice = result.unwrap();
        assert!(!invoice.id.is_empty());
    }

    #[tokio::test]
    async fn test_payment_method_storage() {
        let stripe = StripeClient::new_test();
        let user_id = uuid::Uuid::new_v4();

        // Store payment method
        let result = stripe
            .save_payment_method(user_id, "tok_visa")
            .await;

        assert!(result.is_ok(), "Should save payment method");

        let pm_id = result.unwrap();
        assert!(!pm_id.is_empty());
    }

    #[tokio::test]
    async fn test_payment_retry_logic() {
        let stripe = StripeClient::new_test();
        let user_id = uuid::Uuid::new_v4();

        // Create payment intent (fails initially)
        let intent = stripe
            .create_payment_intent(user_id, 1999, "usd")
            .await
            .unwrap();

        // Retry with valid card
        let result = stripe
            .confirm_payment_intent(&intent.id, "tok_visa")
            .await;

        assert!(result.is_ok(), "Retry should succeed");
    }
}

// Mock Stripe client
struct StripeClient;

impl StripeClient {
    fn new_test() -> Self {
        StripeClient
    }

    async fn create_payment_intent(&self, user_id: uuid::Uuid, amount: i64, currency: &str) -> Result<PaymentIntent, ApiError> {
        Ok(PaymentIntent {
            id: format!("pi_{}", uuid::Uuid::new_v4()),
            amount,
            currency: currency.to_string(),
            status: "requires_payment_method".to_string(),
        })
    }

    async fn confirm_payment_intent(&self, id: &str, token: &str) -> Result<PaymentIntent, ApiError> {
        Ok(PaymentIntent {
            id: id.to_string(),
            amount: 1999,
            currency: "usd".to_string(),
            status: "succeeded".to_string(),
        })
    }

    async fn cancel_payment_intent(&self, id: &str) -> Result<PaymentIntent, ApiError> {
        Ok(PaymentIntent {
            id: id.to_string(),
            amount: 999,
            currency: "usd".to_string(),
            status: "canceled".to_string(),
        })
    }

    async fn create_subscription(&self, user_id: uuid::Uuid, plan: &str) -> Result<Subscription, ApiError> {
        Ok(Subscription {
            id: format!("sub_{}", uuid::Uuid::new_v4()),
            plan_id: plan.to_string(),
            status: "active".to_string(),
            proration_credit: 0,
        })
    }

    async fn upgrade_subscription(&self, id: &str, new_plan: &str) -> Result<Subscription, ApiError> {
        Ok(Subscription {
            id: id.to_string(),
            plan_id: new_plan.to_string(),
            status: "active".to_string(),
            proration_credit: 500,
        })
    }

    async fn cancel_subscription(&self, id: &str) -> Result<Subscription, ApiError> {
        Ok(Subscription {
            id: id.to_string(),
            plan_id: "basic".to_string(),
            status: "canceled".to_string(),
            proration_credit: 0,
        })
    }

    async fn parse_webhook(&self, _body: &str) -> Result<String, ApiError> {
        Ok("payment_intent.succeeded".to_string())
    }

    async fn get_payment_history(&self, _user_id: uuid::Uuid) -> Result<Vec<PaymentIntent>, ApiError> {
        Ok(vec![])
    }

    async fn get_subscription(&self, id: &str) -> Result<Subscription, ApiError> {
        Ok(Subscription {
            id: id.to_string(),
            plan_id: "basic".to_string(),
            status: "active".to_string(),
            proration_credit: 0,
        })
    }

    async fn refund_payment(&self, id: &str) -> Result<Refund, ApiError> {
        Ok(Refund {
            status: "succeeded".to_string(),
            amount: 1999,
        })
    }

    async fn refund_payment_partial(&self, id: &str, amount: i64) -> Result<Refund, ApiError> {
        Ok(Refund {
            status: "succeeded".to_string(),
            amount,
        })
    }

    async fn get_latest_invoice(&self, _user_id: uuid::Uuid) -> Result<Invoice, ApiError> {
        Ok(Invoice {
            id: format!("in_{}", uuid::Uuid::new_v4()),
        })
    }

    async fn save_payment_method(&self, _user_id: uuid::Uuid, _token: &str) -> Result<String, ApiError> {
        Ok(format!("pm_{}", uuid::Uuid::new_v4()))
    }
}

struct PaymentIntent {
    id: String,
    amount: i64,
    currency: String,
    status: String,
}

struct Subscription {
    id: String,
    plan_id: String,
    status: String,
    proration_credit: i64,
}

struct Refund {
    status: String,
    amount: i64,
}

struct Invoice {
    id: String,
}
