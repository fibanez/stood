#!/bin/bash
# Script to run Mistral Large 3 tool calling test
# Usage: Set your AWS credentials as environment variables and run this script

if [ -z "$AWS_ACCESS_KEY_ID" ]; then
    echo "❌ AWS_ACCESS_KEY_ID is not set"
    echo "Please export your AWS credentials:"
    echo "  export AWS_ACCESS_KEY_ID=your_key"
    echo "  export AWS_SECRET_ACCESS_KEY=your_secret"
    echo "  export AWS_SESSION_TOKEN=your_token  # if using temporary credentials"
    echo "  export AWS_REGION=us-east-1"
    exit 1
fi

if [ -z "$AWS_REGION" ]; then
    export AWS_REGION=us-east-1
    echo "ℹ️  Using default region: us-east-1"
fi

echo "🚀 Running Mistral Large 3 tool calling test..."
echo ""
cargo run --example 032_test_mistral_tools
