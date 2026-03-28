import http from 'k6/http';
import { check, sleep, group } from 'k6';
import { Rate, Trend, Counter, Gauge } from 'k6/metrics';

// ==================== CUSTOM METRICS ====================

const errorRate = new Rate('errors');
const successRate = new Rate('success');
const loginDuration = new Trend('login_duration');
const peerCreationDuration = new Trend('peer_creation_duration');
const connectionTime = new Trend('connection_time');
const failedLogins = new Counter('failed_logins');
const activeConnections = new Gauge('active_connections');

// ==================== CONFIGURATION ====================

export const options = {
  stages: [
    { duration: '30s', target: 10 },    // Ramp up to 10 users over 30 seconds
    { duration: '1m30s', target: 50 },   // Ramp up to 50 users over 1:30
    { duration: '3m', target: 100 },     // Stay at 100 users for 3 minutes
    { duration: '1m', target: 50 },      // Ramp down to 50 users over 1 minute
    { duration: '30s', target: 0 },      // Ramp down to 0 users
  ],
  thresholds: {
    'http_req_duration': ['p(95)<500', 'p(99)<1000'],  // 95% reqs < 500ms, 99% < 1s
    'http_req_failed': ['rate<0.1'],                    // Error rate < 10%
    'success': ['rate>0.95'],                           // Success rate > 95%
  },
  ext: {
    loadimpact: {
      projectID: 3470657,
      name: 'VPN Service Load Test',
    },
  },
};

// ==================== DATA ====================

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';
const ADMIN_TOKEN = __ENV.ADMIN_TOKEN || 'test-admin-token';

// Test user credentials
const testUsers = [
  { email: 'user1@test.com', password: 'TestPassword123!' },
  { email: 'user2@test.com', password: 'TestPassword123!' },
  { email: 'user3@test.com', password: 'TestPassword123!' },
];

// ==================== TEST SETUP ====================

export function setup() {
  // Create some test users before starting load test
  console.log('Setting up test users...');

  testUsers.forEach((user) => {
    const response = http.post(`${BASE_URL}/auth/register`, JSON.stringify({
      email: user.email,
      password: user.password,
      name: user.email.split('@')[0],
    }), {
      headers: { 'Content-Type': 'application/json' },
    });

    check(response, {
      'user created': (r) => r.status === 200 || r.status === 409,
    });
  });

  return { baseUrl: BASE_URL };
}

// ==================== TESTS ====================

export default function (data) {
  const baseUrl = data.baseUrl;
  const user = testUsers[Math.floor(Math.random() * testUsers.length)];

  // ==================== Authentication Tests ====================

  group('Authentication', () => {
    // Login
    const loginResponse = http.post(`${baseUrl}/auth/login`, JSON.stringify({
      email: user.email,
      password: user.password,
    }), {
      headers: { 'Content-Type': 'application/json' },
    });

    loginDuration.add(loginResponse.timings.duration);
    check(loginResponse, {
      'login successful': (r) => r.status === 200,
      'has access token': (r) => r.json('access_token') !== undefined,
    });

    if (loginResponse.status !== 200) {
      failedLogins.add(1);
      errorRate.add(1);
      return;
    }

    const token = loginResponse.json('access_token');
    successRate.add(1);

    // ==================== VPN Connection Tests ====================

    group('VPN Operations', () => {
      // Get current user profile
      const profileResponse = http.get(`${baseUrl}/users/me`, {
        headers: { Authorization: `Bearer ${token}` },
      });

      check(profileResponse, {
        'profile retrieved': (r) => r.status === 200,
      });

      // Create a peer (VPN node)
      const createPeerResponse = http.post(`${baseUrl}/peers`, JSON.stringify({
        name: `Test Peer ${new Date().getTime()}`,
        description: 'Load test peer',
      }), {
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${token}`,
        },
      });

      peerCreationDuration.add(createPeerResponse.timings.duration);
      check(createPeerResponse, {
        'peer created': (r) => r.status === 200 || r.status === 201,
        'has peer id': (r) => r.json('id') !== undefined,
      });

      if (createPeerResponse.status === 200 || createPeerResponse.status === 201) {
        const peerId = createPeerResponse.json('id');

        // Get peer details
        const getPeerResponse = http.get(`${baseUrl}/peers/${peerId}`, {
          headers: { Authorization: `Bearer ${token}` },
        });

        check(getPeerResponse, {
          'peer retrieved': (r) => r.status === 200,
        });

        // List peers
        const listPeersResponse = http.get(`${baseUrl}/peers?page=1&limit=10`, {
          headers: { Authorization: `Bearer ${token}` },
        });

        check(listPeersResponse, {
          'peers listed': (r) => r.status === 200,
        });
      }
    });

    // ==================== Subscription Tests ====================

    group('Subscription', () => {
      const getSubscriptionResponse = http.get(`${baseUrl}/subscription`, {
        headers: { Authorization: `Bearer ${token}` },
      });

      check(getSubscriptionResponse, {
        'subscription retrieved': (r) => r.status === 200,
      });
    });

    // ==================== Admin Dashboard Tests ====================

    group('Admin Operations', () => {
      // Only test if admin token available
      const adminResponse = http.get(`${baseUrl}/admin/dashboard`, {
        headers: { Authorization: `Bearer ${ADMIN_TOKEN}` },
      });

      check(adminResponse, {
        'admin dashboard' : (r) => r.status === 200 || r.status === 401,
      });
    });
  });

  // Random sleep between 1-3 seconds
  sleep(Math.random() * 2 + 1);
}

// ==================== TEARDOWN ====================

export function teardown(data) {
  console.log('Test complete. Cleaning up...');
  // Could delete test users here if needed
}

// ==================== SPIKE TEST ====================

export function spikeTest() {
  const options = {
    stages: [
      { duration: '1m', target: 10 },      // Normal load
      { duration: '30s', target: 200 },    // Spike to 200 users
      { duration: '1m', target: 10 },      // Back to normal
    ],
  };
}

// ==================== STRESS TEST ====================

export function stressTest() {
  const options = {
    stages: [
      { duration: '2m', target: 100 },
      { duration: '2m', target: 200 },
      { duration: '2m', target: 300 },
      { duration: '2m', target: 400 },
      { duration: '2m', target: 500 },
      { duration: '2m', target: 0 },
    ],
    thresholds: {
      'http_req_duration': ['p(95)<2000', 'p(99)<5000'],
      'http_req_failed': ['rate<0.5'],
    },
  };
}

// ==================== SMOKE TEST ====================

export function smokeTest() {
  const options = {
    vus: 1,
    duration: '1m',
    thresholds: {
      'http_req_failed': ['rate<0.1'],
      'http_req_duration': ['p(99)<1000'],
    },
  };
}

// ==================== RUNNING TESTS ====================

/*
Run different load tests:

1. Default load test:
   k6 run load-test.js

2. With custom base URL:
   k6 run -e BASE_URL=https://api.vpnservice.com load-test.js

3. Spike test:
   k6 run --stage 1m:10 --stage 30s:200 --stage 1m:10 load-test.js

4. Cloud test (K6 Cloud):
   k6 cloud load-test.js

5. With results output:
   k6 run --out csv=results.csv load-test.js

6. With specific VUs and duration:
   k6 run -u 50 -d 5m load-test.js
*/

// ==================== USEFUL METRICS ====================

/*
Custom metrics created:
- errorRate: Percentage of failed requests
- successRate: Percentage of successful requests
- loginDuration: Time taken to login
- peerCreationDuration: Time to create a peer
- connectionTime: VPN connection establishment time
- failedLogins: Count of failed login attempts
- activeConnections: Current active connections

View results:
- Console output shows pass/fail rate
- Thresholds will fail test if not met
- Export to CSV/JSON for further analysis
- View in K6 Cloud dashboard
*/
