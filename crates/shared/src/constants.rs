// Timeouts
pub const JWT_EXPIRY_SECONDS: i64 = 900;  // 15 minutes
pub const REFRESH_TOKEN_EXPIRY_DAYS: i64 = 7;
pub const AGENT_CERT_EXPIRY_DAYS: i64 = 90;
pub const REGISTRATION_TOKEN_EXPIRY_MINUTES: i64 = 15;

// Rate limiting
pub const RATE_LIMIT_REQUESTS_PER_MINUTE: u32 = 200;
pub const CONFIG_DOWNLOAD_LIMIT_PER_DAY: u32 = 1;

// Defaults
pub const DEFAULT_WG_PORT: u16 = 51820;
pub const DEFAULT_API_PORT: u16 = 3000;
pub const DEFAULT_AGENT_PORT: u16 = 9999;

// Subscription defaults
pub const FREE_TIER_MAX_PEERS: i32 = 1;
pub const PRO_TIER_MAX_PEERS: i32 = 3;
pub const FREE_TIER_DATA_LIMIT_GB: i32 = 0;  // Unlimited for MVP
pub const PRO_TIER_DATA_LIMIT_GB: i32 = 100;

// Database
pub const DB_CONNECTION_POOL_SIZE: u32 = 50;

// Crypto
pub const AES_KEY_SIZE: usize = 32;  // 256 bits
pub const NONCE_SIZE: usize = 12;    // 96 bits for GCM
