#!/bin/bash
# Verification script for 022_aws_doc_mcp example (NEW Simple Method)

echo "🔍 Verifying AWS Documentation MCP Example Setup (NEW Simple Method)"
echo "====================================================================="

# Check if we're in the right directory
if [[ ! -f "022_aws_documentation_mcp.rs" ]]; then
    echo "❌ Not in the correct directory. Please run from: examples/022_aws_doc_mcp/"
    exit 1
fi

echo "✅ In correct directory: $(pwd)"

# Check if Docker is available
if command -v docker >/dev/null 2>&1; then
    echo "✅ Docker is available"
    docker --version
else
    echo "❌ Docker not found. Please install Docker first."
    exit 1
fi

# Check if Rust is available
if command -v cargo >/dev/null 2>&1; then
    echo "✅ Cargo is available"
    cargo --version
else
    echo "❌ Cargo not found. Please install Rust first."
    exit 1
fi

# Check if the example compiles
echo "🔧 Checking compilation..."
if cargo check --example 022_aws_doc_mcp >/dev/null 2>&1; then
    echo "✅ Example compiles successfully"
else
    echo "❌ Compilation failed. Run 'cargo check --example 022_aws_doc_mcp' for details."
    exit 1
fi

# Check if Docker image exists locally
echo "🐳 Checking for Docker image..."
if docker images | grep -q awslabs/aws-documentation-mcp-server; then
    echo "✅ AWS Documentation MCP server image found locally"
    docker images | grep awslabs/aws-documentation-mcp-server
else
    echo "❌ Docker image not found. You need to build it from source:"
    echo ""
    echo "   1. git clone https://github.com/awslabs/mcp.git"
    echo "   2. cd mcp/src/aws-documentation-mcp-server/"
    echo "   3. docker build -t awslabs/aws-documentation-mcp-server ."
    echo ""
    echo "   Or run: ./docker_mcp_setup.sh for detailed guidance"
fi

# Check if Git is available
if command -v git >/dev/null 2>&1; then
    echo "✅ Git is available"
    git --version
else
    echo "⚠️  Git not found. You'll need Git to clone the MCP repository."
fi

echo ""
echo "🎉 Setup verification complete!"
echo ""
echo "📝 Next steps:"
echo "1. If Docker image not found: ./docker_mcp_setup.sh (for build guidance)"
echo "2. If image exists: cargo run --example 022_aws_doc_mcp"
echo "3. Or with debug: RUST_LOG=debug cargo run --example 022_aws_doc_mcp"
echo ""
echo "🎯 What to expect when running:"
echo "   ✅ MCP server connection and tool discovery"
echo "   ✅ Direct MCP tool verification"  
echo "   ✅ Agent creation using NEW with_mcp_client() method"
echo "   ✅ AWS documentation queries with tool call verification"
echo "   ✅ Clear success messages when MCP tools are used"
echo ""
echo "📚 See README.md for detailed usage instructions"