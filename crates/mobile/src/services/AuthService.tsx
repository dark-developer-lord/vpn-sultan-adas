import React, { useState, useCallback, useEffect, useRef } from 'react';
import {
  View,
  Text,
  TextInput,
  TouchableOpacity,
  StyleSheet,
  Alert,
  ActivityIndicator,
  ScrollView,
} from 'react-native';
import AsyncStorage from '@react-native-async-storage/async-storage';
import NetInfo from '@react-native-community/netinfo';
import { useNavigation } from '@react-navigation/native';
import { jwtDecode } from 'jwt-decode';
import * as SecureStore from 'expo-secure-store';

interface AuthContextType {
  isAuthenticated: boolean;
  isLoading: boolean;
  user: User | null;
  userToken: string | null;
  login: (email: string, password: string) => Promise<void>;
  login2FA: (token: string, code: string) => Promise<void>;
  logout: () => Promise<void>;
  refreshToken: () => Promise<void>;
  isOnline: boolean;
}

interface User {
  id: string;
  email: string;
  name: string;
  plan: 'free' | 'basic' | 'pro' | 'enterprise';
  avatar?: string;
  twoFactorEnabled: boolean;
}

interface LoginResponse {
  token: string;
  refreshToken: string;
  user: User;
  twoFactorRequired?: boolean;
  twoFactorToken?: string;
}

// Authentication Service
class AuthService {
  private apiUrl: string;
  private tokenRefreshTimer: NodeJS.Timeout | null = null;

  constructor(apiUrl: string = 'http://localhost:8080') {
    this.apiUrl = apiUrl;
  }

  async login(email: string, password: string): Promise<LoginResponse> {
    try {
      const response = await fetch(`${this.apiUrl}/api/auth/login`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email, password }),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.message || 'Login failed');
      }

      const data: LoginResponse = await response.json();

      // Store tokens securely
      if (data.token) {
        await SecureStore.setItemAsync('authToken', data.token);
      }
      if (data.refreshToken) {
        await SecureStore.setItemAsync('refreshToken', data.refreshToken);
      }

      // Store user info
      await AsyncStorage.setItem('user', JSON.stringify(data.user));

      // Start token refresh timer
      this.scheduleTokenRefresh(data.token);

      return data;
    } catch (error) {
      throw error;
    }
  }

  async login2FA(twoFactorToken: string, code: string): Promise<LoginResponse> {
    try {
      const response = await fetch(`${this.apiUrl}/api/auth/2fa/verify`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ token: twoFactorToken, code }),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.message || '2FA verification failed');
      }

      const data: LoginResponse = await response.json();

      await SecureStore.setItemAsync('authToken', data.token);
      await SecureStore.setItemAsync('refreshToken', data.refreshToken);
      await AsyncStorage.setItem('user', JSON.stringify(data.user));

      this.scheduleTokenRefresh(data.token);

      return data;
    } catch (error) {
      throw error;
    }
  }

  async logout(): Promise<void> {
    if (this.tokenRefreshTimer) {
      clearTimeout(this.tokenRefreshTimer);
    }

    await SecureStore.deleteItemAsync('authToken');
    await SecureStore.deleteItemAsync('refreshToken');
    await AsyncStorage.removeItem('user');
  }

  async getToken(): Promise<string | null> {
    return await SecureStore.getItemAsync('authToken');
  }

  async refreshToken(): Promise<string> {
    try {
      const refreshToken = await SecureStore.getItemAsync('refreshToken');

      if (!refreshToken) {
        throw new Error('No refresh token available');
      }

      const response = await fetch(`${this.apiUrl}/api/auth/refresh`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${refreshToken}`,
        },
      });

      if (!response.ok) {
        throw new Error('Token refresh failed');
      }

      const data = await response.json();
      await SecureStore.setItemAsync('authToken', data.token);

      this.scheduleTokenRefresh(data.token);

      return data.token;
    } catch (error) {
      await this.logout();
      throw error;
    }
  }

  private scheduleTokenRefresh(token: string): void {
    try {
      const decoded: any = jwtDecode(token);
      const expiresIn = decoded.exp * 1000 - Date.now();
      const refreshIn = Math.max(expiresIn - 60000, 60000); // Refresh 1 min before expiry or after 1 min

      if (this.tokenRefreshTimer) {
        clearTimeout(this.tokenRefreshTimer);
      }

      this.tokenRefreshTimer = setTimeout(() => {
        this.refreshToken().catch(console.error);
      }, refreshIn);
    } catch (error) {
      console.error('Failed to decode token:', error);
    }
  }

  async isTokenValid(): Promise<boolean> {
    try {
      const token = await this.getToken();
      if (!token) return false;

      const decoded: any = jwtDecode(token);
      return decoded.exp * 1000 > Date.now();
    } catch (error) {
      return false;
    }
  }
}

// Auth Context
export const createAuthContext = () => {
  const authService = new AuthService();

  return {
    authService,
    useAuth: () => {
      const [state, dispatch] = React.useReducer(authReducer, initialState);
      const navigation = useNavigation<any>();
      const [isOnline, setIsOnline] = useState(true);
      const unsubscribe = useRef<(() => void) | null>(null);

      // Monitor network connectivity
      useEffect(() => {
        unsubscribe.current = NetInfo.addEventListener(state => {
          setIsOnline(!!state.isConnected);
        });

        return () => {
          if (unsubscribe.current) {
            unsubscribe.current();
          }
        };
      }, []);

      // Bootstrap async data
      useEffect(() => {
        const bootstrap = async () => {
          try {
            dispatch({ type: 'RESTORE_TOKEN' });

            const userJson = await AsyncStorage.getItem('user');
            const isValid = await authService.isTokenValid();

            if (userJson && isValid) {
              const user = JSON.parse(userJson);
              const token = await authService.getToken();
              dispatch({ type: 'SIGN_IN', payload: { user, token } });
            } else {
              dispatch({ type: 'SIGN_OUT' });
            }
          } catch (error) {
            console.error('Bootstrap failed:', error);
            dispatch({ type: 'SIGN_OUT' });
          }
        };

        bootstrap();
      }, []);

      const authContext: AuthContextType = {
        isAuthenticated: state.isSignout === false,
        isLoading: state.isLoading,
        user: state.user,
        userToken: state.userToken,
        isOnline,

        login: async (email: string, password: string) => {
          dispatch({ type: 'SET_LOADING', payload: true });
          try {
            const response = await authService.login(email, password);

            if (response.twoFactorRequired) {
              dispatch({
                type: 'SET_2FA_REQUIRED',
                payload: {
                  token: response.twoFactorToken || '',
                  email,
                },
              });
            } else {
              dispatch({
                type: 'SIGN_IN',
                payload: { user: response.user, token: response.token },
              });
            }
          } catch (error: any) {
            Alert.alert('Login Failed', error.message || 'Please try again');
            dispatch({ type: 'SET_LOADING', payload: false });
          }
        },

        login2FA: async (token: string, code: string) => {
          dispatch({ type: 'SET_LOADING', payload: true });
          try {
            const response = await authService.login2FA(token, code);
            dispatch({
              type: 'SIGN_IN',
              payload: { user: response.user, token: response.token },
            });
          } catch (error: any) {
            Alert.alert('2FA Failed', error.message || 'Invalid code');
            dispatch({ type: 'SET_LOADING', payload: false });
          }
        },

        logout: async () => {
          dispatch({ type: 'SET_LOADING', payload: true });
          try {
            await authService.logout();
            dispatch({ type: 'SIGN_OUT' });
          } catch (error) {
            console.error('Logout failed:', error);
          }
        },

        refreshToken: async () => {
          try {
            const newToken = await authService.refreshToken();
            dispatch({ type: 'REFRESH_TOKEN', payload: newToken });
          } catch (error) {
            dispatch({ type: 'SIGN_OUT' });
          }
        },
      };

      return authContext;
    },
  };
};

// Reducer
interface AuthState {
  isLoading: boolean;
  isSignout: boolean;
  userToken: string | null;
  user: User | null;
  twoFactorRequired: boolean;
  twoFactorToken: string | null;
}

const initialState: AuthState = {
  isLoading: true,
  isSignout: true,
  userToken: null,
  user: null,
  twoFactorRequired: false,
  twoFactorToken: null,
};

type AuthAction =
  | { type: 'RESTORE_TOKEN' }
  | { type: 'SIGN_IN'; payload: { user: User; token: string } }
  | { type: 'SIGN_OUT' }
  | { type: 'SET_LOADING'; payload: boolean }
  | { type: 'REFRESH_TOKEN'; payload: string }
  | { type: 'SET_2FA_REQUIRED'; payload: { token: string; email: string } };

const authReducer = (state: AuthState, action: AuthAction): AuthState => {
  switch (action.type) {
    case 'RESTORE_TOKEN':
      return { ...state, isLoading: false };

    case 'SIGN_IN':
      return {
        ...state,
        isSignout: false,
        userToken: action.payload.token,
        user: action.payload.user,
        isLoading: false,
        twoFactorRequired: false,
      };

    case 'SIGN_OUT':
      return {
        ...state,
        isSignout: true,
        userToken: null,
        user: null,
        isLoading: false,
        twoFactorRequired: false,
      };

    case 'SET_LOADING':
      return { ...state, isLoading: action.payload };

    case 'REFRESH_TOKEN':
      return { ...state, userToken: action.payload };

    case 'SET_2FA_REQUIRED':
      return {
        ...state,
        twoFactorRequired: true,
        twoFactorToken: action.payload.token,
        isLoading: false,
      };

    default:
      return state;
  }
};

// Login Screen Component
export const LoginScreen = ({ navigation }: any) => {
  const { login, isLoading, isOnline } = React.useContext(AuthContext);
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [showPassword, setShowPassword] = useState(false);

  const handleLogin = async () => {
    if (!email || !password) {
      Alert.alert('Validation', 'Please fill in all fields');
      return;
    }

    if (!isOnline) {
      Alert.alert('Offline', 'Cannot login while offline. Please check your connection.');
      return;
    }

    await login(email, password);
  };

  return (
    <ScrollView style={styles.container}>
      <View style={styles.innerContainer}>
        <Text style={styles.title}>VPN Service</Text>
        <Text style={styles.subtitle}>Secure your connection</Text>

        <View style={styles.form}>
          <TextInput
            style={styles.input}
            placeholder="Email"
            value={email}
            onChangeText={setEmail}
            editable={!isLoading}
            keyboardType="email-address"
            autoCapitalize="none"
          />

          <View style={styles.passwordContainer}>
            <TextInput
              style={styles.passwordInput}
              placeholder="Password"
              value={password}
              onChangeText={setPassword}
              secureTextEntry={!showPassword}
              editable={!isLoading}
            />
            <TouchableOpacity
              onPress={() => setShowPassword(!showPassword)}
              style={styles.toggleButton}
            >
              <Text>{showPassword ? '👁️' : '🙈'}</Text>
            </TouchableOpacity>
          </View>

          <TouchableOpacity
            style={[styles.button, !isOnline && styles.disabledButton]}
            onPress={handleLogin}
            disabled={isLoading || !isOnline}
          >
            {isLoading ? (
              <ActivityIndicator color="#fff" />
            ) : (
              <Text style={styles.buttonText}>Login</Text>
            )}
          </TouchableOpacity>

          {!isOnline && (
            <Text style={styles.offlineText}>📡 You are offline</Text>
          )}
        </View>

        <TouchableOpacity onPress={() => navigation.navigate('Register')}>
          <Text style={styles.link}>Don't have an account? Sign up</Text>
        </TouchableOpacity>
      </View>
    </ScrollView>
  );
};

// 2FA Screen Component
export const TwoFactorScreen = ({ navigation }: any) => {
  const { login2FA, isLoading } = React.useContext(AuthContext);
  const [code, setCode] = useState('');

  const handleVerify = async () => {
    if (!code || code.length !== 6) {
      Alert.alert('Validation', 'Please enter a valid 6-digit code');
      return;
    }

    await login2FA('', code);
  };

  return (
    <View style={styles.container}>
      <View style={styles.innerContainer}>
        <Text style={styles.title}>Two-Factor Authentication</Text>
        <Text style={styles.subtitle}>Enter your 6-digit code</Text>

        <View style={styles.form}>
          <TextInput
            style={styles.input}
            placeholder="000000"
            value={code}
            onChangeText={setCode}
            keyboardType="number-pad"
            maxLength={6}
            editable={!isLoading}
            textAlign="center"
            fontSize={24}
          />

          <TouchableOpacity
            style={styles.button}
            onPress={handleVerify}
            disabled={isLoading || code.length !== 6}
          >
            {isLoading ? (
              <ActivityIndicator color="#fff" />
            ) : (
              <Text style={styles.buttonText}>Verify</Text>
            )}
          </TouchableOpacity>
        </View>
      </View>
    </View>
  );
};

// Context
export const AuthContext = React.createContext({} as AuthContextType);

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#f5f5f5',
  },
  innerContainer: {
    flex: 1,
    padding: 20,
    justifyContent: 'center',
  },
  title: {
    fontSize: 32,
    fontWeight: 'bold',
    textAlign: 'center',
    marginBottom: 8,
    color: '#333',
  },
  subtitle: {
    fontSize: 16,
    textAlign: 'center',
    marginBottom: 30,
    color: '#666',
  },
  form: {
    marginBottom: 20,
  },
  input: {
    borderWidth: 1,
    borderColor: '#ddd',
    borderRadius: 8,
    padding: 12,
    marginBottom: 12,
    fontSize: 16,
    backgroundColor: '#fff',
  },
  passwordContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    borderWidth: 1,
    borderColor: '#ddd',
    borderRadius: 8,
    marginBottom: 12,
    backgroundColor: '#fff',
  },
  passwordInput: {
    flex: 1,
    padding: 12,
    fontSize: 16,
  },
  toggleButton: {
    padding: 12,
  },
  button: {
    backgroundColor: '#007AFF',
    borderRadius: 8,
    padding: 14,
    alignItems: 'center',
    marginTop: 10,
  },
  disabledButton: {
    backgroundColor: '#ccc',
  },
  buttonText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
  link: {
    color: '#007AFF',
    textAlign: 'center',
    marginTop: 15,
    fontSize: 14,
  },
  offlineText: {
    color: '#FF9500',
    textAlign: 'center',
    marginTop: 10,
    fontSize: 14,
  },
});
