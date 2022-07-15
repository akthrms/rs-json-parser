# Json Parser

```rust
let json = Json::parse(r#"{ "name": "Tanaka", "age": 26 }"#).unwrap();

assert_eq!(
    json,
    Json::Object(
        vec![
            ("name".to_string(), Json::String("Tanaka".to_string())),
            ("age".to_string(), Json::Number(26.0))
        ]
        .into_iter()
        .collect::<HashMap<String, Json>>()
    )
);
```
