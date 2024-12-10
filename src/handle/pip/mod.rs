use anyhow::Result;
use clap::arg;
use ini::Ini;
use process_arg_derive::ProcessArg;
use select_mirror_derive::SelectMirror;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, path::PathBuf, sync::LazyLock};

use super::{MirrorConfigurate, Reader};

use crate::utils::{
    file_utils::{read_config, write_config},
    net_utils::test_connection,
};

static DEFAULT_PIP_PROFILES: LazyLock<Vec<PathBuf>> = LazyLock::new(|| {
    vec![if cfg!(target_os = "windows") {
        dirs::config_dir().unwrap().join("pip").join("pip.ini")
    } else {
        dirs::home_dir().unwrap().join(".pip").join("pip.conf")
    }]
});

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct PipMirror {
    url: String,
    host: String,
    /// The delay time of the url, in milliseconds.
    #[serde(default)]
    url_delay: i128,
}

impl PipMirror {
    pub fn new(url: String) -> Self {
        let host = url
            .clone()
            .split("://")
            .last()
            .unwrap()
            .split("/")
            .next()
            .unwrap()
            .to_owned();
        Self {
            url,
            host,
            url_delay: -1,
        }
    }
}

impl Display for PipMirror {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}ms", self.url, self.url_delay)
    }
}

impl From<serde_json::Value> for PipMirror {
    fn from(value: serde_json::Value) -> Self {
        let url = value["url"].as_str();
        Self::new(url.unwrap_or_default().to_string())
    }
}

impl Reader for PipMirror {
    fn new_config(&self) -> Result<String> {
        let str = match read_config(DEFAULT_PIP_PROFILES.to_vec()) {
            Ok((_, conf)) => {
                let mut ini = Ini::load_from_str(&conf)?;
                ini.with_section(Some("global"))
                    .set("index-url", self.url.clone());
                ini.with_section(Some("install"))
                    .set("trusted-host", self.host.clone());
                let mut writer = Vec::new();
                ini.write_to(&mut writer)?;
                String::from_utf8(writer)?
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

#[derive(ProcessArg, SelectMirror, Clone, Copy)]
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
        if let Ok((_, conf)) = read_config(self.get_default_profile_vec()) {
            if let Ok(ini) = Ini::load_from_str(&conf) {
                let url = ini
                    .section(Some("global"))
                    .map(|i| i.get("index-url").unwrap_or_default())
                    .unwrap_or_default();
                let host = ini
                    .section(Some("install"))
                    .map(|i| i.get("trusted-host").unwrap_or_default())
                    .unwrap_or_default();
                if url.contains(host) && !host.is_empty() && !url.is_empty() {
                    return Some(PipMirror::new(url.to_string()));
                }
            }
        }
        None
    }

    fn get_mirrors(&self) -> Vec<Self::R> {
        let mirrors = include_str!("../../../mirrors/pip.json");
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
        let url = args.get_one::<String>("url").cloned().unwrap_or_default();
        let mirror = PipMirror::new(url);
        self.set_mirror(mirror);
    }

    fn remove_mirror(&self, mirror: PipMirror) {
        if let Ok((_, conf)) = read_config(self.get_default_profile_vec()) {
            if let Ok(ref mut ini) = Ini::load_from_str(&conf) {
                let url = ini
                    .section(Some("global"))
                    .map(|i| i.get("index-url").unwrap_or_default())
                    .unwrap_or_default();
                if url == mirror.url {
                    ini.with_section(Some("global")).delete(&"index-url");
                    ini.with_section(Some("install")).delete(&"trusted-host");
                }
                let mut writer = Vec::new();
                ini.write_to(&mut writer).unwrap_or_default();
                let new_config = String::from_utf8(writer).unwrap_or_default();
                let _ = write_config(self.get_default_profile_vec(), &new_config);
            }
        }
    }

    fn reset_mirrors(&self) {
        if let Ok((_, conf)) = read_config(self.get_default_profile_vec()) {
            if let Ok(ref mut ini) = Ini::load_from_str(&conf) {
                ini.with_section(Some("global")).delete(&"index-url");
                ini.with_section(Some("install")).delete(&"trusted-host");
                let mut writer = Vec::new();
                ini.write_to(&mut writer).unwrap_or_default();
                let new_config = String::from_utf8(writer).unwrap_or_default();
                let _ = write_config(self.get_default_profile_vec(), &new_config);
            }
        }
    }

    fn get_default_profile_vec(&self) -> Vec<PathBuf> {
        DEFAULT_PIP_PROFILES.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::{PipMirror, Reader};

    #[test]
    fn test_gen() {
        let mirror = PipMirror::new("https://mirrors.aliyun.com/pypi/simple/".into());
        let new_config = mirror.new_config().unwrap();
        println!("{}", new_config)
    }
}
