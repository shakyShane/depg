use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use swc_ecma_parser::TsConfig;

#[derive(Debug, Default, serde::Deserialize)]
pub struct UserTsConfig {
    #[serde(rename = "compilerOptions")]
    pub compiler_options: Option<CompilerOptions>,
}

#[derive(Debug, serde::Deserialize)]
pub struct CompilerOptions {
    pub paths: Option<HashMap<String, HashSet<String>>>,
}

impl UserTsConfig {
    pub fn paths(&self) -> HashMap<String, HashSet<String>> {
        if let Some(CompilerOptions { paths: Some(hm) }) = self.compiler_options.as_ref() {
            return hm.clone();
        }
        HashMap::new()
    }
    pub fn from_file(cwd: &PathBuf) -> Self {
        let as_str = std::fs::read_to_string(cwd.join("tsconfig.json"));
        if let Err(e) = as_str {
            eprintln!("{}", e);
            return Self::default();
        }
        match serde_json::from_str::<UserTsConfig>(&as_str.expect("guarded")) {
            Ok(ts_config) => {
                log::debug!("Read ts_config");
                log::debug!("ts_config paths: {:?}", ts_config.paths());
                log::trace!("{:#?}", ts_config);
                ts_config
            }
            Err(e) => {
                eprintln!("{:?}", e);
                Self::default()
            }
        }
    }
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
        let ts_config: UserTsConfig = serde_json::from_str(input).unwrap();
        dbg!(ts_config.compiler_options.unwrap().paths.unwrap());
    }
}
