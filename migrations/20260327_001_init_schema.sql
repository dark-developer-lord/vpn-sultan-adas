-- Create enum types
CREATE TYPE subscription_plan AS ENUM ('free', 'pro', 'enterprise');
CREATE TYPE user_role AS ENUM ('admin', 'user');
CREATE TYPE peer_status AS ENUM ('active', 'revoked', 'inactive');
CREATE TYPE node_status AS ENUM ('online', 'offline', 'degraded');

-- Users table
CREATE TABLE users (
  id UUID PRIMARY KEY,
  email VARCHAR(255) UNIQUE NOT NULL,
  password_hash VARCHAR(255) NOT NULL,
  hashed_refresh_token VARCHAR(255),
  created_at TIMESTAMPTZ NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL,
  deleted_at TIMESTAMPTZ,
  mfa_enabled BOOLEAN DEFAULT FALSE,
  mfa_secret_encrypted BYTEA
);

-- Subscriptions table
CREATE TABLE subscriptions (
  id UUID PRIMARY KEY,
  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  plan subscription_plan NOT NULL,
  status VARCHAR(50) NOT NULL,
  data_limit_gb INTEGER,
  max_peers INTEGER NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  renews_at TIMESTAMPTZ
);

-- VPN Nodes table
CREATE TABLE vpn_nodes (
  id UUID PRIMARY KEY,
  name VARCHAR(255) UNIQUE NOT NULL,
  public_ip VARCHAR(45) NOT NULL,
  internal_ip VARCHAR(45) NOT NULL,
  wg_public_key VARCHAR(255) NOT NULL UNIQUE,
  certificate_pem TEXT NOT NULL,
  certificate_pem_expires_at TIMESTAMPTZ NOT NULL,
  country_code VARCHAR(2),
  region VARCHAR(100),
  status node_status NOT NULL,
  last_heartbeat_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL
);

-- VPN Peers table
CREATE TABLE vpn_peers (
  id UUID PRIMARY KEY,
  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  node_id UUID NOT NULL REFERENCES vpn_nodes(id) ON DELETE CASCADE,
  name VARCHAR(255),
  private_key_encrypted BYTEA NOT NULL,
  private_key_nonce BYTEA NOT NULL,
  public_key VARCHAR(255) NOT NULL,
  allowed_ips VARCHAR(255),
  endpoint VARCHAR(255),
  status peer_status NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  last_connected_at TIMESTAMPTZ,
  revoked_at TIMESTAMPTZ,
  UNIQUE(user_id, node_id, public_key)
);

-- Traffic Stats table
CREATE TABLE traffic_stats (
  id UUID PRIMARY KEY,
  peer_id UUID NOT NULL REFERENCES vpn_peers(id) ON DELETE CASCADE,
  node_id UUID NOT NULL REFERENCES vpn_nodes(id) ON DELETE CASCADE,
  bytes_sent BIGINT NOT NULL DEFAULT 0,
  bytes_received BIGINT NOT NULL DEFAULT 0,
  timestamp TIMESTAMPTZ NOT NULL
);

-- Audit Logs table
CREATE TABLE audit_logs (
  id UUID PRIMARY KEY,
  user_id UUID REFERENCES users(id),
  action VARCHAR(255) NOT NULL,
  resource_type VARCHAR(100),
  resource_id UUID,
  status VARCHAR(50),
  ip_address VARCHAR(45),
  user_agent TEXT,
  details JSONB,
  created_at TIMESTAMPTZ NOT NULL
);

-- API Keys table
CREATE TABLE api_keys (
  id UUID PRIMARY KEY,
  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  key_hash VARCHAR(255) UNIQUE NOT NULL,
  name VARCHAR(255),
  created_at TIMESTAMPTZ NOT NULL,
  last_used_at TIMESTAMPTZ,
  expires_at TIMESTAMPTZ,
  scope VARCHAR(255)
);

-- Sessions table
CREATE TABLE sessions (
  id UUID PRIMARY KEY,
  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  token_hash VARCHAR(255) UNIQUE NOT NULL,
  expires_at TIMESTAMPTZ NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  ip_address VARCHAR(45),
  user_agent TEXT
);

-- Registration tokens table
CREATE TABLE registration_tokens (
  id UUID PRIMARY KEY,
  token_hash VARCHAR(255) UNIQUE NOT NULL,
  node_id UUID REFERENCES vpn_nodes(id),
  created_at TIMESTAMPTZ NOT NULL,
  expires_at TIMESTAMPTZ NOT NULL,
  redeemed_at TIMESTAMPTZ
);

-- Create indexes
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_deleted_at ON users(deleted_at);
CREATE INDEX idx_subscriptions_user_id ON subscriptions(user_id);
CREATE INDEX idx_vpn_nodes_status ON vpn_nodes(status);
CREATE INDEX idx_vpn_nodes_last_heartbeat ON vpn_nodes(last_heartbeat_at);
CREATE INDEX idx_vpn_peers_user_id ON vpn_peers(user_id);
CREATE INDEX idx_vpn_peers_node_id ON vpn_peers(node_id);
CREATE INDEX idx_vpn_peers_status ON vpn_peers(status);
CREATE INDEX idx_traffic_stats_peer_id_timestamp ON traffic_stats(peer_id, timestamp);
CREATE INDEX idx_traffic_stats_node_id_timestamp ON traffic_stats(node_id, timestamp);
CREATE INDEX idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at DESC);
CREATE INDEX idx_audit_logs_action ON audit_logs(action);
CREATE INDEX idx_api_keys_user_id ON api_keys(user_id);
CREATE INDEX idx_sessions_user_id ON sessions(user_id);
