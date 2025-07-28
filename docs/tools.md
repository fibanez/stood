# Tool Development Approaches

This guide covers the two primary approaches for creating tools in Stood: **Macro-Based Tools** and **Struct-Based Tools**. Both approaches integrate seamlessly with the `Agent::builder()` pattern.

## Quick Comparison

| Aspect | Macro Tools | Struct Tools |
|--------|-------------|--------------|
| **Syntax** | `#[tool] fn my_tool() -> Result<T, E>` | `impl Tool for MyTool` |
| **Usage** | `.tool(my_tool())` | `.tool(Box::new(MyTool::new()))` |
| **Development Speed** | ⭐⭐⭐⭐⭐ Very Fast | ⭐⭐⭐ Moderate |
| **Flexibility** | ⭐⭐ Limited | ⭐⭐⭐⭐⭐ Full Control |
| **IDE Support** | ⭐⭐⭐ Good | ⭐⭐⭐⭐⭐ Excellent |
| **Debugging** | ⭐⭐⭐ Good | ⭐⭐⭐⭐⭐ Excellent |
| **Best For** | Simple, stateless tools | Production tools with complex requirements |

## Macro-Based Tools

### Overview

Macro-based tools use the `#[tool]` procedural macro to automatically generate tool implementations from simple functions. This approach prioritizes developer velocity and reduces boilerplate.

### Basic Example

```rust
use stood::tool;

# [tool]
/// Calculate a percentage of a value
async fn calculate_percentage(value: f64, percentage: f64) -> Result<f64, String> {
    if percentage < 0.0 || percentage > 100.0 {
        return Err("Percentage must be between 0 and 100".to_string());
    }
    Ok(value * percentage / 100.0)
}

// Usage with agents
let agent = Agent::builder()
    .tool(calculate_percentage())  // ✅ Natural function call syntax
    .build().await?;
```

### What the Macro Generates

The `#[tool]` macro transforms your function into a complete tool implementation:

```rust
// Input: Your function
# [tool]
async fn calculate_percentage(value: f64, percentage: f64) -> Result<f64, String> { ... }

// Generated: Complete tool system
async fn calculate_percentage_impl(value: f64, percentage: f64) -> Result<f64, String> { ... }

struct CalculatePercentageTool;

impl Tool for CalculatePercentageTool {
    fn name(&self) -> &str { "calculate_percentage" }
    fn description(&self) -> &str { "Calculate a percentage of a value" }
    fn parameters_schema(&self) -> Value { /* Auto-generated JSON schema */ }
    async fn execute(&self, params: Option<Value>) -> Result<ToolResult, ToolError> {
        // Parameter extraction and validation
        // Call to calculate_percentage_impl()
        // Result serialization
    }
}

/// Create a new instance of this tool for use with agents
pub fn calculate_percentage() -> Box<dyn Tool> {
    Box::new(CalculatePercentageTool)
}
```

### Advanced Features

#### Custom Tool Metadata

```rust
# [tool(name = "weather_api", description = "Get current weather data")]
async fn get_weather(location: String) -> Result<String, String> {
    // Implementation
}
```

#### Documentation Comments

```rust
# [tool]
/// Get weather information for a specific location
/// 
/// This tool fetches current weather conditions including temperature,
/// humidity, and forecast data from a reliable weather service.
async fn get_weather(
    /// The city or location to get weather for
    location: String,
    /// Optional country code (e.g., "US", "UK")
    country: Option<String>
) -> Result<String, String> {
    // Implementation
}
```

#### Type Support

The macro automatically generates JSON schemas for common Rust types:

```rust
# [tool]
async fn complex_calculation(
    numbers: Vec<f64>,           // → "array" with "number" items
    config: HashMap<String, i32>, // → "object" 
    enabled: bool,               // → "boolean"
    precision: Option<u32>,      // → "integer" (optional)
) -> Result<f64, String> {
    // Implementation
}
```

### When to Use Macro Tools

**✅ Ideal for:**
- Functions that are simple and stateless computations
- Pure functions with no external dependencies
- Quick tool creation for testing
- Prototype development and experimentation

**✅ Examples:**
- Mathematical calculations
- String manipulations
- Data transformations
- Format conversions

## Struct-Based Tools

### Overview

Struct-based tools provide complete control over tool behavior by implementing the `Tool` trait directly. This approach offers maximum flexibility for complex production requirements.

### Basic Example

```rust
use stood::tools::{Tool, ToolResult, ToolError};
use async_trait::async_trait;
use serde_json::{json, Value};

# [derive(Debug)]
pub struct WeatherTool {
    api_key: String,
    client: reqwest::Client,
    cache: Arc<RwLock<HashMap<String, CachedWeather>>>,
    cache_ttl: Duration,
}

impl WeatherTool {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: Duration::from_secs(300), // 5 minutes
        }
    }
    
    pub fn with_cache_ttl(mut self, ttl: Duration) -> Self {
        self.cache_ttl = ttl;
        self
    }
}

# [async_trait]
impl Tool for WeatherTool {
    fn name(&self) -> &str {
        "weather"
    }
    
    fn description(&self) -> &str {
        "Get current weather conditions for any location worldwide"
    }
    
    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "location": {
                    "type": "string",
                    "description": "City name or coordinates (lat,lon)"
                },
                "units": {
                    "type": "string",
                    "enum": ["metric", "imperial", "kelvin"],
                    "default": "metric",
                    "description": "Temperature units"
                }
            },
            "required": ["location"]
        })
    }
    
    async fn execute(&self, parameters: Option<Value>) -> Result<ToolResult, ToolError> {
        // Custom parameter validation
        let params = self.validate_and_parse_params(parameters)?;
        
        // Check cache first
        if let Some(cached) = self.get_cached_weather(&params.location).await {
            return Ok(ToolResult::success(json!(cached)));
        }
        
        // Make API call with retry logic
        let weather_data = self.fetch_weather_with_retry(&params).await?;
        
        // Update cache
        self.cache_weather_data(&params.location, &weather_data).await;
        
        Ok(ToolResult::success(json!(weather_data)))
    }
}

// Usage with agents
let weather_tool = WeatherTool::new(api_key)
    .with_cache_ttl(Duration::from_secs(600));

let tools = vec![
    Box::new(weather_tool) as Box<dyn Tool>,
];
let agent = Agent::builder()
    .tools(tools)
    .build().await?;
```

### Advanced Patterns

#### Builder Pattern Integration

```rust
impl WeatherTool {
    pub fn builder() -> WeatherToolBuilder {
        WeatherToolBuilder::default()
    }
}

pub struct WeatherToolBuilder {
    api_key: Option<String>,
    timeout: Duration,
    cache_ttl: Duration,
    rate_limit: Option<RateLimiter>,
}

impl WeatherToolBuilder {
    pub fn api_key<S: Into<String>>(mut self, key: S) -> Self {
        self.api_key = Some(key.into());
        self
    }
    
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
    
    pub fn build(self) -> Result<WeatherTool, ConfigError> {
        let api_key = self.api_key.ok_or(ConfigError::MissingApiKey)?;
        Ok(WeatherTool::new_with_config(WeatherConfig {
            api_key,
            timeout: self.timeout,
            cache_ttl: self.cache_ttl,
            rate_limiter: self.rate_limit,
        }))
    }
}

// Usage
let weather_tool = WeatherTool::builder()
    .api_key(env::var("WEATHER_API_KEY")?)
    .timeout(Duration::from_secs(10))
    .build()?;
```

#### Complex State Management

```rust
# [derive(Debug)]
pub struct DatabaseTool {
    pool: Arc<Pool<PostgresConnectionManager>>,
    query_cache: Arc<RwLock<LruCache<String, QueryResult>>>,
    metrics: Arc<Metrics>,
    access_control: AccessControl,
}

impl DatabaseTool {
    async fn execute_query(&self, query: &str) -> Result<QueryResult, ToolError> {
        // Rate limiting
        self.check_rate_limit().await?;
        
        // Access control
        self.access_control.validate_query(query)?;
        
        // Check cache
        if let Some(cached) = self.get_cached_result(query).await {
            self.metrics.record_cache_hit();
            return Ok(cached);
        }
        
        // Execute with connection pooling
        let conn = self.pool.get().await
            .map_err(|e| ToolError::ExecutionFailed {
                message: format!("Database connection failed: {}", e)
            })?;
            
        let result = conn.query(query, &[]).await
            .map_err(|e| ToolError::ExecutionFailed {
                message: format!("Query execution failed: {}", e)
            })?;
            
        // Cache and return
        let query_result = QueryResult::from_rows(result);
        self.cache_result(query, &query_result).await;
        self.metrics.record_query_execution();
        
        Ok(query_result)
    }
}
```

### When to Use Struct Tools

**✅ Ideal for:**
- Production tools with complex requirements
- Tools needing persistent state or configuration
- Integration with external services (databases, APIs)
- Tools requiring custom validation or error handling
- Performance-critical implementations

**✅ Examples:**
- Database query tools
- File system operations
- HTTP API clients
- Machine learning inference
- System monitoring tools

## Hybrid Approach

The most powerful pattern combines both approaches in a single application:

```rust
use stood::{Agent, tool};
use stood::tools::builtin::CalculatorTool;

// Macro tools for simple operations
# [tool]
async fn format_currency(amount: f64, currency: &str) -> Result<String, String> {
    Ok(format!("{:.2} {}", amount, currency))
}

# [tool]  
async fn calculate_tip(bill: f64, percentage: f64) -> Result<f64, String> {
    Ok(bill * percentage / 100.0)
}

// Struct tool for complex operations
# [derive(Debug)]
pub struct PaymentProcessorTool {
    stripe_client: stripe::Client,
    fraud_detector: FraudDetector,
    audit_logger: AuditLogger,
}

impl Tool for PaymentProcessorTool {
    // Complex implementation with state management,
    // error handling, logging, etc.
}

// Seamless integration
# [tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let payment_tool = PaymentProcessorTool::new(config).await?;
    
    let tools = vec![
        format_currency(),                           // Macro tool
        calculate_tip(),                            // Macro tool
        Box::new(CalculatorTool::new()) as Box<dyn Tool>, // Builtin tool
        Box::new(payment_tool) as Box<dyn Tool>,    // Custom struct tool
    ];
    
    let agent = Agent::builder()
    .tools(tools)
    .build().await?;
    
    // Agent can use any tool type seamlessly
    let result = agent.execute_agentic(
        "Process a $150 payment with 18% tip, format in USD, and calculate tax"
    ).await?;
    
    Ok(())
}
```

## Capability Comparison

### What Macro Tools Cannot Do

1. **Custom State Management**
   ```rust
   // ❌ Macro tools: No built-in state
   #[tool]
   async fn get_weather(location: String) -> Result<String, String> {
       // Where do I store API keys? Rate limits? Cache?
   }
   
   // ✅ Struct tools: Full state control
   struct WeatherTool {
       api_key: String,
       cache: Cache,
       rate_limiter: RateLimiter,
   }
   ```

2. **Complex Parameter Validation**
   ```rust
   // ❌ Macro tools: Limited to JSON schema
   #[tool]
   async fn transfer_money(from: String, to: String, amount: f64) -> Result<(), String> {
       // Can't easily validate account existence, balance, etc.
   }
   
   // ✅ Struct tools: Custom validation
   impl Tool for TransferTool {
       async fn execute(&self, params: Option<Value>) -> Result<ToolResult, ToolError> {
           let transfer = self.parse_and_validate_transfer(params).await?;
           self.validate_business_rules(&transfer).await?;
           self.execute_transfer(transfer).await
       }
   }
   ```

3. **Dynamic Schema Generation**
   ```rust
   // ❌ Macro tools: Fixed schema at compile time
   #[tool]
   async fn query_database(table: String) -> Result<Vec<Row>, String> {
       // Schema is always the same
   }
   
   // ✅ Struct tools: Runtime schema adaptation
   impl Tool for DatabaseTool {
       fn parameters_schema(&self) -> Value {
           // Generate schema based on connected database
           self.introspect_and_generate_schema()
       }
   }
   ```

4. **Resource Management**
   ```rust
   // ❌ Macro tools: No lifecycle management
   #[tool]
   async fn process_file(path: String) -> Result<String, String> {
       // No cleanup, connection pooling, etc.
   }
   
   // ✅ Struct tools: Full resource control
   impl Tool for FileProcessorTool {
       async fn execute(&self, params: Option<Value>) -> Result<ToolResult, ToolError> {
           let _guard = self.acquire_resource().await?;
           let result = self.process_with_cleanup(params).await;
           // Cleanup happens automatically
           result
       }
   }
   ```

## Performance Characteristics

Both approaches have **identical runtime performance** - they compile to the same underlying trait implementations.

### Compile Time
- **Macro tools**: Slight overhead during compilation due to code generation
- **Struct tools**: Standard compilation speed

### Memory Usage
- **Macro tools**: Zero-sized structs, minimal memory footprint
- **Struct tools**: Memory usage depends on stored state

### Binary Size
- **Both approaches**: Identical binary size after compilation

## Best Practices

### Choosing the Right Approach

**Start with macros**, migrate to structs as complexity grows:

```rust
// Development progression:

// 1. Prototype with macro
# [tool]
async fn calculate_shipping(weight: f64) -> Result<f64, String> {
    Ok(weight * 0.1) // Simple calculation
}

// 2. Add complexity, keep macro
# [tool]
async fn calculate_shipping(weight: f64, distance: f64, method: String) -> Result<f64, String> {
    // More complex logic, but still stateless
}

// 3. Need state/config, migrate to struct
struct ShippingCalculator {
    rate_tables: HashMap<String, RateTable>,
    distance_cache: Cache,
    api_client: Client,
}
```

### Migration Path

Converting from macro to struct is straightforward:

```rust
// Original macro tool
# [tool]
async fn my_tool(param: String) -> Result<String, String> {
    // Implementation
}

// Equivalent struct tool
# [derive(Debug)]
struct MyTool;

impl MyTool {
    pub fn new() -> Self { Self }
}

# [async_trait]
impl Tool for MyTool {
    fn name(&self) -> &str { "my_tool" }
    fn description(&self) -> &str { "Auto-generated tool" }
    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "param": { "type": "string", "description": "Parameter" }
            },
            "required": ["param"]
        })
    }
    
    async fn execute(&self, parameters: Option<Value>) -> Result<ToolResult, ToolError> {
        // Parameter extraction (can be simplified with helper functions)
        let param = extract_required_param::<String>(parameters, "param")?;
        
        // Call original implementation
        match my_tool_impl(param).await {
            Ok(result) => Ok(ToolResult::success(json!(result))),
            Err(e) => Ok(ToolResult::error(e)),
        }
    }
}

async fn my_tool_impl(param: String) -> Result<String, String> {
    // Original implementation moved here
}
```

### Code Organization

Structure your tools for maintainability:

```rust
// tools/mod.rs
pub mod calculations;  // Macro tools for simple math
pub mod integrations;  // Struct tools for external APIs
pub mod workflows;     // Hybrid tools for complex processes

// tools/calculations.rs - Simple macro tools
# [tool]
async fn add(a: f64, b: f64) -> Result<f64, String> { ... }

# [tool]
async fn percentage(value: f64, percent: f64) -> Result<f64, String> { ... }

// tools/integrations.rs - Complex struct tools
pub struct DatabaseTool { ... }
pub struct PaymentTool { ... }
pub struct EmailTool { ... }

// tools/workflows.rs - Hybrid approach
# [tool]
async fn quick_calc(expr: String) -> Result<f64, String> { ... }

pub struct WorkflowEngine { ... } // Orchestrates other tools
```

## Testing Strategies

### Testing Macro Tools

```rust
# [cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_calculate_percentage() {
        // Test the generated function directly
        let result = calculate_percentage_impl(100.0, 15.0).await.unwrap();
        assert_eq!(result, 15.0);
    }
    
    #[tokio::test]
    async fn test_tool_integration() {
        // Test the tool through the Tool trait
        let tool = calculate_percentage();
        let result = tool.execute(Some(json!({
            "value": 100.0,
            "percentage": 15.0
        }))).await.unwrap();
        
        assert!(result.success);
        assert_eq!(result.content, 15.0);
    }
}
```

### Testing Struct Tools

```rust
# [cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_weather_tool_success() {
        let tool = WeatherTool::new_mock();
        
        let result = tool.execute(Some(json!({
            "location": "London"
        }))).await.unwrap();
        
        assert!(result.success);
        // More detailed assertions on structure
    }
    
    #[tokio::test] 
    async fn test_weather_tool_caching() {
        let tool = WeatherTool::new_mock();
        
        // First call
        let start = Instant::now();
        tool.execute(Some(json!({"location": "London"}))).await.unwrap();
        let first_duration = start.elapsed();
        
        // Second call (should be cached)
        let start = Instant::now();
        tool.execute(Some(json!({"location": "London"}))).await.unwrap();
        let second_duration = start.elapsed();
        
        assert!(second_duration < first_duration / 2);
    }
    
    #[test]
    fn test_internal_logic() {
        // Test internal methods directly
        let tool = WeatherTool::new_mock();
        let params = tool.parse_location("New York, NY").unwrap();
        assert_eq!(params.city, "New York");
        assert_eq!(params.state, Some("NY"));
    }
}
```

## Error Handling Patterns

### Macro Tool Errors

```rust
# [tool]
async fn divide_numbers(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err("Division by zero is not allowed".to_string())
    } else {
        Ok(a / b)
    }
}
```

### Struct Tool Errors

```rust
impl Tool for DatabaseTool {
    async fn execute(&self, parameters: Option<Value>) -> Result<ToolResult, ToolError> {
        match self.execute_query(parameters).await {
            Ok(result) => Ok(ToolResult::success(result)),
            Err(QueryError::ConnectionFailed(e)) => {
                Err(ToolError::ExecutionFailed {
                    message: format!("Database connection failed: {}", e)
                })
            },
            Err(QueryError::InvalidSql(e)) => {
                Err(ToolError::InvalidParameters {
                    message: format!("Invalid SQL query: {}", e)
                })
            },
            Err(QueryError::AccessDenied(table)) => {
                Ok(ToolResult::error(format!("Access denied to table: {}", table)))
            },
        }
    }
}
```

## Conclusion

Both macro and struct approaches are first-class citizens in the Stood ecosystem. The choice depends on your specific requirements:

- **Macro tools** excel at rapid development and simple operations
- **Struct tools** provide complete control for production requirements
- **Hybrid approach** leverages the best of both worlds

The unified `Agent::builder()` pattern makes it seamless to mix and match approaches as your application evolves.

Remember: Start simple with macros, add complexity with structs as needed. Both compile to identical runtime performance, so the choice is purely about development experience and feature requirements.
