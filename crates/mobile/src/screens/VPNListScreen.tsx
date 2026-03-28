import React, { useState, useEffect, useRef, useCallback } from 'react';
import {
  View,
  Text,
  FlatList,
  TouchableOpacity,
  StyleSheet,
  RefreshControl,
  ActivityIndicator,
  Alert,
} from 'react-native';
import WebSocket from 'isomorphic-ws';
import { AuthContext } from '../services/AuthService';

interface VPNConnection {
  id: string;
  name: string;
  country: string;
  city: string;
  ip: string;
  latency: number;
  isConnected: boolean;
  speed: number;
  protocol: 'OpenVPN' | 'WireGuard' | 'IKEv2';
}

interface RealTimeMetrics {
  bytesDownloaded: number;
  bytesUploaded: number;
  duration: number;
  isConnected: boolean;
}

export const VPNListScreen = ({ navigation }: any) => {
  const { userToken, isOnline } = React.useContext(AuthContext);
  const [servers, setServers] = useState<VPNConnection[]>([]);
  const [loading, setLoading] = useState(false);
  const [selectedServer, setSelectedServer] = useState<string | null>(null);
  const [metrics, setMetrics] = useState<RealTimeMetrics>({
    bytesDownloaded: 0,
    bytesUploaded: 0,
    duration: 0,
    isConnected: false,
  });
  const wsRef = useRef<WebSocket | null>(null);
  const metricsTimerRef = useRef<NodeJS.Timeout | null>(null);

  // WebSocket Real-time Connection
  useEffect(() => {
    if (!userToken || !isOnline) return;

    connectWebSocket();

    return () => {
      if (wsRef.current) {
        wsRef.current.close();
      }
      if (metricsTimerRef.current) {
        clearInterval(metricsTimerRef.current);
      }
    };
  }, [userToken, isOnline]);

  const connectWebSocket = () => {
    try {
      const wsUrl = 'ws://localhost:8080/api/vpn/stream';
      wsRef.current = new WebSocket(wsUrl, {
        headers: {
          Authorization: `Bearer ${userToken}`,
        },
      });

      wsRef.current.onopen = () => {
        console.log('WebSocket connected');
        // Start metrics polling
        if (metricsTimerRef.current) clearInterval(metricsTimerRef.current);
        metricsTimerRef.current = setInterval(() => {
          wsRef.current?.send(JSON.stringify({ type: 'get_metrics' }));
        }, 1000);
      };

      wsRef.current.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);

          if (data.type === 'metrics') {
            setMetrics(data.payload);
          } else if (data.type === 'connection_status') {
            setSelectedServer(data.payload.serverId);
            setMetrics(prev => ({
              ...prev,
              isConnected: data.payload.isConnected,
            }));
          }
        } catch (error) {
          console.error('Failed to parse WebSocket message:', error);
        }
      };

      wsRef.current.onerror = (error) => {
        console.error('WebSocket error:', error);
      };

      wsRef.current.onclose = () => {
        console.log('WebSocket disconnected');
        // Attempt reconnect after 3 seconds
        setTimeout(connectWebSocket, 3000);
      };
    } catch (error) {
      console.error('Failed to connect WebSocket:', error);
    }
  };

  // Fetch VPN Servers
  const fetchServers = useCallback(async () => {
    if (!userToken) return;

    setLoading(true);
    try {
      const response = await fetch('http://localhost:8080/api/vpn/servers', {
        headers: {
          Authorization: `Bearer ${userToken}`,
        },
      });

      if (response.ok) {
        const data = await response.json();
        setServers(data);
      }
    } catch (error) {
      Alert.alert('Error', 'Failed to fetch VPN servers');
      console.error(error);
    } finally {
      setLoading(false);
    }
  }, [userToken]);

  useEffect(() => {
    fetchServers();
  }, [fetchServers]);

  // Connect to VPN Server
  const connectVPN = async (serverId: string) => {
    if (!userToken) return;

    try {
      const response = await fetch(
        `http://localhost:8080/api/vpn/connect/${serverId}`,
        {
          method: 'POST',
          headers: {
            Authorization: `Bearer ${userToken}`,
          },
        }
      );

      if (response.ok) {
        setSelectedServer(serverId);
        // WebSocket will handle metrics updates
      } else {
        Alert.alert('Connection Failed', 'Failed to connect to VPN server');
      }
    } catch (error) {
      Alert.alert('Error', 'Failed to connect to VPN');
      console.error(error);
    }
  };

  // Disconnect VPN
  const disconnectVPN = async () => {
    if (!userToken || !selectedServer) return;

    try {
      const response = await fetch(
        `http://localhost:8080/api/vpn/disconnect`,
        {
          method: 'POST',
          headers: {
            Authorization: `Bearer ${userToken}`,
          },
        }
      );

      if (response.ok) {
        setSelectedServer(null);
        setMetrics({
          bytesDownloaded: 0,
          bytesUploaded: 0,
          duration: 0,
          isConnected: false,
        });
      }
    } catch (error) {
      Alert.alert('Error', 'Failed to disconnect VPN');
      console.error(error);
    }
  };

  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const formatDuration = (seconds: number): string => {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;
    return `${hours}h ${minutes}m ${secs}s`;
  };

  const renderServer = ({ item }: { item: VPNConnection }) => {
    const isSelected = item.id === selectedServer;

    return (
      <TouchableOpacity
        style={[styles.serverCard, isSelected && styles.selectedCard]}
        onPress={() => (isSelected ? disconnectVPN() : connectVPN(item.id))}
      >
        <View style={styles.serverHeader}>
          <View>
            <Text style={styles.serverName}>{item.name}</Text>
            <Text style={styles.serverLocation}>
              {item.country}, {item.city}
            </Text>
          </View>
          <View style={styles.statusBadge}>
            <Text style={styles.statusText}>
              {isSelected ? '🔒' : '🔓'}
            </Text>
          </View>
        </View>

        <View style={styles.serverDetails}>
          <View style={styles.detail}>
            <Text style={styles.detailLabel}>Latency</Text>
            <Text style={styles.detailValue}>{item.latency}ms</Text>
          </View>
          <View style={styles.detail}>
            <Text style={styles.detailLabel}>Speed</Text>
            <Text style={styles.detailValue}>{item.speed} Mbps</Text>
          </View>
          <View style={styles.detail}>
            <Text style={styles.detailLabel}>Protocol</Text>
            <Text style={styles.detailValue}>{item.protocol}</Text>
          </View>
        </View>

        {isSelected && (
          <TouchableOpacity
            style={styles.disconnectButton}
            onPress={disconnectVPN}
          >
            <Text style={styles.disconnectButtonText}>Disconnect</Text>
          </TouchableOpacity>
        )}
      </TouchableOpacity>
    );
  };

  return (
    <View style={styles.container}>
      {/* Metrics Display */}
      {metrics.isConnected && (
        <View style={styles.metricsContainer}>
          <View style={styles.metric}>
            <Text style={styles.metricLabel}>↓ Downloaded</Text>
            <Text style={styles.metricValue}>
              {formatBytes(metrics.bytesDownloaded)}
            </Text>
          </View>
          <View style={styles.metric}>
            <Text style={styles.metricLabel}>↑ Uploaded</Text>
            <Text style={styles.metricValue}>
              {formatBytes(metrics.bytesUploaded)}
            </Text>
          </View>
          <View style={styles.metric}>
            <Text style={styles.metricLabel}>Duration</Text>
            <Text style={styles.metricValue}>
              {formatDuration(metrics.duration)}
            </Text>
          </View>
        </View>
      )}

      {/* Servers List */}
      <FlatList
        data={servers}
        renderItem={renderServer}
        keyExtractor={item => item.id}
        refreshControl={
          <RefreshControl
            refreshing={loading}
            onRefresh={fetchServers}
          />
        }
        ListEmptyComponent={
          loading ? (
            <ActivityIndicator style={styles.loader} size="large" />
          ) : (
            <Text style={styles.emptyText}>No servers available</Text>
          )
        }
      />
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#f8f9fa',
  },
  metricsContainer: {
    flexDirection: 'row',
    backgroundColor: '#2c3e50',
    padding: 15,
    paddingTop: 20,
    marginBottom: 10,
  },
  metric: {
    flex: 1,
    alignItems: 'center',
  },
  metricLabel: {
    color: '#95a5a6',
    fontSize: 12,
    marginBottom: 5,
  },
  metricValue: {
    color: '#ecf0f1',
    fontSize: 16,
    fontWeight: '600',
  },
  serverCard: {
    backgroundColor: '#fff',
    margin: 10,
    padding: 15,
    borderRadius: 10,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.1,
    shadowRadius: 4,
    elevation: 3,
  },
  selectedCard: {
    backgroundColor: '#e3f2fd',
    borderWidth: 2,
    borderColor: '#2196F3',
  },
  serverHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 10,
  },
  serverName: {
    fontSize: 16,
    fontWeight: '600',
    color: '#2c3e50',
  },
  serverLocation: {
    fontSize: 12,
    color: '#7f8c8d',
    marginTop: 4,
  },
  statusBadge: {
    width: 40,
    height: 40,
    borderRadius: 20,
    backgroundColor: '#ecf0f1',
    justifyContent: 'center',
    alignItems: 'center',
  },
  statusText: {
    fontSize: 20,
  },
  serverDetails: {
    flexDirection: 'row',
    justifyContent: 'space-around',
    paddingVertical: 10,
    borderTopWidth: 1,
    borderBottomWidth: 1,
    borderColor: '#ecf0f1',
  },
  detail: {
    alignItems: 'center',
  },
  detailLabel: {
    fontSize: 11,
    color: '#95a5a6',
    marginBottom: 4,
  },
  detailValue: {
    fontSize: 14,
    fontWeight: '600',
    color: '#2c3e50',
  },
  disconnectButton: {
    backgroundColor: '#e74c3c',
    borderRadius: 6,
    padding: 10,
    marginTop: 10,
    alignItems: 'center',
  },
  disconnectButtonText: {
    color: '#fff',
    fontWeight: '600',
  },
  loader: {
    marginTop: 50,
  },
  emptyText: {
    textAlign: 'center',
    color: '#7f8c8d',
    marginTop: 50,
  },
});
