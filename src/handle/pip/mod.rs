mod object;

use crate::command::ProcessArg;
use anyhow::Result;
use clap::arg;
use object::PipConfig;
use process_arg_derive::ProcessArg;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::LazyLock};

use super::{MirrorConfigurate, Render};

static DEFAULT_PIP_HOME: LazyLock<PathBuf> = LazyLock::new(|| {
    if cfg!(target_os = "windows") {
        dirs::config_dir().unwrap().join("pip")
    } else {
        dirs::home_dir().unwrap().join(".pip")
    }
});

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct PipMirror {
    url: String,
    host: String,
}

impl PipMirror {
    pub fn new(url: String) -> Self {
        let host = url
            .split("://")
            .last()
            .unwrap()
            .split("/")
            .next()
            .unwrap()
            .to_owned();
        Self { url, host }
    }
}

impl Render for PipMirror {
    fn new_config(&self) -> Result<String> {
        let str = match old_config() {
            Ok(conf) => {
                let mut old: PipConfig = toml::from_str(&conf).unwrap();
                old.global.index_url = self.url.clone();
                old.install.trusted_host = self.host.clone();
                toml::to_string(&old)?
            }
            Err(_) => {
                format!(
                    include_str!("../../../templates/pip.conf"),
                    self.url, self.host
                )
            }
        };
        Ok(str)
    }
}

#[derive(ProcessArg)]
pub(crate) struct PipPackageManager {}

impl MirrorConfigurate for PipPackageManager {
    type R = PipMirror;
    fn parse_args(&self) -> Vec<clap::Arg> {
        vec![arg!(-u --url <URL>).help("mirror url").required(true)]
    }

    fn name(&self) -> &'static str {
        "pip"
    }

    fn current_mirror(&self) -> Option<PipMirror> {
        if let Ok(conf) = old_config() {
            if let Ok(old) = toml::from_str::<PipConfig>(&conf) {
                return Some(PipMirror::new(old.global.index_url));
            }
        }
        None
    }

    fn get_mirrors(&self) -> Vec<PipMirror> {
        let mirrors = include_str!("../../../mirrors/pip.json");
        serde_json::from_str(mirrors).unwrap_or_default()
    }

    fn set_mirror(&self, args: &clap::ArgMatches) {
        let url = args.get_one::<String>("url").cloned().unwrap_or_default();
        let mirror = PipMirror::new(url);
        if let Ok(new_config) = mirror.new_config() {
            if !new_config.is_empty() {
                let cargo_config_path = profile_path();
                std::fs::write(cargo_config_path, new_config).unwrap();
            }
        }
    }

    fn remove_mirror(&self, mirror: PipMirror) {
        if let Ok(conf) = old_config() {
            if let Ok(mut old) = toml::from_str::<PipConfig>(&conf) {
                if old.global.index_url == mirror.url {
                    old.global.index_url = "".to_string();
                    old.install.trusted_host = "".to_string();
                }
                let new_config = toml::to_string(&old).unwrap();
                if !new_config.is_empty() {
                    let cargo_config_path = profile_path();
                    std::fs::write(cargo_config_path, new_config).unwrap();
                }
            }
        }
    }

    fn reset_mirrors(&self) {
        if let Ok(conf) = old_config() {
            if let Ok(mut old) = toml::from_str::<PipConfig>(&conf) {
                old.global.index_url = "".to_string();
                old.install.trusted_host = "".to_string();
                let new_config = toml::to_string(&old).unwrap();
                if !new_config.is_empty() {
                    let cargo_config_path = profile_path();
                    std::fs::write(cargo_config_path, new_config).unwrap();
                }
            }
        }
    }

    fn test_mirror(&self, _mirror: PipMirror) -> bool {
        todo!()
    }
}

fn old_config() -> Result<String> {
    let config_path = profile_path();
    let config = std::fs::read_to_string(&config_path)?;
    Ok(config)
}

fn profile_path() -> PathBuf {
    let mvn_home = DEFAULT_PIP_HOME.to_path_buf();
    mvn_home.join(if cfg!(target_os = "windows") {
        "pip.ini"
    } else {
        "pip.conf"
    })
}

#[cfg(test)]
mod tests {
    use super::{PipMirror, Render};

    #[test]
    fn test_gen() {
        let mirror = PipMirror::new("https://mirrors.aliyun.com/pypi/simple/".into());
        let new_config = mirror.new_config().unwrap();
        println!("{}", new_config)
    }
}
