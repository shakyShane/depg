use std::collections::{HashMap, HashSet};

#[derive(Debug, serde::Deserialize)]
pub struct TsConfig {
    #[serde(rename = "compilerOptions")]
    pub compiler_options: Option<CompilerOptions>,
}

#[derive(Debug, serde::Deserialize)]
pub struct CompilerOptions {
    pub paths: Option<HashMap<String, HashSet<String>>>,
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_read_ts_config() {
        let input = r#"
            {
              "compilerOptions": {
                "jsx": "react",
                "baseUrl": ".",
                "paths": {
                  "~/src/*": ["./src/*"],
                  "app-src/*": ["./app-src/*"]
                }
              }
            }
        "#;
        let ts_config: TsConfig = serde_json::from_str(input).unwrap();
        dbg!(ts_config.compiler_options.unwrap().paths.unwrap());
    }
}
