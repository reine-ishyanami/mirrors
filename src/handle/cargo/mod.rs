mod object;

use crate::command::ProcessArg;
use anyhow::Result;
use clap::arg;
use object::CargoConfig;
use process_arg_derive::ProcessArg;
use serde::{Deserialize, Serialize};
use toml::Value;

use super::{MirrorConfigurate, Render};
use std::{collections::HashMap, env, path::PathBuf, sync::LazyLock};

static DEFAULT_CARGO_HOME: LazyLock<PathBuf> = LazyLock::new(|| {
    let home = dirs::home_dir().unwrap();
    home.join(".cargo")
});

const ENV_NAME: &str = "CARGO_HOME";

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct CargoMirror {
    name: String,
    url: String,
}

impl CargoMirror {
    pub(crate) fn new(name: String, url: String) -> Self {
        Self { name, url }
    }
}

impl Render for CargoMirror {
    fn new_config(&self) -> Result<String> {
        if let Ok(toml) = old_config() {
            if let Ok(mut old) = toml::from_str::<CargoConfig>(&toml) {
                old.source.insert(
                    self.name.clone(),
                    HashMap::from([("registry".into(), Value::String(self.url.clone()))]),
                );
                old.registries.insert(
                    self.name.clone(),
                    HashMap::from([("index".into(), Value::String(self.url.clone()))]),
                );
                for (k, v) in old.source.iter_mut() {
                    if k == "crates-io" {
                        // 替换掉原来的镜像名称
                        let rw = v.get_mut("replace-with");
                        if let Some(rw) = rw {
                            *rw = Value::String(self.name.clone());
                        }
                    }
                }
                return Ok(toml::to_string(&old)?);
            }
        }
        Ok(format!(
            include_str!("../../../templates/config.toml"),
            self.name, self.name, self.url, self.name, self.url
        ))
    }
}

#[derive(ProcessArg)]
pub(crate) struct CargoPackageManager {}

impl MirrorConfigurate for CargoPackageManager {
    type R = CargoMirror;
    fn parse_args(&self) -> Vec<clap::Arg> {
        vec![
            arg!(-n --name <NAME>).help("mirror name").required(true),
            arg!(-u --url <URL>).help("mirror url").required(true),
        ]
    }

    fn name(&self) -> &'static str {
        "cargo"
    }

    fn current_mirror(&self) -> Option<CargoMirror> {
        if let Ok(toml) = old_config() {
            if let Ok(old) = toml::from_str::<CargoConfig>(&toml) {
                let name = old
                    .source
                    .get("crates-io")
                    .map(|v| v.get("replace-with"))
                    .map(Option::unwrap)
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string();
                let url = old
                    .registries
                    .get(&name)
                    .map(|v| v.get("index"))
                    .map(Option::unwrap)
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string();
                return Some(CargoMirror { name, url });
            }
        }
        None
    }

    fn get_mirrors(&self) -> Vec<CargoMirror> {
        let mirrors = include_str!("../../../mirrors/cargo.json");
        serde_json::from_str(mirrors).unwrap_or_default()
    }

    fn set_mirror(&self, args: &clap::ArgMatches) {
        let name = args.get_one::<String>("name").cloned().unwrap_or_default();
        let url = args.get_one::<String>("url").cloned().unwrap_or_default();
        let mirror = CargoMirror::new(name, url);
        if let Ok(new_config) = mirror.new_config() {
            if !new_config.is_empty() {
                let cargo_config_path = profile_path();
                std::fs::write(cargo_config_path, new_config).unwrap();
            }
        }
    }

    fn remove_mirror(&self, mirror: CargoMirror) {
        if let Ok(toml) = old_config() {
            if let Ok(mut old) = toml::from_str::<CargoConfig>(&toml) {
                let name = old
                    .source
                    .get("crates-io")
                    .map(|v| v.get("replace-with"))
                    .map(Option::unwrap)
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string();
                if name == mirror.name {
                    old.source.remove("crates-io");
                }
                old.source.remove(&mirror.name);
                old.registries.remove(&mirror.name);
                let new_config = toml::to_string(&old).unwrap();
                if !new_config.is_empty() {
                    let cargo_config_path = profile_path();
                    std::fs::write(cargo_config_path, new_config).unwrap();
                }
            }
        }
    }

    fn reset_mirrors(&self) {
        if let Ok(toml) = old_config() {
            if let Ok(mut old) = toml::from_str::<CargoConfig>(&toml) {
                let new_source = old
                    .source
                    .get("crates-io")
                    .map(|v| {
                        let mut v = v.clone();
                        v.remove("replace-with");
                        v
                    })
                    .unwrap();
                old.source = HashMap::from([("crates-io".into(), new_source)]);
                old.registries.clear();
                let new_config = toml::to_string(&old).unwrap();
                if !new_config.is_empty() {
                    let cargo_config_path = profile_path();
                    std::fs::write(cargo_config_path, new_config).unwrap();
                }
            }
        }
    }

    fn test_mirror(&self, _mirror: CargoMirror) -> bool {
        todo!()
    }
}

fn old_config() -> Result<String> {
    let cargo_config_path = profile_path();
    let cargo_config = std::fs::read_to_string(&cargo_config_path)?;
    Ok(cargo_config)
}

fn profile_path() -> PathBuf {
    let cargo_home = match env::var(ENV_NAME) {
        Ok(value) => PathBuf::from(value),
        Err(_) => DEFAULT_CARGO_HOME.to_path_buf(),
    };
    cargo_home.join("config.toml")
}

#[cfg(test)]
mod tests {
    use object::CargoConfig;

    use super::*;

    #[test]
    fn resolve_cargo_config() {
        let cargo_config_text = r#"
[source.crates-io]
replace-with = 'rsproxy'

[source.rsproxy]
registry = "https://rsproxy.cn/crates.io-index"

[source.rsproxy-sparse]
registry = "sparse+https://rsproxy.cn/index/"

[registries.rsproxy]
index = "https://rsproxy.cn/crates.io-index"

[net]
git-fetch-with-cli = true
"#;
        let cargo_config: CargoConfig = toml::from_str(cargo_config_text).unwrap();
        println!("cargo_config: \n{:#?}", cargo_config);

        let cargo_config_text = toml::to_string(&cargo_config).unwrap();
        println!("cargo_config_text: \n{}", cargo_config_text);
    }

    #[test]
    fn test_gen() {
        let config = CargoMirror::new("111".into(), "https://111.com".into());
        let new_config = config.new_config().unwrap();
        println!("{}", new_config);
    }
}