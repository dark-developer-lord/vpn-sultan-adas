# Feature Specifications: Advanced VPN Features

**Document Version**: 1.0
**Last Updated**: 2024
**Status**: Ready for Implementation

---

## 1. Split Tunneling Feature

### Overview
Allow users to exclude specific applications and/or websites from VPN tunnel. Traffic destined for excluded targets routes through user's regular internet connection.

### Use Cases
1. **Banking**: User excludes banking app to maintain local banking connection detection
2. **Work**: Exclude corporate VPN to avoid conflicts
3. **Streaming**: Exclude streaming apps to avoid geo-restriction
4. **Gaming**: Exclude gaming servers for lower latency
5. **Performance**: Exclude performance-critical applications

---

### Functional Requirements

#### Database Schema
```sql
-- Split tunnel configuration
CREATE TABLE split_tunnel_config (
  id UUID PRIMARY KEY,
  user_id UUID REFERENCES users(id) ON DELETE CASCADE,
  enabled BOOLEAN DEFAULT FALSE,
  mode TEXT CHECK (mode IN ('whitelist', 'blacklist')), -- apps to exclude or include
  created_at TIMESTAMP DEFAULT NOW(),
  updated_at TIMESTAMP DEFAULT NOW()
);

-- Excluded applications
CREATE TABLE split_tunnel_apps (
  id UUID PRIMARY KEY,
  config_id UUID REFERENCES split_tunnel_config(id) ON DELETE CASCADE,
  app_bundle_id TEXT, -- iOS: com.app.name, Android: com.package.name, Windows: program.exe
  app_name TEXT,
  platform TEXT CHECK (platform IN ('ios', 'android', 'windows', 'macos', 'linux')),
  created_at TIMESTAMP DEFAULT NOW()
);

-- Excluded domains/IPs
CREATE TABLE split_tunnel_networks (
  id UUID PRIMARY KEY,
  config_id UUID REFERENCES split_tunnel_config(id) ON DELETE CASCADE,
  domain TEXT, -- example.com or *.example.com
  ip_address CIDR, -- 192.168.0.0/24
  port INTEGER, -- optional, NULL = all ports
  protocol TEXT CHECK (protocol IN ('tcp', 'udp', 'both')),
  created_at TIMESTAMP DEFAULT NOW()
);
```

#### API Endpoints

**GET /api/v1/vpn/split-tunnel/config**
```json
{
  "enabled": true,
  "mode": "blacklist",
  "apps": [
    {
      "id": "uuid",
      "app_bundle_id": "com.banking.app",
      "app_name": "Bank of America",
      "platform": "ios"
    }
  ],
  "networks": [
    {
      "id": "uuid",
      "domain": "*.banking.com",
      "protocol": "tcp",
      "port": 443
    }
  ]
}
```

**POST /api/v1/vpn/split-tunnel/config**
```json
{
  "enabled": true,
  "mode": "blacklist"
}
```

**POST /api/v1/vpn/split-tunnel/apps**
```json
{
  "app_bundle_id": "com.banking.app",
  "app_name": "Bank of America",
  "platform": "ios"
}
```

**POST /api/v1/vpn/split-tunnel/networks**
```json
{
  "domain": "*.banking.com",
  "protocol": "tcp",
  "port": 443
}
```

**DELETE /api/v1/vpn/split-tunnel/apps/{app_id}**
**DELETE /api/v1/vpn/split-tunnel/networks/{network_id}**

---

### Mobile Implementation

#### iOS (Swift Implementation)
```swift
class SplitTunnelManager {
    // Exclude specific apps from VPN tunnel
    func configureExcludedApps(_ appBundleIds: [String]) {
        let settingsDict: [String: Any] = [
            "ExcludedApps": appBundleIds
        ]
        // Update NEPacketTunnelProvider configuration
        updateVPNConfiguration(with: settingsDict)
    }
    
    // Monitor app launches and exclude from tunnel
    func monitorAppBundles(_ bundleIds: [String]) {
        // Use ProcessMonitor or similar to detect app launches
        // Dynamically update routing rules
    }
}
```

#### Android (Java Implementation)
```java
public class SplitTunnelManager {
    private VpnService vpnService;
    
    public void configureExcludedApps(List<String> packageNames) {
        // Build excluded package list
        for (String packageName : packageNames) {
            vpnService.addDisallowedApplication(packageName);
        }
    }
    
    public void setExcludedDomains(List<String> domains) {
        // Create separate routing table for excluded domains
        // Use eBPF rules to redirect traffic
    }
}
```

---

### Desktop Implementation

#### Windows (Uses WinDivert)
```cpp
class SplitTunnelManager {
public:
    void ExcludeApplication(const std::string& processName) {
        // Use Windows Filtering Platform (WFP) API
        // Create filter rule to allow traffic bypass VPN
        UINT64 filter_id;
        CreateFilterRule(processName, &filter_id);
    }
    
private:
    void CreateFilterRule(const std::string& appName, UINT64* filterId) {
        // Configure WFP for application-level routing
        // Map process to traffic and bypass VPN adapter
    }
};
```

#### macOS (Uses pfctl)
```swift
class SplitTunnelManager {
    func excludeApplication(_ bundleID: String) {
        // Use system_cmds and pfctl rules
        // Get process ID for bundle
        let pid = getProcessID(for: bundleID)
        // Create pf rule to bypass VPN for this process
        updatePFRules(excludeProcess: pid)
    }
}
```

---

### Testing Requirements

**Unit Tests**:
- [ ] Database storage/retrieval of split tunnel config
- [ ] API validation (domain format, CIDR notation, etc.)
- [ ] Configuration merge logic (whitelist ordering)

**Integration Tests**:
- [ ] Create split tunnel configuration (API)
- [ ] Add/remove app from tunnel (API)
- [ ] Update domain exclusion (API)
- [ ] Fetch current configuration (API)

**E2E Tests** (Per Platform):
- [ ] iOS: Exclude banking app, verify traffic bypass
- [ ] Android: Exclude app, verify separate routing
- [ ] Windows: Exclude streaming app, verify direct connection
- [ ] macOS: Exclude app, verify latency improvement
- [ ] Linux: Exclude domain, verify routing

**Network Tests**:
- [ ] Traffic analysis: Verify excluded apps use ISP DNS
- [ ] Performance: Confirm excluded apps have lower latency
- [ ] Overlap: Test when VPN and ISP both excluded (should use ISP)

---

## 2. Kill Switch Feature

### Overview
Automatic internet disconnection if VPN connection drops. Prevents data leakage from unencrypted traffic.

### Use Cases
1. **Privacy**: Prevent ISP from seeing user traffic during VPN outage
2. **Security**: Prevent exposed location/IP
3. **Remote Work**: Enforce security policy for team members
4. **Regulatory Compliance**: Meet compliance requirements for data protection

---

### Functional Requirements

#### Database Schema
```sql
CREATE TABLE kill_switch_config (
  id UUID PRIMARY KEY,
  user_id UUID REFERENCES users(id) ON DELETE CASCADE,
  enabled BOOLEAN DEFAULT TRUE,
  mode TEXT CHECK (mode IN ('strict', 'selective')), -- strict = all traffic, selective = whitelist
  created_at TIMESTAMP DEFAULT NOW(),
  updated_at TIMESTAMP DEFAULT NOW()
);

-- Apps allowed when kill switch activates
CREATE TABLE kill_switch_whitelist (
  id UUID PRIMARY KEY,
  config_id UUID REFERENCES kill_switch_config(id) ON DELETE CASCADE,
  app_bundle_id TEXT,
  app_name TEXT,
  platform TEXT,
  created_at TIMESTAMP DEFAULT NOW()
);
```

#### API Endpoints

**GET /api/v1/vpn/kill-switch/config**
```json
{
  "enabled": true,
  "mode": "strict",
  "whitelist": []
}
```

**PUT /api/v1/vpn/kill-switch/config**
```json
{
  "enabled": true,
  "mode": "selective"
}
```

**POST /api/v1/vpn/kill-switch/whitelist**
```json
{
  "app_bundle_id": "com.apple.mobilesafari",
  "app_name": "Safari",
  "platform": "ios"
}
```

---

### Implementation Details

#### iOS Implementation
```swift
class KillSwitchManager {
    private var vpnConnection: NEVPNConnection?
    private var networkMonitor: NWPathMonitor?
    
    func enableKillSwitch() {
        networkMonitor = NWPathMonitor()
        
        networkMonitor?.pathUpdateHandler = { path in
            if path.status == .satisfied {
                // Check if VPN is connected
                if !self.isVPNConnected() {
                    self.blockAllTraffic()
                }
            }
        }
        
        networkMonitor?.start(queue: DispatchQueue.global())
    }
    
    private func blockAllTraffic() {
        // Create NEFilterManager to block non-VPN traffic
        let filterManager = NEFilterManager.shared()
        // Configure to drop packets not from VPN interface
        filterManager.enabled = true
        filterManager.saveToPreferences()
    }
    
    private func isVPNConnected() -> Bool {
        let vpnConnection = NEVPNConnection()
        return vpnConnection.status == .connected
    }
}
```

#### Android Implementation
```java
public class KillSwitchManager {
    private VpnService vpnService;
    private ConnectivityManager connectivityManager;
    
    public void enableKillSwitch() {
        // Setup network change listener
        connectivityManager.registerNetworkCallback(
            networkRequest,
            new ConnectivityManager.NetworkCallback() {
                @Override
                public void onAvailable(Network network) {
                    if (!isVPNConnected()) {
                        blockAllTraffic();
                    }
                }
            }
        );
    }
    
    private void blockAllTraffic() {
        // Create firewall rules using iptables or netfilter
        // Block all traffic except VPN
        executeCommand("iptables -I OUTPUT -j DROP");
        executeCommand("iptables -I OUTPUT -o tun0 -j ACCEPT");
    }
    
    private boolean isVPNConnected() {
        Network vpnNetwork = connectivityManager.getBoundNetworkForProcess();
        return vpnNetwork != null && isVPNInterface(vpnNetwork);
    }
}
```

#### Windows Implementation
```cpp
class KillSwitchManager {
private:
    HANDLE interfaceChangeNotification;
    
public:
    void EnableKillSwitch() {
        // Register for network interface changes
        NotifyIpInterfaceChange(
            AF_INET,
            IpInterfaceChangeCallback,
            this,
            TRUE,
            &interfaceChangeNotification
        );
    }
    
private:
    static void CALLBACK IpInterfaceChangeCallback(
        PVOID context,
        PMIB_IPINTERFACE_ROW pRow,
        MIB_NOTIFICATION_TYPE NotificationType)
    {
        KillSwitchManager* mgr = (KillSwitchManager*)context;
        if (!mgr->IsVPNConnected()) {
            mgr->BlockAllTraffic();
        }
    }
    
    void BlockAllTraffic() {
        // Use Windows Filtering Platform to block non-VPN traffic
        CreateWFPFilterForVPNOnly();
    }
};
```

#### macOS Implementation
```swift
class KillSwitchManager {
    func enableKillSwitch() {
        // Monitor VPN connection status
        let connectionStatus = NEVPNConnection()
        
        DispatchQueue.main.asyncAfter(deadline: .now() + 1.0) {
            if connectionStatus.status != .connected {
                self.blockAllTraffic()
            }
        }
    }
    
    private func blockAllTraffic() {
        // Use pfctl (packet filter control)
        let process = Process()
        process.executableURL = URL(fileURLWithPath: "/sbin/pfctl")
        process.arguments = ["-E", "-f", "-"] // enable pf with custom rules
        
        // Define rules: block all except VPN interface utun
        let rules = "pass on utun0\nblock all\n"
        // Send rules to stdin
    }
}
```

---

### Testing Requirements

**Unit Tests**:
- [ ] Kill switch enable/disable (API)
- [ ] Whitelist configuration management
- [ ] Network status detection logic

**Integration Tests**:
- [ ] Enable kill switch, simulate VPN drop → verify traffic blocked
- [ ] Disable kill switch, simulate VPN drop → verify traffic flows
- [ ] Kill switch with whitelist → verify whitelist apps accessible
- [ ] Kill switch on connection recovery → verify traffic unblocked

**E2E Tests**:
- [ ] iOS: Enable kill switch, VPN drops → verify all traffic blocked
- [ ] Android: Enable kill switch, network changes → verify firewall active
- [ ] Windows: Kill switch enabled, VPN crashes → verify traffic blocked
- [ ] macOS: Kill switch active, VPN disconnects → verify pf rules active

---

## 3. Multi-Hop (Double VPN) Feature

### Overview
Route traffic through two or more VPN servers for enhanced privacy. Each hop independently encrypts traffic, so neither VPN provider can see final destination.

### Architecture
```
User → VPN Server A (Exit IP 1.1.1.1) → VPN Server B (Exit IP 2.2.2.2) → Internet
```

---

### Database Schema
```sql
CREATE TABLE multi_hop_config (
  id UUID PRIMARY KEY,
  user_id UUID REFERENCES users(id) ON DELETE CASCADE,
  enabled BOOLEAN DEFAULT FALSE,
  hop_1_server_id UUID REFERENCES vpn_servers(id),
  hop_2_server_id UUID REFERENCES vpn_servers(id),
  hop_3_server_id UUID, -- optional for triple hop
  created_at TIMESTAMP DEFAULT NOW(),
  updated_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE multi_hop_routes (
  id UUID PRIMARY KEY,
  config_id UUID REFERENCES multi_hop_config(id) ON DELETE CASCADE,
  route_name TEXT, -- "Netflix", "Work", "Ultra-Privacy"
  servers JSONB, -- array of server IDs in order
  created_at TIMESTAMP DEFAULT NOW()
);
```

### API Endpoints

**GET /api/v1/vpn/multi-hop/config**
```json
{
  "enabled": true,
  "hops": [
    {
      "hop_number": 1,
      "server": {
        "id": "uuid",
        "name": "Amsterdam VPN",
        "latency": 45
      }
    },
    {
      "hop_number": 2,
      "server": {
        "id": "uuid",
        "name": "Singapore VPN",
        "latency": 120
      }
    }
  ]
}
```

**POST /api/v1/vpn/multi-hop/route**
```json
{
  "route_name": "Ultra-Privacy",
  "servers": ["uuid-1", "uuid-2", "uuid-3"]
}
```

---

### Implementation

#### Latency Calculation
```algorithm
Total latency = Sum of latencies to each server + encryption overhead

Optimization Strategy:
1. Measure direct latency to first VPN server (A)
2. Measure latency from A to second server (B) - A shows B's latency
3. Estimate total: direct_to_A + (A_to_B or measured)
4. Warn user if total > 500ms
5. Auto-optimize by suggesting closer servers
```

#### Connection Flow
```
1. User selects 2 servers (Amsterdam & Singapore)
2. Client connects to Amsterdam server (standard VPN)
3. Over Amsterdam tunnel, establish second connection to Singapore
4. Singapore connection is now the exit point
5. All traffic encrypted end-to-end through both servers
```

---

### Testing Requirements

**Performance Tests**:
- [ ] Latency with single hop vs multi-hop (measure overhead)
- [ ] Throughput with multi-hop (typically 20-30% reduction)
- [ ] Connection establishment time
- [ ] Failover if first hop disconnects

**E2E Tests**:
- [ ] Establish multi-hop connection and verify exit IP is from second server
- [ ] Verify traffic is encrypted at both layers
- [ ] Test route switching mid-connection
- [ ] Test with 3-hop configuration

---

## Implementation Timeline

### Phase 1: Foundation (Month 1)
- [ ] Database schema finalized
- [ ] API endpoints implemented
- [ ] Unit tests written

### Phase 2: Mobile Implementation (Month 2)
- [ ] iOS implementation complete
- [ ] Android implementation complete
- [ ] Mobile E2E testing

### Phase 3: Desktop Implementation (Month 3)
- [ ] Windows implementation
- [ ] macOS implementation
- [ ] Linux implementation

### Phase 4: Optimization & Launch (Month 4)
- [ ] Performance optimization
- [ ] Security audit
- [ ] Beta testing with users
- [ ] Production deployment

---

## Infrastructure Requirements

### Server Changes
- Split tunneling: Lightweight, local routing config
- Kill switch: Requires low-latency network monitoring
- Multi-hop: Additional server-to-server connections needed
  - Estimated 20% increase in bandwidth
  - Estimated 10-15% increase in compute

### Monitoring
- [ ] Track split tunnel usage (percentage of users)
- [ ] Monitor kill switch activation frequency
- [ ] Alert if multi-hop connections failing
- [ ] Performance impact tracking

---

**Owner**: [Product Manager]
**Tech Lead**: [Engineering Lead]
**Status**: Ready for Sprint Planning
**Next Review**: Weekly during implementation
