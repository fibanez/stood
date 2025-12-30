//! Procedural macros for the Stood agent library
//!
//! This crate provides the `#[tool]` procedural macro that automatically generates
//! tool implementations from Rust functions.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, punctuated::Punctuated, Expr, ExprLit, FnArg, ItemFn, Lit, Meta, Pat,
    PatType, Token, Type, Visibility,
};

/// The `#[tool]` procedural macro
///
/// Transforms a regular Rust function into a tool that can be used by agents.
///
/// # Example
///
/// ```rust
/// use stood_macros::tool;
///
/// #[tool]
/// /// Calculate the sum of two numbers
/// async fn add(
///     /// First number
///     a: f64,
///     /// Second number
///     b: f64
/// ) -> Result<f64, String> {
///     Ok(a + b)
/// }
/// ```
///
/// This generates:
/// - A tool struct that implements the `Tool` trait
/// - Automatic JSON schema generation from function parameters
/// - Parameter validation and type conversion
/// - Error handling and result serialization
#[proc_macro_attribute]
pub fn tool(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args with Punctuated::<Meta, Token![,]>::parse_terminated);
    let input_fn = parse_macro_input!(input as ItemFn);

    match generate_tool_impl(args, input_fn) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// Generate the tool implementation
fn generate_tool_impl(
    args: Punctuated<Meta, Token![,]>,
    input_fn: ItemFn,
) -> syn::Result<TokenStream2> {
    let tool_config = parse_tool_args(&args)?;
    let fn_info = parse_function(&input_fn)?;

    let renamed_fn = generate_renamed_function(&input_fn)?;
    let tool_struct = generate_tool_struct(&fn_info, &tool_config)?;
    let tool_impl = generate_tool_trait_impl(&fn_info, &tool_config)?;
    let function_constructor = generate_function_constructor(&fn_info)?;

    Ok(quote! {
        #renamed_fn

        #tool_struct

        #tool_impl

        #function_constructor
    })
}

/// Configuration for the tool macro
#[derive(Debug, Default)]
struct ToolConfig {
    name: Option<String>,
    description: Option<String>,
}

/// Information extracted from the function
#[derive(Debug)]
struct FunctionInfo {
    name: String,
    vis: Visibility,
    is_async: bool,
    inputs: Vec<FunctionInput>,
    doc_comment: Option<String>,
}

/// Information about a function input parameter
#[derive(Debug)]
struct FunctionInput {
    name: String,
    ty: Type,
    doc_comment: Option<String>,
    is_optional: bool,
}

/// Parse tool macro arguments
fn parse_tool_args(args: &Punctuated<Meta, Token![,]>) -> syn::Result<ToolConfig> {
    let mut config = ToolConfig::default();

    for arg in args {
        match arg {
            Meta::NameValue(nv) if nv.path.is_ident("name") => {
                if let Expr::Lit(ExprLit {
                    lit: Lit::Str(lit_str),
                    ..
                }) = &nv.value
                {
                    config.name = Some(lit_str.value());
                }
            }
            Meta::NameValue(nv) if nv.path.is_ident("description") => {
                if let Expr::Lit(ExprLit {
                    lit: Lit::Str(lit_str),
                    ..
                }) = &nv.value
                {
                    config.description = Some(lit_str.value());
                }
            }
            _ => {
                return Err(syn::Error::new_spanned(
                    arg,
                    "Unsupported tool attribute. Use `name` or `description`",
                ));
            }
        }
    }

    Ok(config)
}

/// Parse function information
fn parse_function(input_fn: &ItemFn) -> syn::Result<FunctionInfo> {
    let name = input_fn.sig.ident.to_string();
    let vis = input_fn.vis.clone();
    let is_async = input_fn.sig.asyncness.is_some();

    // Extract documentation comment
    let doc_comment = extract_doc_comment(&input_fn.attrs);

    // Parse inputs
    let mut inputs = Vec::new();
    for input in &input_fn.sig.inputs {
        match input {
            FnArg::Typed(PatType { pat, ty, attrs, .. }) => {
                if let Pat::Ident(pat_ident) = pat.as_ref() {
                    let param_name = pat_ident.ident.to_string();
                    let param_doc = extract_doc_comment(attrs);
                    let is_optional = is_option_type(ty);

                    inputs.push(FunctionInput {
                        name: param_name,
                        ty: (**ty).clone(),
                        doc_comment: param_doc,
                        is_optional,
                    });
                }
            }
            FnArg::Receiver(_) => {
                return Err(syn::Error::new_spanned(
                    input,
                    "Tool functions cannot have self parameters",
                ));
            }
        }
    }

    Ok(FunctionInfo {
        name,
        vis,
        is_async,
        inputs,
        doc_comment,
    })
}

/// Extract documentation comments from attributes
fn extract_doc_comment(attrs: &[syn::Attribute]) -> Option<String> {
    let mut doc_lines = Vec::new();

    for attr in attrs {
        if attr.path().is_ident("doc") {
            if let Meta::NameValue(nv) = &attr.meta {
                if let Expr::Lit(ExprLit {
                    lit: Lit::Str(lit_str),
                    ..
                }) = &nv.value
                {
                    let line = lit_str.value();
                    // Remove leading space if present
                    let trimmed = if let Some(stripped) = line.strip_prefix(' ') {
                        stripped
                    } else {
                        &line
                    };
                    doc_lines.push(trimmed.to_string());
                }
            }
        }
    }

    if doc_lines.is_empty() {
        None
    } else {
        Some(doc_lines.join("\n"))
    }
}

/// Check if a type is Option<T>
fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Option";
        }
    }
    false
}

/// Generate the renamed function (original function with `_impl` suffix)
fn generate_renamed_function(input_fn: &ItemFn) -> syn::Result<TokenStream2> {
    let mut renamed_fn = input_fn.clone();
    let original_name = &input_fn.sig.ident;
    let impl_name = format_ident!("{}_impl", original_name);
    renamed_fn.sig.ident = impl_name;

    Ok(quote! { #renamed_fn })
}

/// Generate the function constructor that returns Box<dyn Tool>
fn generate_function_constructor(fn_info: &FunctionInfo) -> syn::Result<TokenStream2> {
    let fn_name = format_ident!("{}", fn_info.name);
    let struct_name = format_ident!("{}Tool", snake_to_pascal(&fn_info.name));
    let vis = &fn_info.vis;

    Ok(quote! {
        /// Create a new instance of this tool for use with agents
        #vis fn #fn_name() -> Box<dyn stood::tools::Tool> {
            Box::new(#struct_name)
        }
    })
}

/// Generate the tool struct (now private since accessed through function)
fn generate_tool_struct(
    fn_info: &FunctionInfo,
    _tool_config: &ToolConfig,
) -> syn::Result<TokenStream2> {
    let struct_name = format_ident!("{}Tool", snake_to_pascal(&fn_info.name));

    Ok(quote! {
        #[derive(Debug, Clone)]
        struct #struct_name;

        impl #struct_name {
            fn new() -> Self {
                Self
            }
        }
    })
}

/// Generate the Tool trait implementation
fn generate_tool_trait_impl(
    fn_info: &FunctionInfo,
    tool_config: &ToolConfig,
) -> syn::Result<TokenStream2> {
    let struct_name = format_ident!("{}Tool", snake_to_pascal(&fn_info.name));
    let fn_impl_name = format_ident!("{}_impl", fn_info.name);

    // Determine tool name and description
    let tool_name = tool_config.name.as_ref().unwrap_or(&fn_info.name);
    let tool_description = tool_config
        .description
        .as_ref()
        .or(fn_info.doc_comment.as_ref())
        .map(|s| s.as_str())
        .unwrap_or("Auto-generated tool");

    // Generate JSON schema for inputs
    let schema = generate_input_schema(&fn_info.inputs)?;

    // Generate parameter extraction code
    let param_extractions = generate_parameter_extraction(&fn_info.inputs)?;

    // Generate parameter names for function call
    let param_names: Vec<_> = fn_info
        .inputs
        .iter()
        .map(|input| format_ident!("{}", input.name))
        .collect();

    // Generate function call to the renamed implementation
    let fn_call = if fn_info.is_async {
        quote! { #fn_impl_name(#(#param_names),*).await }
    } else {
        quote! { #fn_impl_name(#(#param_names),*) }
    };

    Ok(quote! {
        #[async_trait::async_trait]
        impl stood::tools::Tool for #struct_name {
            fn name(&self) -> &str {
                #tool_name
            }

            fn description(&self) -> &str {
                #tool_description
            }

            fn parameters_schema(&self) -> serde_json::Value {
                #schema
            }

            async fn execute(&self, parameters: Option<serde_json::Value>, _agent_context: Option<&stood::agent::AgentContext>) -> Result<stood::tools::ToolResult, stood::tools::ToolError> {
                use serde_json::Value;

                // Get parameters or default to empty object
                let input = parameters.unwrap_or_else(|| serde_json::json!({}));

                // Validate input is an object
                let input_obj = input.as_object()
                    .ok_or_else(|| stood::tools::ToolError::InvalidParameters {
                        message: "Tool input must be a JSON object".to_string()
                    })?;

                // Extract and validate parameters
                #(#param_extractions)*

                // Call the renamed implementation function
                let result = #fn_call;

                // Handle the result
                match result {
                    Ok(value) => {
                        // Serialize the result to JSON
                        let json_value = serde_json::to_value(value)
                            .map_err(|e| stood::tools::ToolError::ExecutionFailed {
                                message: format!("Failed to serialize result: {}", e)
                            })?;
                        Ok(stood::tools::ToolResult::success(json_value))
                    }
                    Err(e) => {
                        // Return error result
                        Ok(stood::tools::ToolResult::error(e.to_string()))
                    }
                }
            }
        }
    })
}

/// Generate JSON schema for function inputs
fn generate_input_schema(inputs: &[FunctionInput]) -> syn::Result<TokenStream2> {
    let mut properties = Vec::new();
    let mut required = Vec::new();

    for input in inputs {
        let name = &input.name;
        let description = input.doc_comment.as_deref().unwrap_or("Parameter");

        // Determine JSON schema type from Rust type
        let json_type = rust_type_to_json_schema(&input.ty)?;

        properties.push(quote! {
            #name: serde_json::json!({
                "type": #json_type,
                "description": #description
            })
        });

        if !input.is_optional {
            required.push(name);
        }
    }

    Ok(quote! {
        serde_json::json!({
            "type": "object",
            "properties": {
                #(#properties,)*
            },
            "required": [#(#required,)*]
        })
    })
}

/// Convert Rust type to JSON schema type string
fn rust_type_to_json_schema(ty: &Type) -> syn::Result<String> {
    match ty {
        Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last() {
                let type_name = segment.ident.to_string();
                match type_name.as_str() {
                    "String" | "str" => Ok("string".to_string()),
                    "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32"
                    | "u64" | "u128" | "usize" => Ok("integer".to_string()),
                    "f32" | "f64" => Ok("number".to_string()),
                    "bool" => Ok("boolean".to_string()),
                    "Vec" => Ok("array".to_string()),
                    "HashMap" | "BTreeMap" => Ok("object".to_string()),
                    "Option" => {
                        // For Option<T>, get the inner type
                        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                            if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                                return rust_type_to_json_schema(inner_ty);
                            }
                        }
                        Ok("string".to_string()) // Default fallback
                    }
                    _ => Ok("string".to_string()), // Default fallback for unknown types
                }
            } else {
                Ok("string".to_string())
            }
        }
        _ => Ok("string".to_string()), // Default fallback
    }
}

/// Generate parameter extraction code
fn generate_parameter_extraction(inputs: &[FunctionInput]) -> syn::Result<Vec<TokenStream2>> {
    let mut extractions = Vec::new();

    for input in inputs {
        let param_name = format_ident!("{}", input.name);
        let param_str = &input.name;

        let extraction = if input.is_optional {
            quote! {
                let #param_name = input_obj.get(#param_str)
                    .and_then(|v| serde_json::from_value(v.clone()).ok());
            }
        } else {
            quote! {
                let #param_name = input_obj.get(#param_str)
                    .ok_or_else(|| stood::tools::ToolError::InvalidParameters {
                        message: format!("Missing required parameter: {}", #param_str)
                    })?;
                let #param_name = serde_json::from_value(#param_name.clone())
                    .map_err(|e| stood::tools::ToolError::InvalidParameters {
                        message: format!("Invalid parameter {}: {}", #param_str, e)
                    })?;
            }
        };

        extractions.push(extraction);
    }

    Ok(extractions)
}

/// Capitalize the first character of a string
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

/// Convert snake_case to PascalCase
fn snake_to_pascal(s: &str) -> String {
    s.split('_').map(capitalize_first).collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_capitalize_first() {
        assert_eq!(capitalize_first("hello"), "Hello");
        assert_eq!(capitalize_first("world"), "World");
        assert_eq!(capitalize_first(""), "");
        assert_eq!(capitalize_first("a"), "A");
    }

    #[test]
    fn test_rust_type_to_json_schema() {
        let string_type: Type = parse_quote!(String);
        assert_eq!(rust_type_to_json_schema(&string_type).unwrap(), "string");

        let int_type: Type = parse_quote!(i32);
        assert_eq!(rust_type_to_json_schema(&int_type).unwrap(), "integer");

        let float_type: Type = parse_quote!(f64);
        assert_eq!(rust_type_to_json_schema(&float_type).unwrap(), "number");

        let bool_type: Type = parse_quote!(bool);
        assert_eq!(rust_type_to_json_schema(&bool_type).unwrap(), "boolean");
    }

    #[test]
    fn test_is_option_type() {
        let option_type: Type = parse_quote!(Option<String>);
        assert!(is_option_type(&option_type));

        let string_type: Type = parse_quote!(String);
        assert!(!is_option_type(&string_type));
    }

    #[test]
    fn test_parse_tool_args_empty() {
        let args: Punctuated<Meta, Token![,]> = Punctuated::new();
        let config = parse_tool_args(&args).unwrap();
        assert!(config.name.is_none());
        assert!(config.description.is_none());
    }
}
