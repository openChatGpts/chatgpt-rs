#!/bin/bash

# Quick Start Script for Responses API

echo "🚀 ChatGPT-RS Responses API - Quick Start"
echo "========================================"
echo ""

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo is not installed. Please install Rust first."
    exit 1
fi

# Build the project
echo "📦 Building the project..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "❌ Build failed!"
    exit 1
fi

echo "✅ Build successful!"
echo ""

# Start the server
echo "🌐 Starting API server on http://localhost:6969..."
echo "   Press Ctrl+C to stop"
echo ""

cargo run --bin api_server

# Note: The following will only run after the server is stopped
echo ""
echo "👋 Server stopped. Goodbye!"
