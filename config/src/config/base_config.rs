use colored::*;
use serde::{Deserialize, Serialize};
use std::{env, path::PathBuf};
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub enum EnvironmentKind {
    Development,
    Production,
    Testing,
}
impl AsRef<str> for EnvironmentKind {
    fn as_ref(&self) -> &str {
        match self {
            EnvironmentKind::Development => "Development",
            EnvironmentKind::Production => "Production",
            EnvironmentKind::Testing => "Testing",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BaseConfig {
    pub root_path: PathBuf,
    pub env: EnvironmentKind,
}

impl Default for BaseConfig {
    fn default() -> Self {
        let env = if cfg!(test) {
            EnvironmentKind::Testing
        } else {
            env::var("ENV").map_or_else(
                |_| {
                    println!(
                        "{} Environment variable `ENV` not found, defaulting to {}",
                        "Config Info:".green(),
                        "`Testing`".blue()
                    );
                    EnvironmentKind::Testing
                },
                |e| match e.to_lowercase().as_str() {
                    "prod" => EnvironmentKind::Production,
                    "dev" => EnvironmentKind::Development,
                    _ => {
                        println!(
                            "{} Invalid `ENV` value: {}, defaulting to {}",
                            "Config Warning:".yellow(),
                            e.red(),
                            "`Testing`".blue()
                        );
                        EnvironmentKind::Testing
                    }
                },
            )
        };
        let path = match env {
            EnvironmentKind::Production => dirs::home_dir()
                .unwrap_or_else(|| {
                    panic!("Home directory not found");
                })
                .join(".space"),
            EnvironmentKind::Development => dirs::home_dir()
                .unwrap_or_else(|| {
                    panic!("Home directory not found");
                })
                .join(".space-dev"),
            EnvironmentKind::Testing => PathBuf::from("../.space-test"),
        };
        let absolute_path = env::current_dir()
            .unwrap()
            .join(path.clone())
            .canonicalize()
            .unwrap();
        println!(
            "{} Environment and the Root Path is {}",
            env.as_ref().blue(),
            absolute_path.to_string_lossy().blue()
        );
        Self {
            root_path: path,
            env: env,
        }
    }
}
