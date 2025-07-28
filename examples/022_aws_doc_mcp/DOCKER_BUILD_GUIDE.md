# Docker Build Guide for AWS Documentation MCP Server (NEW Simple Method)

This guide provides step-by-step instructions for building the AWS Documentation MCP server Docker image from source, since the pre-built image is not available in Docker Hub.

This example demonstrates the NEW simple MCP integration using `with_mcp_client()` method!

## 📋 Prerequisites

- Docker installed and running
- Git for cloning the repository
- ~500MB free disk space
- Internet connection

## 🔧 Build Steps

### 1. Clone the MCP Repository
```bash
git clone https://github.com/awslabs/mcp.git
cd mcp
```

### 2. Navigate to AWS Documentation Server
```bash
cd src/aws-documentation-mcp-server/
```

### 3. Build the Docker Image
```bash
docker build -t awslabs/aws-documentation-mcp-server .
```

**Note**: This step may take 5-15 minutes depending on your internet connection and system performance.

### 4. Verify the Build
```bash
docker images | grep awslabs/aws-documentation-mcp-server
```

You should see output similar to:
```
awslabs/aws-documentation-mcp-server   latest    cb03e4828c50   2 minutes ago   1.85GB
```

### 5. Test the Container (Optional)
```bash
docker run --rm --interactive \
  --env FASTMCP_LOG_LEVEL=DEBUG \
  --env AWS_DOCUMENTATION_PARTITION=aws \
  awslabs/aws-documentation-mcp-server
```

Press `Ctrl+C` to stop the test container.

## 🚀 Return to Stood Example

After successfully building the image, navigate back to the Stood example:

```bash
cd examples/022_aws_doc_mcp
cargo run --example 022_aws_doc_mcp
```

## 🐛 Troubleshooting

### Build Fails with "No space left on device"
```bash
# Clean up Docker system
docker system prune -a

# Try building again
docker build -t awslabs/aws-documentation-mcp-server . --no-cache
```

### Build Fails with Network Errors
```bash
# Retry the build (Docker will resume from cache)
docker build -t awslabs/aws-documentation-mcp-server .

# Or force a clean build
docker build -t awslabs/aws-documentation-mcp-server . --no-cache
```

### Git Clone Fails
```bash
# Use HTTPS instead of SSH
git clone https://github.com/awslabs/mcp.git

# Or download as ZIP if Git is not available
# Visit: https://github.com/awslabs/mcp/archive/refs/heads/main.zip
```

## 📊 Expected Build Time and Resources

- **Build Time**: 5-15 minutes
- **Final Image Size**: ~1.8GB  
- **Peak Build Memory**: ~2GB
- **Network Usage**: ~500MB (dependencies)

The build process will download Node.js dependencies and compile the MCP server, which explains the time and resource requirements.

## ✅ Verification

Once built, you can verify the setup with:

```bash
# From the Stood example directory
./verify_setup.sh
```

This will check for the Docker image, compilation, and all prerequisites.

## 🎯 What to Expect After Building

When you run the NEW example, you'll see:

1. **🐳 MCP Server Setup** - Docker container starting automatically
2. **📚 Tool Discovery** - Automatic discovery of AWS documentation tools
3. **🔍 Direct Verification** - Testing MCP tools work correctly  
4. **🤖 Agent Creation** - Using NEW `with_mcp_client()` method (one line!)
5. **🎯 Success Indicators** - Clear messages when MCP tools are actually used

The example demonstrates both direct MCP tool usage and agent-based usage with verification.