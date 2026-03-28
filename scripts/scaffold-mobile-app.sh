#!/bin/bash

# VPN Service Mobile App Generator (React Native)
# This script scaffolds a React Native Expo app for iOS and Android

set -e

PROJECT_NAME="vpn-service-mobile"
PROJECT_DIR="./$PROJECT_NAME"

echo "🚀 Generating VPN Service Mobile App..."

# Create project with Expo
if ! command -v npx &> /dev/null; then
    echo "❌ Node.js/npm not found. Please install Node.js"
    exit 1
fi

# Initialize Expo project
npx create-expo-app $PROJECT_NAME --template

cd $PROJECT_DIR

echo "📦 Installing dependencies..."

# Install required packages
npm install \
    @react-navigation/native \
    @react-navigation/bottom-tabs \
    react-native-screens \
    react-native-safe-area-context \
    @react-native-community/masked-view \
    axios \
    react-native-keychain \
    react-native-circular-progress-indicator \
    react-native-gesture-handler \
    expo-auth-session \
    expo-web-browser \
    expo-battery \
    expo-network \
    react-native-svg \
    lodash \
    date-fns \
    zustand \
    zod

npm install --save-dev \
    @types/react \
    @types/react-native \
    typescript

echo "📁 Setting up project structure..."

# Create directories
mkdir -p src/{screens,components,services,store,types,utils,navigation,constants}
mkdir -p assets/{images,icons,fonts}

echo "✅ Mobile app scaffolding complete!"
echo ""
echo "📝 Next steps:"
echo "  1. cd $PROJECT_DIR"
echo "  2. Update .env with API endpoint"
echo "  3. npm start"
echo "  4. Scan QR code with Expo Go app"
echo ""
echo "🚀 To build for production:"
echo "  - iOS: eas build --platform ios"
echo "  - Android: eas build --platform android"
echo ""
echo "📚 Learn more: https://docs.expo.dev/"
