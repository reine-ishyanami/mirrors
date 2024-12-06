use crate::command::ProcessArg;
use crate::utils::file_utils::{read_config, write_config};
use anyhow::Result;
use clap::arg;
use process_arg_derive::ProcessArg;
use serde::{Deserialize, Serialize};
use std::env;
use std::str::FromStr;
use std::{path::PathBuf, sync::LazyLock};

use super::{MirrorConfigurate, Reader};

const ENV_NAME: &str = "GRADLE_USER_HOME";

static DEFAULT_GRADLE_PROFILES: LazyLock<Vec<PathBuf>> = LazyLock::new(|| {
    let profile_path = match env::var(ENV_NAME) {
        Ok(value) => PathBuf::from(value),
        Err(_) => dirs::home_dir().unwrap().join(".gradle"),
    };
    vec![profile_path.join("init.gradle.kts")]
});

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct GradleMirror {
    maven: String,
    android: String,
    plugins: String,
}

impl GradleMirror {
    pub fn new(maven: String, android: String, plugins: String) -> Self {
        Self {
            maven,
            android,
            plugins,
        }
    }
}

impl Reader for GradleMirror {
    fn new_config(&self) -> Result<String> {
        Ok(format!(
            include_str!("../../../templates/init.gradle.kts"),
            if self.maven.is_empty() {
                "https://repo.maven.apache.org/maven2".into()
            } else {
                self.maven.clone()
            },
            if self.android.is_empty() {
                "https://dl.google.com/dl/android/maven2".into()
            } else {
                self.android.clone()
            },
            if self.plugins.is_empty() {
                "https://plugins.gradle.org/m2".into()
            } else {
                self.plugins.clone()
            }
        ))
    }
}

#[derive(ProcessArg, Clone, Copy)]
pub(crate) struct GradlePackageManager {}

impl MirrorConfigurate for GradlePackageManager {
    type R = GradleMirror;
    fn parse_args(&self) -> Vec<clap::Arg> {
        vec![
            arg!(-m --maven <maven_mirror>)
                .help("The mirror of maven repository")
                .required(false),
            arg!(-a --android <android_mirror>)
                .help("The mirror of android repository")
                .required(false),
            arg!(-p --plugins <gradle_plugins_mirror>)
                .help("The mirror of gradle plugins repository")
                .required(false),
        ]
    }

    fn name(&self) -> &'static str {
        "gradle"
    }

    fn current_mirror(&self) -> Option<GradleMirror> {
        match read_config(DEFAULT_GRADLE_PROFILES.to_vec()) {
            Ok((_, kts)) => {
                let lines = kts.lines();
                let mut maven = String::from_str("https://repo.maven.apache.org/maven2").unwrap();
                let mut android =
                    String::from_str("https://dl.google.com/dl/android/maven2").unwrap();
                let mut plugins = String::from_str("https://plugins.gradle.org/m2").unwrap();
                for line in lines {
                    if line.contains("https://repo.maven.apache.org/maven2") {
                        maven = line
                            .replace(" ", "")
                            .split(r#""to""#)
                            .last()
                            .unwrap()
                            .replace(r#"""#, "")
                            .replace(",", "")
                            .trim()
                            .to_string();
                    }
                    if line.contains("https://dl.google.com/dl/android/maven2") {
                        android = line
                            .replace(" ", "")
                            .split(r#""to""#)
                            .last()
                            .unwrap()
                            .replace(r#"""#, "")
                            .replace(",", "")
                            .trim()
                            .to_string();
                    }
                    if line.contains("https://plugins.gradle.org/m2") {
                        plugins = line
                            .replace(" ", "")
                            .split(r#""to""#)
                            .last()
                            .unwrap()
                            .replace(r#"""#, "")
                            .replace(",", "")
                            .trim()
                            .to_string();
                    }
                }
                Some(GradleMirror {
                    maven,
                    android,
                    plugins,
                })
            }
            Err(_) => None,
        }
    }

    fn get_mirrors(&self) -> Vec<GradleMirror> {
        let mirrors = include_str!("../../../mirrors/gradle.json");
        serde_json::from_str(mirrors).unwrap_or_default()
    }

    fn set_mirror(&self, args: &clap::ArgMatches) {
        let maven = args.get_one::<String>("maven").cloned().unwrap_or_default();
        let android = args
            .get_one::<String>("android")
            .cloned()
            .unwrap_or_default();
        let plugins = args
            .get_one::<String>("plugins")
            .cloned()
            .unwrap_or_default();
        let mirror = GradleMirror::new(maven, android, plugins);
        if let Ok(kts) = mirror.new_config() {
            let _ = write_config(DEFAULT_GRADLE_PROFILES.to_vec(), &kts);
        }
    }

    fn remove_mirror(&self, mirror: GradleMirror) {
        if self.current_mirror().unwrap() == mirror {
            self.reset_mirrors();
        }
    }

    fn reset_mirrors(&self) {
        let _ = write_config(DEFAULT_GRADLE_PROFILES.to_vec(), "");
    }

    fn test_mirror(&self, _mirror: GradleMirror) -> bool {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gen() {
        let mirror = GradleMirror::new(
            "https://maven.aliyun.com/repository/central".into(),
            "https://maven.aliyun.com/repository/google".into(),
            "https://maven.aliyun.com/repository/gradle-plugin".into(),
        );
        let config = mirror.new_config().unwrap();
        println!("{}", config);
    }
}
