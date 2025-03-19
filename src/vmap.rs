// vmap.rs
//
// This file provides macros for working with ValueType maps and extracting values
// in a type-safe and ergonomic way.

/// Create a HashMap with ValueType values
/// 
/// This macro allows for easy creation of parameter maps for service requests.
/// 
/// # Examples
/// 
/// ```
/// // Basic usage with key-value pairs
/// let params = vmap! {
///     "name" => "John Doe",
///     "age" => 30,
///     "is_active" => true
/// };
///
/// // Extract a value from a map with default
/// let name = vmap!(response.data, "name" => "Unknown");
/// ```
#[macro_export]
macro_rules! vmap {
    // Empty map
    {} => {
        {
            let map: std::collections::HashMap<String, runar_common::types::ValueType> = std::collections::HashMap::new();
            runar_common::types::ValueType::Map(map)
        }
    };
    
    // Map with key-value pairs
    {
        $($key:expr => $value:expr),* $(,)?
    } => {
        {
            let mut map = std::collections::HashMap::new();
            $(
                map.insert($key.to_string(), runar_common::types::ValueType::from($value));
            )*
            runar_common::types::ValueType::Map(map)
        }
    };
    
    // Extract a value from a map with default
    ($map:expr, $key:expr => $default:expr) => {
        match &$map {
            runar_common::types::ValueType::Map(map) => {
                match map.get(&$key.to_string()) {
                    Some(value) => {
                        match value {
                            runar_common::types::ValueType::String(s) => {
                                if std::any::type_name_of_val(&$default) == "std::string::String"
                                    || std::any::type_name_of_val(&$default) == "&str" {
                                    s.clone()
                                } else {
                                    $default
                                }
                            },
                            runar_common::types::ValueType::Number(n) => {
                                let default_val = $default;
                                match std::any::type_name_of_val(&default_val) {
                                    "i32" | "i64" => (*n as i64) as _,
                                    "u32" | "u64" => (*n as u64) as _,
                                    "f32" | "f64" => *n as _,
                                    _ => $default,
                                }
                            },
                            runar_common::types::ValueType::Bool(b) => {
                                if std::any::type_name_of_val(&$default) == "bool" {
                                    *b
                                } else {
                                    $default
                                }
                            },
                            runar_common::types::ValueType::Array(a) => {
                                if let Ok(converted) = serde_json::to_value(a) {
                                    if let Ok(typed) = serde_json::from_value(converted) {
                                        typed
                                    } else {
                                        $default
                                    }
                                } else {
                                    $default
                                }
                            },
                            runar_common::types::ValueType::Map(inner_map) => {
                                if let Ok(converted) = serde_json::to_value(inner_map) {
                                    if let Ok(typed) = serde_json::from_value(converted) {
                                        typed
                                    } else {
                                        $default
                                    }
                                } else {
                                    $default
                                }
                            },
                            _ => $default,
                        }
                    },
                    None => $default,
                }
            },
            _ => $default,
        }
    };
    
    // Extract a direct value with default
    ($value:expr, => $default:expr) => {
        match $value {
            runar_common::types::ValueType::String(s) => s.clone(),
            runar_common::types::ValueType::Number(n) => {
                let default_val = $default;
                match std::any::type_name_of_val(&default_val) {
                    "i32" | "i64" => (*n as i64) as _,
                    "u32" | "u64" => (*n as u64) as _,
                    "f32" | "f64" => *n as _,
                    _ => $default,
                }
            },
            runar_common::types::ValueType::Bool(b) => {
                if std::any::type_name_of_val(&$default) == "bool" {
                    *b
                } else {
                    $default
                }
            },
            runar_common::types::ValueType::Array(a) => {
                if let Ok(converted) = serde_json::to_value(a) {
                    if let Ok(typed) = serde_json::from_value(converted) {
                        typed
                    } else {
                        $default
                    }
                } else {
                    $default
                }
            },
            runar_common::types::ValueType::Map(m) => {
                if let Ok(converted) = serde_json::to_value(m) {
                    if let Ok(typed) = serde_json::from_value(converted) {
                        typed
                    } else {
                        $default
                    }
                } else {
                    $default
                }
            },
            _ => $default,
        }
    };
}

/// Create an optional HashMap with ValueType values
/// 
/// This macro is similar to `vmap!` but wraps the result in Some().
/// 
/// # Examples
/// 
/// ```
/// let optional_params = vmap_opt! {
///     "name" => "John Doe",
///     "age" => 30
/// };
/// ```
#[macro_export]
macro_rules! vmap_opt {
    // Empty optional map
    {} => {
        Some(vmap!{})
    };
    
    // Optional map with key-value pairs
    {
        $($key:expr => $value:expr),* $(,)?
    } => {
        Some(vmap!{$($key => $value),*})
    };
}

/// Extract values from ValueType with defaults
/// 
/// This macro allows extracting values from a ValueType with default values
/// if the key is not found or the value has the wrong type.
/// 
/// # Examples
/// 
/// ```
/// let name = vmap_extract!(response.data, "name" => "Unknown");
/// let age = vmap_extract!(response.data, "age" => 0);
/// let items = vmap_extract!(response.data, "items" => Vec::<String>::new());
/// 
/// // Extract a direct value (not a key)
/// let value = vmap_extract!(response.data, => "default");
/// ```
#[macro_export]
macro_rules! vmap_extract {
    // Extract a direct value with default
    ($value:expr, => $default:expr) => {
        match $value {
            Some(ref v) => {
                // Get the value directly
                match v {
                    runar_common::types::ValueType::String(s) => s.clone(),
                    runar_common::types::ValueType::Number(n) => {
                        let default_val = $default;
                        match std::any::type_name_of_val(&default_val) {
                            "i32" | "i64" => (*n as i64) as _,
                            "u32" | "u64" => (*n as u64) as _,
                            "f32" | "f64" => *n as _,
                            _ => $default,
                        }
                    },
                    runar_common::types::ValueType::Bool(b) => {
                        if std::any::type_name_of_val(&$default) == "bool" {
                            *b
                        } else {
                            $default
                        }
                    },
                    runar_common::types::ValueType::Array(a) => {
                        if let Ok(converted) = serde_json::to_value(a) {
                            if let Ok(typed) = serde_json::from_value(converted) {
                                typed
                            } else {
                                $default
                            }
                        } else {
                            $default
                        }
                    },
                    runar_common::types::ValueType::Map(m) => {
                        if let Ok(converted) = serde_json::to_value(m) {
                            if let Ok(typed) = serde_json::from_value(converted) {
                                typed
                            } else {
                                $default
                            }
                        } else {
                            $default
                        }
                    },
                    _ => $default,
                }
            },
            None => $default,
        }
    };
    
    // Extract a value from a map with default
    ($value:expr, $key:expr => $default:expr) => {
        match $value {
            Some(ref v) => {
                match v {
                    runar_common::types::ValueType::Map(map) => {
                        match map.get(&$key.to_string()) {
                            Some(value) => {
                                match value {
                                    runar_common::types::ValueType::String(s) => {
                                        if std::any::type_name_of_val(&$default) == "std::string::String"
                                            || std::any::type_name_of_val(&$default) == "&str" {
                                            s.clone()
                                        } else {
                                            $default
                                        }
                                    },
                                    runar_common::types::ValueType::Number(n) => {
                                        let default_val = $default;
                                        match std::any::type_name_of_val(&default_val) {
                                            "i32" | "i64" => (*n as i64) as _,
                                            "u32" | "u64" => (*n as u64) as _,
                                            "f32" | "f64" => *n as _,
                                            _ => $default,
                                        }
                                    },
                                    runar_common::types::ValueType::Bool(b) => {
                                        if std::any::type_name_of_val(&$default) == "bool" {
                                            *b
                                        } else {
                                            $default
                                        }
                                    },
                                    runar_common::types::ValueType::Array(a) => {
                                        if let Ok(converted) = serde_json::to_value(a) {
                                            if let Ok(typed) = serde_json::from_value(converted) {
                                                typed
                                            } else {
                                                $default
                                            }
                                        } else {
                                            $default
                                        }
                                    },
                                    runar_common::types::ValueType::Map(inner_map) => {
                                        if let Ok(converted) = serde_json::to_value(inner_map) {
                                            if let Ok(typed) = serde_json::from_value(converted) {
                                                typed
                                            } else {
                                                $default
                                            }
                                        } else {
                                            $default
                                        }
                                    },
                                    _ => $default,
                                }
                            },
                            None => $default,
                        }
                    },
                    _ => $default,
                }
            },
            None => $default,
        }
    };
} 