use serde_json::json;

#[tokio::test]
async fn test_lm_studio_quote_handling() {
    // Mock LM Studio response with escaped quotes in arguments
    let mock_response = json!({
        "choices": [
            {
                "finish_reason": "tool_calls",
                "index": 0,
                "message": {
                    "role": "assistant",
                    "tool_calls": [
                        {
                            "function": {
                                "arguments": "{\"location\":\"San Francisco\"}",
                                "name": "get_weather"
                            },
                            "id": "232727927",
                            "type": "function"
                        }
                    ]
                }
            }
        ],
        "created": 1751929044,
        "id": "chatcmpl-test",
        "model": "google/gemma-3-12b",
        "object": "chat.completion",
        "usage": {
            "completion_tokens": 103,
            "prompt_tokens": 771,
            "total_tokens": 874
        }
    });

    println!("🧪 Testing LM Studio quote handling");
    println!("Mock response:");
    println!("{}", serde_json::to_string_pretty(&mock_response).unwrap());

    // Test the fix by manually calling the conversion function logic
    if let Some(choices) = mock_response.get("choices").and_then(|c| c.as_array()) {
        if let Some(choice) = choices.first() {
            if let Some(message) = choice.get("message") {
                if let Some(tool_calls) = message.get("tool_calls").and_then(|tc| tc.as_array()) {
                    for call in tool_calls {
                        if let Some(function) = call.get("function") {
                            if let Some(name) = function.get("name").and_then(|n| n.as_str()) {
                                // Test our new logic
                                let parsed_args: serde_json::Value = match function.get("arguments")
                                {
                                    Some(serde_json::Value::String(s)) => {
                                        println!("📄 Raw arguments string: {}", s);
                                        match serde_json::from_str(s) {
                                            Ok(parsed) => {
                                                println!(
                                                    "✅ Successfully parsed arguments: {}",
                                                    serde_json::to_string_pretty(&parsed).unwrap()
                                                );
                                                parsed
                                            }
                                            Err(e) => {
                                                println!("❌ Failed to parse arguments: {}", e);
                                                serde_json::Value::String(s.clone())
                                            }
                                        }
                                    }
                                    Some(obj) => {
                                        println!(
                                            "📦 Arguments already parsed as object: {}",
                                            serde_json::to_string_pretty(obj).unwrap()
                                        );
                                        obj.clone()
                                    }
                                    None => {
                                        println!("🚫 No arguments found");
                                        serde_json::Value::Object(serde_json::Map::new())
                                    }
                                };

                                println!("🔧 Tool: {}", name);
                                println!(
                                    "📝 Final parsed arguments: {}",
                                    serde_json::to_string_pretty(&parsed_args).unwrap()
                                );

                                // Verify the arguments are now a proper JSON object
                                if let serde_json::Value::Object(map) = &parsed_args {
                                    if let Some(location) = map.get("location") {
                                        if let serde_json::Value::String(loc_str) = location {
                                            assert_eq!(loc_str, "San Francisco");
                                            println!(
                                                "✅ Successfully extracted location: {}",
                                                loc_str
                                            );
                                        } else {
                                            panic!("❌ Location is not a string: {:?}", location);
                                        }
                                    } else {
                                        panic!("❌ No location field found in arguments");
                                    }
                                } else {
                                    panic!("❌ Arguments are not a JSON object: {:?}", parsed_args);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    println!("✅ LM Studio quote handling test passed!");
}
