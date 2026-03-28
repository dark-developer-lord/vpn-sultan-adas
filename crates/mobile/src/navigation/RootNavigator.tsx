import React from 'react';
import { NavigationContainer } from '@react-navigation/native';
import { createNativeStackNavigator } from '@react-navigation/native-stack';
import { createBottomTabNavigator } from '@react-navigation/bottom-tabs';
import { ActivityIndicator, View } from 'react-native';

import { LoginScreen, TwoFactorScreen, AuthContext, createAuthContext } from '../services/AuthService';
import { VPNListScreen } from '../screens/VPNListScreen';
import { ProfileScreen } from '../screens/ProfileScreen';
import { SettingsScreen } from '../screens/SettingsScreen';

const Stack = createNativeStackNavigator();
const Tab = createBottomTabNavigator();
const { authService, useAuth } = createAuthContext();

const AuthStack = () => {
  const { is2FARequired } = React.useContext(AuthContext) as any;

  return (
    <Stack.Navigator
      screenOptions={{
        headerShown: false,
        animationEnabled: true,
      }}
    >
      {!is2FARequired ? (
        <Stack.Screen
          name="Login"
          component={LoginScreen}
          options={{
            animationTypeForReplace: true,
          }}
        />
      ) : (
        <Stack.Screen
          name="TwoFactor"
          component={TwoFactorScreen}
          options={{
            headerShown: true,
            headerTitle: 'Verify Account',
          }}
        />
      )}
    </Stack.Navigator>
  );
};

const VPNStack = () => {
  return (
    <Stack.Navigator
      screenOptions={{
        headerShown: true,
        headerStyle: {
          backgroundColor: '#f8f9fa',
        },
        headerTitleStyle: {
          fontWeight: '600',
        },
      }}
    >
      <Stack.Screen
        name="VPNList"
        component={VPNListScreen}
        options={{
          headerTitle: 'VPN Servers',
        }}
      />
    </Stack.Navigator>
  );
};

const AppStack = () => {
  return (
    <Tab.Navigator
      screenOptions={{
        headerShown: true,
        tabBarActiveTintColor: '#2196F3',
        tabBarInactiveTintColor: '#95a5a6',
        headerStyle: {
          backgroundColor: '#f8f9fa',
        },
        headerTitleStyle: {
          fontWeight: '600',
        },
      }}
    >
      <Tab.Screen
        name="VPN"
        component={VPNStack}
        options={{
          tabBarLabel: 'VPN',
          tabBarIcon: ({ color }) => <Text style={{ color }}>🛡️</Text>,
          headerShown: false,
        }}
      />
      <Tab.Screen
        name="Profile"
        component={ProfileScreen}
        options={{
          tabBarLabel: 'Profile',
          tabBarIcon: ({ color }) => <Text style={{ color }}>👤</Text>,
        }}
      />
      <Tab.Screen
        name="Settings"
        component={SettingsScreen}
        options={{
          tabBarLabel: 'Settings',
          tabBarIcon: ({ color }) => <Text style={{ color }}>⚙️</Text>,
        }}
      />
    </Tab.Navigator>
  );
};

export const RootNavigator = () => {
  const authContext = useAuth();
  const [state, dispatch] = React.useReducer(
    (prevState: any, action: any) => {
      switch (action.type) {
        case 'RESTORE_TOKEN':
          return {
            ...prevState,
            isLoading: false,
          };
        case 'SIGN_IN':
          return {
            ...prevState,
            isSignout: false,
          };
        case 'SIGN_OUT':
          return {
            ...prevState,
            isSignout: true,
          };
      }
    },
    {
      isLoading: true,
      isSignout: true,
      userToken: null,
    }
  );

  React.useEffect(() => {
    const bootstrapAsync = async () => {
      try {
        // Fetch the token from storage then navigate to appropriate place
        const token = await authService.getToken();
        dispatch({ type: 'RESTORE_TOKEN' });

        if (token) {
          dispatch({ type: 'SIGN_IN' });
        }
      } catch (e) {
        // Restoring token failed
      }
    };

    bootstrapAsync();
  }, []);

  if (state.isLoading) {
    return (
      <View style={{ flex: 1, justifyContent: 'center', alignItems: 'center' }}>
        <ActivityIndicator size="large" />
      </View>
    );
  }

  return (
    <AuthContext.Provider value={authContext}>
      <NavigationContainer>
        {state.isSignout ? <AuthStack /> : <AppStack />}
      </NavigationContainer>
    </AuthContext.Provider>
  );
};

// Helper component
const Text = ({ style, ...props }: any) => (
  <React.Fragment {...props}>{props.children}</React.Fragment>
);
