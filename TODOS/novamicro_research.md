# JSON Request and Response Structure Comparison: Amazon Nova Micro vs Anthropic Claude API

Implementing a parser for Amazon Nova requires understanding the fundamental differences in both request formats and JSON response structures compared to Anthropic's Claude API. While both services offer powerful text generation capabilities, their request and response formats differ significantly in organization, field naming conventions, and streaming implementations.

## Request Structure Comparison

Understanding the request formats is crucial for proper implementation, as they reveal architectural differences that impact response parsing.

### Amazon Nova Micro (Invoke API) Request Format
Nova's Invoke API requires a specific request structure with mandatory `schemaVersion` and follows the messages-v1 format:

```json
{
  "schemaVersion": "messages-v1",
  "messages": [
    {
      "role": "user",
      "content": [
        {
          "text": "Describe the purpose of a 'hello world' program in one line."
        }
      ]
    }
  ],
  "system": [
    {
      "text": "You are a helpful assistant."
    }
  ],
  "inferenceConfig": {
    "maxTokens": 500,
    "temperature": 0.7,
    "topP": 0.9,
    "topK": 20
  }
}
```

### Anthropic Claude API Request Format
Claude's Messages API uses a cleaner structure without version schema requirements:

```json
{
  "model": "claude-3-5-sonnet-20241022",
  "max_tokens": 500,
  "temperature": 0.7,
  "top_p": 0.9,
  "messages": [
    {
      "role": "user",
      "content": "Describe the purpose of a 'hello world' program in one line."
    }
  ],
  "system": "You are a helpful assistant."
}
```

**Key Request Differences:**
- **Schema Version**: Nova requires `"schemaVersion": "messages-v1"` in all requests
- **Parameter Names**: Nova uses `inferenceConfig.maxTokens` vs Claude's `max_tokens` 
- **System Prompts**: Nova requires an array of objects with `text` fields, Claude accepts a simple string
- **Content Structure**: Nova always requires content as arrays with typed objects, Claude allows simple strings
- **Model Specification**: Claude includes the model in the request body, Nova specifies it in the API call

## Core architectural differences shape response formats

The most significant difference between Nova and Claude lies in their architectural approach. **Amazon Nova operates through AWS Bedrock's Invoke API**, providing responses wrapped in AWS service metadata and an additional body layer, while **Anthropic's Claude API delivers responses in a more direct format**. This fundamental difference impacts every aspect of response parsing.

When working with Nova through Bedrock's Invoke API, responses include both `ResponseMetadata` containing AWS-specific information like request IDs and HTTP status codes, plus the actual model output nested within a `body` field. Claude's responses, by contrast, place the core content at the top level without this extra wrapping. This means Nova parsers must navigate through both the metadata layer and the body wrapper to access the actual model output.

The Invoke API requires responses to be accessed through `response["body"]` before reaching the actual content, and all model responses must include `"schemaVersion": "messages-v1"` in the request payload to ensure proper response formatting.

## Non-streaming response structures reveal key differences

Based on the official documentation, here are the verified response structures:

### Amazon Nova Micro (Invoke API) - Verified Structure
Based on AWS documentation, Nova Invoke API responses follow this structure:

```json
{
  "ResponseMetadata": {
    "RequestId": "12345-67890-abcdef",
    "HTTPStatusCode": 200,
    "HTTPHeaders": {},
    "RetryAttempts": 0
  },
  "body": {
    "output": {
      "message": {
        "role": "assistant",
        "content": [
          {
            "text": "Here is my response to your query."
          }
        ]
      }
    },
    "stopReason": "end_turn",
    "usage": {
      "inputTokens": 25,
      "outputTokens": 150,
      "totalTokens": 175
    }
  }
}
```

### Anthropic Claude API - Verified Structure
Based on Anthropic's official Messages API documentation:

```json
{
  "id": "msg_013Zva2CMHLNnXjNJJKqJ2EF",
  "type": "message",
  "role": "assistant",
  "model": "claude-3-5-sonnet-20241022",
  "content": [
    {
      "type": "text",
      "text": "Here is my response to your query."
    }
  ],
  "stop_reason": "end_turn",
  "stop_sequence": null,
  "usage": {
    "input_tokens": 25,
    "output_tokens": 150
  }
}
```

The structural differences are immediately apparent. **Nova nests its content four levels deep** (`body.output.message.content[0].text`), while **Claude places content at the root level** (`content[0].text`). Additionally, Nova uses camelCase for token counts (`inputTokens`, `outputTokens`), while Claude uses snake_case (`input_tokens`, `output_tokens`).

## Streaming responses diverge even more dramatically

The streaming implementations represent the most complex parsing challenge when migrating between these APIs. Nova's Invoke API streaming and Claude use entirely different event structures and content delivery mechanisms.

### Nova Streaming Format (Invoke API) - Verified
Nova's streaming chunks are returned as base64-encoded JSON that must be decoded:

```json
{
  "ResponseMetadata": {
    "RequestId": "12345-67890-abcdef",
    "HTTPStatusCode": 200
  },
  "body": {
    "chunk": {
      "bytes": "eyJtZXNzYWdlU3RhcnQiOnsicm9sZSI6ImFzc2lzdGFudCJ9fQ=="
    }
  }
}
```

**Decoded chunk content:**
```json
{"messageStart":{"role":"assistant"}}
{"contentBlockStart":{"start":{"text":""},"contentBlockIndex":0}}
{"contentBlockDelta":{"delta":{"text":"Here is the "},"contentBlockIndex":0}}
{"contentBlockDelta":{"delta":{"text":"streaming response."},"contentBlockIndex":0}}
{"metadata":{"usage":{"inputTokens":25,"outputTokens":150,"totalTokens":175}}}
```

### Claude Streaming Format (SSE) - Verified
Claude uses Server-Sent Events (SSE) with explicit event types and data fields:

```
event: message_start
data: {"type": "message_start", "message": {"id": "msg_1nZdL29xx5MUA1yADyHTEsnR8uuvGzszyY", "type": "message", "role": "assistant", "content": [], "model": "claude-3-5-sonnet-20241022", "stop_reason": null, "stop_sequence": null, "usage": {"input_tokens": 25, "output_tokens": 1}}}

event: content_block_start
data: {"type": "content_block_start", "index": 0, "content_block": {"type": "text", "text": ""}}

event: content_block_delta
data: {"type": "content_block_delta", "index": 0, "delta": {"type": "text_delta", "text": "Here is the "}}

event: content_block_delta
data: {"type": "content_block_delta", "index": 0, "delta": {"type": "text_delta", "text": "streaming response."}}

event: content_block_stop
data: {"type": "content_block_stop", "index": 0}

event: message_delta
data: {"type": "message_delta", "delta": {"stop_reason": "end_turn", "stop_sequence": null}, "usage": {"output_tokens": 150}}

event: message_stop
data: {"type": "message_stop"}
```

**Claude uses Server-Sent Events (SSE)** with explicit event types and data fields, while **Nova's Invoke API provides base64-encoded JSON chunks** that must be decoded before parsing. Nova's streaming chunks are nested under `body.chunk.bytes` and require base64 decoding, whereas Claude's events are self-contained with type information at the root level.

## Field mapping reveals naming convention differences

Understanding field equivalencies is crucial for building a robust parser based on verified documentation:

| Purpose | Nova Field (Invoke API) | Claude Field |
|---------|-------------------------|--------------|
| Generated text | `body.output.message.content[0].text` | `content[0].text` |
| Stop reason | `body.stopReason` | `stop_reason` |
| Input token count | `body.usage.inputTokens` | `usage.input_tokens` |
| Output token count | `body.usage.outputTokens` | `usage.output_tokens` |
| Total tokens | `body.usage.totalTokens` | Not provided (must calculate) |
| Message ID | Not provided | `id` |
| Model used | Not in response | `model` |
| Content type | Not specified | `content[0].type` |
| Response type | Not provided | `type` (always "message") |

**Nova provides a `totalTokens` field**, eliminating the need for manual calculation, while **Claude requires adding `input_tokens` and `output_tokens`**. Claude includes a unique message ID and echoes the model name in responses, features absent from Nova responses.

## Nova-specific request and response patterns

Several Nova-specific patterns distinguish it from Claude's implementation:

### Request Requirements
1. **Schema Version**: Nova's Invoke API requires `"schemaVersion": "messages-v1"` in all requests
2. **Content Structure**: Nova requires all content as arrays with typed objects, even for simple text
3. **System Prompts**: Must be arrays of objects with `text` fields
4. **Model ID Format**: Uses format like `"us.amazon.nova-lite-v1:0"`

### Response Characteristics
1. **Base64 Encoding**: Nova's Invoke streaming API returns chunks as base64-encoded JSON within a `body.chunk.bytes` field, requiring additional decoding
2. **Body Wrapper**: All Nova Invoke API responses wrap the actual model output in a `body` field
3. **AWS Metadata**: Includes standard AWS `ResponseMetadata` with request IDs and HTTP status codes
4. **Error Response Format**: Nova wraps errors in AWS-standard formatting with error codes and types

### Claude-specific patterns
1. **Direct Content**: Claude allows simple strings for content or arrays of content blocks
2. **Flexible System Prompts**: Accepts simple strings for system prompts
3. **Rich Metadata**: Includes message IDs, model names, and detailed stop information
4. **Type Safety**: All responses include explicit `type` fields for content blocks

## Practical parser implementation strategies

Building a robust parser requires handling these structural differences systematically. Here's a unified parsing approach for the Invoke API based on verified documentation:

```python
import base64
import json

class UnifiedLLMParser:
    def create_request(self, messages, system_prompt=None, max_tokens=500, temperature=0.7, is_nova=True):
        """Create properly formatted request for each API"""
        if is_nova:
            # Nova Invoke API request format
            request = {
                "schemaVersion": "messages-v1",
                "messages": self._format_nova_messages(messages),
                "inferenceConfig": {
                    "maxTokens": max_tokens,
                    "temperature": temperature,
                    "topP": 0.9,
                    "topK": 20
                }
            }
            if system_prompt:
                request["system"] = [{"text": system_prompt}]
        else:
            # Claude API request format
            request = {
                "model": "claude-3-5-sonnet-20241022",
                "max_tokens": max_tokens,
                "temperature": temperature,
                "messages": self._format_claude_messages(messages)
            }
            if system_prompt:
                request["system"] = system_prompt
        return request
    
    def _format_nova_messages(self, messages):
        """Format messages for Nova's required structure"""
        formatted = []
        for msg in messages:
            formatted.append({
                "role": msg["role"],
                "content": [{"text": msg["content"]}]
            })
        return formatted
    
    def _format_claude_messages(self, messages):
        """Format messages for Claude's flexible structure"""
        return messages  # Claude accepts simple string content
    
    def parse_response(self, response, is_nova=True):
        """Parse response from either API"""
        if is_nova:
            # Navigate Nova's Invoke API nested structure
            body = response.get("body", {})
            message = body.get("output", {}).get("message", {})
            content = message.get("content", [])
            text = content[0]["text"] if content else ""
            
            usage = body.get("usage", {})
            tokens = {
                "input": usage.get("inputTokens", 0),
                "output": usage.get("outputTokens", 0),
                "total": usage.get("totalTokens", 0)
            }
            stop_reason = body.get("stopReason")
            message_id = None
            model = None
        else:
            # Parse Claude's direct structure
            content = response.get("content", [])
            text = content[0]["text"] if content else ""
            
            usage = response.get("usage", {})
            tokens = {
                "input": usage.get("input_tokens", 0),
                "output": usage.get("output_tokens", 0),
                "total": usage.get("input_tokens", 0) + usage.get("output_tokens", 0)
            }
            stop_reason = response.get("stop_reason")
            message_id = response.get("id")
            model = response.get("model")
        
        return {
            "text": text,
            "tokens": tokens,
            "stop_reason": stop_reason,
            "message_id": message_id,
            "model": model
        }
```

For streaming responses, the parser must handle different event structures and base64 decoding:

```python
def parse_streaming_chunk(self, chunk, is_nova=True):
    """Parse streaming chunk from either API"""
    if is_nova:
        # Extract and decode Nova's Invoke API stream structure
        body = chunk.get("body", {})
        chunk_data = body.get("chunk", {})
        
        if "bytes" in chunk_data:
            try:
                # Decode base64 content
                decoded_bytes = base64.b64decode(chunk_data["bytes"])
                decoded_json = json.loads(decoded_bytes.decode('utf-8'))
                
                if "contentBlockDelta" in decoded_json:
                    return decoded_json["contentBlockDelta"]["delta"].get("text", "")
            except (json.JSONDecodeError, UnicodeDecodeError) as e:
                logger.error(f"Failed to decode Nova chunk: {e}")
                return ""
    else:
        # Parse Claude's SSE format
        if chunk.get("type") == "content_block_delta":
            delta = chunk.get("delta", {})
            return delta.get("text", "")
    return ""
```

## Critical parsing considerations for production systems

When implementing parsers for both APIs, several considerations ensure reliability based on official documentation:

**Request Format Validation**
- Nova requires strict adherence to the messages-v1 schema
- Claude offers more flexibility in content formatting
- Both APIs have specific parameter name requirements that must be followed exactly

**Base64 Decoding Requirements** 
Nova's streaming implementation adds a processing step not present in Claude's implementation. All streaming chunks must be decoded before JSON parsing:

```python
def decode_nova_chunk(self, event):
    """Safely decode Nova's base64 streaming chunks"""
    try:
        chunk_bytes = event["body"]["chunk"]["bytes"]
        decoded_data = base64.b64decode(chunk_bytes)
        return json.loads(decoded_data.decode('utf-8'))
    except (KeyError, json.JSONDecodeError, UnicodeDecodeError) as e:
        logger.error(f"Failed to decode Nova chunk: {e}")
        return {}
```

**Timeout Configuration**
The timeout period for inference calls to Amazon Nova is 60 minutes. By default, AWS SDK clients timeout after 1 minute. We recommend that you increase the read timeout period of your AWS SDK client to at least 60 minutes:

```python
client = boto3.client(
    "bedrock-runtime", 
    region_name="us-east-1",
    config=Config(
        connect_timeout=3600,  # 60 minutes
        read_timeout=3600,     # 60 minutes
        retries={'max_attempts': 1}
    )
)
```

**Token Counting Mechanisms**
- Nova provides immediate total token calculation for simplified billing tracking
- Claude's approach requires manual summation of input and output tokens
- Both services provide final token counts only after stream completion for streaming responses

**Error Handling Patterns** 
- Nova uses AWS-standard error formatting with specific error codes like `ModelErrorException`
- Claude's errors follow a simpler structure but may include detailed stop reasons and sequences
- Region-specific availability must be considered for Nova models

**Content Block Handling**
- Claude explicitly types all content blocks with `type` fields
- Nova's content structure accommodates future multimodal capabilities through its array-based approach
- Both APIs support multimodal content but with different formatting requirements

The key to successful implementation lies in understanding these structural differences and building abstractions that handle both the AWS service layer, the body wrapper, base64 decoding requirements, and the model-specific request/response formats. By accounting for the deeply nested structure, different field names, base64 encoding, request format requirements, and streaming event patterns, developers can create robust parsers that efficiently process both Nova and Claude responses while maintaining compatibility across different implementation approaches.