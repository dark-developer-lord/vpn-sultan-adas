# Mobile App Integration Guide

## Overview

The VPN Service mobile app provides secure VPN connectivity with real-time monitoring and offline support.

**Platform Support**: iOS 13+, Android 8+
**Framework**: React Native with Expo
**State Management**: Redux with offline persistence
**Authentication**: JWT tokens with 2FA support
**Real-time**: WebSocket for connection metrics

## Architecture

### Core Components

```
├── services/
│   ├── AuthService.tsx       # JWT token management, 2FA verification
│   ├── VPNService.tsx        # VPN connection management
│   ├── OfflineQueue.ts       # Offline operation queue
│   └── WebSocketService.ts   # Real-time metrics streaming
├── screens/
│   ├── LoginScreen.tsx       # Authentication & 2FA
│   ├── VPNListScreen.tsx     # Server selection & connection
│   ├── ProfileScreen.tsx     # User account management
│   └── SettingsScreen.tsx    # App preferences
├── navigation/
│   └── RootNavigator.tsx     # Navigation routing
├── redux/
│   ├── store.ts              # Redux store configuration
│   ├── slices/               # State slices
│   └── middleware/           # Custom middleware
└── App.tsx                   # App entry point
```

## Setup Instructions

### Prerequisites

- Node.js 16+
- npm or yarn
- Expo CLI (`npm install -g expo-cli`)

### Installation

```bash
cd crates/mobile

# Install dependencies
npm install

# Configure environment
cp .env.example .env

# Edit .env with your API endpoint
API_URL=http://your-api-server:8080
```

### Running on Simulator

```bash
# iOS Simulator (macOS only)
npm run ios

# Android Emulator (requires Android Studio)
npm run android

# Expo CLI (works on both)
npm start
```

### Building for Production

```bash
# Build signed APK for Android
npm run build:android

# Build IPA for iOS
npm run build:ios

# Submit to app stores
eas submit --platform ios
eas submit --platform android
```

## Key Features

### 1. Authentication

**Login Flow**:
```
User Input → API Auth → Token Storage → 2FA Check → App Access
```

**Files**:
- `AuthService.tsx` - Authentication logic
- `LoginScreen.tsx` - UI components

**Token Management**:
- Secure storage with `expo-secure-store`
- Automatic refresh 1 minute before expiry
- Automatic logout on token expiry

**2FA Flow**:
```
Email/Password → 2FA Required → TOTP Code Entry → Final Auth Token
```

### 2. VPN Connection Management

**Real-time Connection**:
- WebSocket connection to `ws://api:8080/api/vpn/stream`
- Real-time metrics updates (1-second intervals)
- Connection status change notifications

**Metrics Tracked**:
- Bytes downloaded/uploaded
- Connection duration
- Active protocol
- Server latency
- Current server

**Connection Selection**:
```
User Taps Server → Connect Request → WebSocket Stream Starts → Metrics Display
```

### 3. Offline Support

**Offline Queue**:
- Queues actions when offline
- Syncs when reconnected
- Persistent storage using AsyncStorage

**Supported Offline Actions**:
- Server selection (queued)
- Settings updates (queued)
- Account preference changes (queued)

**Sync Strategy**:
```typescript
When Online:
  1. Retrieve queued actions
  2. Retry in order
  3. Clear from queue on success
  4. Alert user of failures
```

### 4. State Management

**Redux Structure**:
```
{
  auth: {
    isAuthenticated: boolean,
    user: User,
    token: string,
    is2FARequired: boolean,
    twoFactorToken: string
  },
  vpn: {
    servers: VPNConnection[],
    selectedServer: string | null,
    isConnected: boolean,
    metrics: RealTimeMetrics,
    connectionError: string | null
  },
  network: {
    isOnline: boolean,
    type: 'wifi' | 'cellular' | 'none'
  },
  offline: {
    queue: QueuedAction[],
    isSyncing: boolean
  }
}
```

### 5. Error Handling

**Network Errors**:
- Automatic retry with exponential backoff
- Alert user on persistent failures
- Log all errors for debugging

**Authentication Errors**:
- 401 → Force logout & return to login
- 403 → Show permission error
- Automatic token refresh on 401

**Connection Errors**:
- Retry up to 3 times
- Show error messages with reconnect option
- Log for incident analysis

## API Integration

### Required Backend Endpoints

```
POST   /api/auth/login                  # User login
POST   /api/auth/2fa/verify             # 2FA verification
POST   /api/auth/refresh                # Token refresh
POST   /api/auth/logout                 # Logout
GET    /api/vpn/servers                 # List VPN servers
POST   /api/vpn/connect/:serverId       # Connect to server
POST   /api/vpn/disconnect              # Disconnect VPN
WS     /api/vpn/stream                  # Real-time metrics
GET    /api/user/profile                # Fetch user profile
PUT    /api/user/profile                # Update user profile
```

### Request Format

```typescript
// All requests include auth token
headers: {
  'Authorization': `Bearer ${token}`,
  'Content-Type': 'application/json'
}

// 2FA requests include temporary token
headers: {
  'Authorization': `Bearer ${twoFactorToken}`,
  'Content-Type': 'application/json'
}
```

### WebSocket Message Format

```typescript
// Client -> Server
{
  "type": "get_metrics"  | "subscribe_connection" | "ping"
}

// Server -> Client
{
  "type": "metrics" | "connection_status" | "error",
  "payload": { ... }
}
```

## Performance Optimization

### Bundle Size

- Target: < 50MB for Android APK
- Target: < 30MB for iOS IPA
- Strategies:
  - Tree-shaking unused code
  - Code splitting for screens
  - Lazy loading of modules

### Network Optimization

- Batch API requests where possible
- Cache server list locally
- Resume partial uploads/downloads
- Compress payloads

### Memory Management

- Clean up WebSocket on disconnect
- Clear large objects after use
- Use FlatList for large lists
- Implement proper garbage collection

## Testing

### Unit Tests

```bash
npm run test:unit
```

**Coverage Target**: > 80%

### Integration Tests

```bash
npm run test:integration
```

**Tests**:
- AuthService token refresh
- VPN connection lifecycle
- Offline queue sync
- WebSocket reconnection

### E2E Tests (if using Detox)

```bash
detox build-framework-cache ios
detox build-config ios.sim.release
detox test ios.sim.release
```

## Development

### Debug Mode

```bash
# Enable debug logging
API_DEBUG=1 npm start

# View device logs
npm run logs:ios
npm run logs:android
```

### Network Inspector

Use Flipper or React Native debugger:
```bash
npm run flipper
# or
npm run debugger
```

## Security Best Practices

1. **Token Storage**:
   - Never store tokens in AsyncStorage
   - Use SecureStore for sensitive data
   - Implement token rotation

2. **SSL Pinning**:
   - Implement certificate pinning on iOS/Android
   - Validate server certificates

3. **Data Encryption**:
   - Encrypt sensitive data at rest
   - Use TLS 1.3 for all connections

4. **Permissions**:
   - Request minimum required permissions
   - Handle permission denials gracefully
   - Use privacy-friendly analytics

## Deployment Checklist

- [ ] All tests passing (100% pass rate)
- [ ] No security vulnerabilities (npm audit clean)
- [ ] Bundle size acceptable (< 50MB)
- [ ] Performance baseline established
- [ ] Privacy policy updated
- [ ] Terms of service reviewed
- [ ] Beta testers provide feedback
- [ ] Store submission approved
- [ ] Monitoring configured
- [ ] Incident response plan ready

## Monitoring & Analytics

### Key Metrics

```
• App crash rate
• Startup time (< 3 seconds)
• Connection success rate (> 99%)
• Connection latency (< 100ms)
• Offline sync success rate
• User retention
```

### Error Tracking

- Use Sentry or similar service
- Track all unhandled exceptions
- Monitor performance issues
- Alert on critical errors

## Support & Documentation

**In-App Help**:
- FAQ section in UI
- Troubleshooting guides
- Contact support form

**External Documentation**:
- API docs: https://docs.vpn-service.com/api
- FAQ: https://vpn-service.com/faq
- Support: support@vpn-service.com

## Troubleshooting

### Common Issues

**App Crashes on Startup**
- Check API connectivity
- Verify token is valid
- Clear app cache and data
- Reinstall app

**VPN Won't Connect**
- Check server availability
- Verify user has active subscription
- Check network connectivity
- Restart app

**WebSocket Disconnects**
- Network connectivity issues
- API server restarted
- Token expired (should auto-refresh)
- Token refresh failed → re-login

**High Memory Usage**
- Check for WebSocket memory leaks
- Verify FlatList virtualization working
- Review Redux middleware
- Profile with Hermes performance

## Future Enhancements

1. **Split Tunneling**: Choose apps to exclude from VPN
2. **Auto-Connect**: Connect on app launch
3. **Kill Switch**: Disconnect internet if VPN drops
4. **Ad Blocker**: Built-in ad/malware blocking
5. **Multi-hop**: Chain multiple VPN servers
6. **Smart Routing**: Auto-select best server by location/latency

---

**Last Updated**: 2024
**Version**: 1.0.0
**Status**: Production Ready
