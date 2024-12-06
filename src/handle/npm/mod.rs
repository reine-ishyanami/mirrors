use crate::command::ProcessArg;
use anyhow::Result;
use clap::arg;
use process_arg_derive::ProcessArg;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::LazyLock};

use super::{MirrorConfigurate, Render};

static DEFAULT_NPM_HOME: LazyLock<PathBuf> = LazyLock::new(|| dirs::home_dir().unwrap());

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct NpmMirror {
    url: String,
}

impl NpmMirror {
    pub fn new(url: String) -> Self {
        Self { url }
    }
}

impl Render for NpmMirror {
    fn new_config(&self) -> Result<String> {
        let str = match old_config() {
            Ok(properties) => {
                let mut new_properties = String::new();
                let mut has_registry = false;
                for line in properties.lines() {
                    if line.starts_with("registry=") {
                        new_properties.push_str(&format!("registry={}\n", self.url));
                        has_registry = true;
                    } else {
                        new_properties.push_str(line);
                        new_properties.push('\n');
                    }
                }
                if !has_registry {
                    new_properties.push_str(&format!("registry={}\n", self.url));
                }
                new_properties
            }
            Err(_) => {
                format!(include_str!("../../../templates/.npmrc"), self.url)
            }
        };
        Ok(str)
    }
}

#[derive(ProcessArg)]
pub(crate) struct NpmPackageManager {}

impl MirrorConfigurate for NpmPackageManager {
    type R = NpmMirror;
    fn parse_args(&self) -> Vec<clap::Arg> {
        vec![arg!(-u --url <url>)
            .help("The url of the mirror")
            .required(true)]
    }

    fn name(&self) -> &'static str {
        "npm"
    }

    fn current_mirror(&self) -> Option<NpmMirror> {
        match old_config() {
            Ok(properties) => properties
                .lines()
                .find(|line| line.starts_with("registry="))
                .map(|r| NpmMirror::new(r.replace("registry=", ""))),
            Err(_) => None,
        }
    }

    fn get_mirrors(&self) -> Vec<NpmMirror> {
        let mirrors = include_str!("../../../mirrors/npm.json");
        serde_json::from_str(mirrors).unwrap_or_default()
    }

    fn set_mirror(&self, args: &clap::ArgMatches) {
        let url = args.get_one::<String>("url").cloned().unwrap_or_default();
        let mirror = NpmMirror::new(url);
        if let Ok(properties) = mirror.new_config() {
            if !properties.is_empty() {
                let cargo_config_path = profile_path();
                std::fs::write(cargo_config_path, properties).unwrap();
            }
        }
    }

    fn remove_mirror(&self, mirror: NpmMirror) {
        if let Ok(properties) = old_config() {
            let mut new_properties = String::new();
            for line in properties.lines() {
                if !line.contains(&mirror.url) {
                    new_properties.push_str(line);
                    new_properties.push('\n');
                }
            }
            if !new_properties.is_empty() {
                let cargo_config_path = profile_path();
                std::fs::write(cargo_config_path, new_properties).unwrap();
            }
        }
    }

    fn reset_mirrors(&self) {
        if let Ok(properties) = old_config() {
            let mut new_properties = String::new();
            for line in properties.lines() {
                if !line.starts_with("registry=") {
                    new_properties.push_str(line);
                    new_properties.push('\n');
                }
            }
            if !new_properties.is_empty() {
                let cargo_config_path = profile_path();
                std::fs::write(cargo_config_path, new_properties).unwrap();
            }
        }
    }

    fn test_mirror(&self, _mirror: NpmMirror) -> bool {
        todo!()
    }
}

fn old_config() -> Result<String> {
    let config_path = profile_path();
    let config = std::fs::read_to_string(&config_path)?;
    Ok(config)
}

fn profile_path() -> PathBuf {
    let mvn_home = DEFAULT_NPM_HOME.to_path_buf();
    mvn_home.join(".npmrc")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gen() {
        let mirror = NpmMirror::new("https://registry.npm.taobao.org".into());
        let config = mirror.new_config().unwrap();
        println!("{}", config)
    }
}
