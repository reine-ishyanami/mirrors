mod object;

use crate::utils::{
    file_utils::{read_config, write_config},
    net_utils::test_connection,
};
use anyhow::Result;
use clap::arg;
use object::CargoConfig;
use process_arg_derive::ProcessArg;
use select_mirror_derive::SelectMirror;
use serde::{Deserialize, Serialize};
use toml::Value;

use super::{MirrorConfigurate, Reader};
use std::{collections::HashMap, env, fmt::Display, path::PathBuf, sync::LazyLock};

const ENV_NAME: &str = "CARGO_HOME";

static DEFAULT_CARGO_PROFILES: LazyLock<Vec<PathBuf>> = LazyLock::new(|| {
    let profile_path = match env::var(ENV_NAME) {
        Ok(value) => PathBuf::from(value),
        Err(_) => dirs::home_dir().unwrap().join(".cargo"),
    };
    vec![profile_path.join("config.toml")]
});

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct CargoMirror {
    name: String,
    url: String,
    /// The delay time of the url, in milliseconds.
    #[serde(default)]
    url_delay: i128,
}

impl CargoMirror {
    pub(crate) fn new(name: String, url: String) -> Self {
        // 测试延迟
        let url_delay = if let Ok((_, delay)) = test_connection(url.clone()) {
            delay as i128
        } else {
            -1
        };
        Self {
            name,
            url,
            url_delay,
        }
    }
}

impl Display for CargoMirror {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {} {}ms", self.name, self.url, self.url_delay)
    }
}

impl From<serde_json::Value> for CargoMirror {
    fn from(value: serde_json::Value) -> Self {
        let name = value["name"].as_str();
        let url = value["url"].as_str();
        Self::new(name.unwrap().to_string(), url.unwrap().to_string())
    }
}

impl Reader for CargoMirror {
    fn new_config(&self) -> Result<String> {
        if let Ok((_, toml)) = read_config(DEFAULT_CARGO_PROFILES.to_vec()) {
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

#[derive(ProcessArg, SelectMirror, Clone, Copy)]
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
        if let Ok((_, toml)) = read_config(self.get_default_profile_vec()) {
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
                return Some(CargoMirror::new(name, url));
            }
        }
        None
    }

    fn get_mirrors(&self) -> Vec<CargoMirror> {
        let mirrors = include_str!("../../../mirrors/cargo.json");
        let mirrors: Vec<Self::R> = serde_json::from_str(mirrors).unwrap_or_default();
        mirrors
            .into_iter()
            .map(|x| {
                let url_delay = if let Ok((_, delay)) = test_connection(x.url.clone()) {
                    delay as i128
                } else {
                    -1
                };
                Self::R { url_delay, ..x }
            })
            .collect()
    }

    fn set_mirror_by_args(&self, args: &clap::ArgMatches) {
        let name = args.get_one::<String>("name").cloned().unwrap_or_default();
        let url = args.get_one::<String>("url").cloned().unwrap_or_default();
        let mirror = CargoMirror::new(name, url);
        self.set_mirror(mirror);
    }

    fn remove_mirror(&self, mirror: CargoMirror) {
        if let Ok((_, toml)) = read_config(self.get_default_profile_vec()) {
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
                let toml = toml::to_string(&old).unwrap();
                let _ = write_config(self.get_default_profile_vec(), &toml);
            }
        }
    }

    fn reset_mirrors(&self) {
        if let Ok((_, toml)) = read_config(self.get_default_profile_vec()) {
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
                let toml = toml::to_string(&old).unwrap();
                let _ = write_config(self.get_default_profile_vec(), &toml);
            }
        }
    }

    fn get_default_profile_vec(&self) -> Vec<PathBuf> {
        DEFAULT_CARGO_PROFILES.to_vec()
    }
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
