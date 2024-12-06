use crate::{
    command::ProcessArg,
    utils::file_utils::{read_config, write_config},
};
use anyhow::Result;
use clap::arg;
use process_arg_derive::ProcessArg;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::LazyLock};

use super::{MirrorConfigurate, Reader};

static DEFAULT_NPM_PROFILES: LazyLock<Vec<PathBuf>> =
    LazyLock::new(|| vec![dirs::home_dir().unwrap().join(".npmrc")]);

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct NpmMirror {
    url: String,
}

impl NpmMirror {
    pub fn new(url: String) -> Self {
        Self { url }
    }
}

impl Reader for NpmMirror {
    fn new_config(&self) -> Result<String> {
        let str = match read_config(DEFAULT_NPM_PROFILES.to_vec()) {
            Ok((_, properties)) => {
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

#[derive(ProcessArg, Clone, Copy)]
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
        match read_config(DEFAULT_NPM_PROFILES.to_vec()) {
            Ok((_, properties)) => properties
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
            let _ = write_config(DEFAULT_NPM_PROFILES.to_vec(), &properties);
        }
    }

    fn remove_mirror(&self, mirror: NpmMirror) {
        if let Ok((_, properties)) = read_config(DEFAULT_NPM_PROFILES.to_vec()) {
            let mut new_properties = String::new();
            for line in properties.lines() {
                if !line.contains(&mirror.url) {
                    new_properties.push_str(line);
                    new_properties.push('\n');
                }
            }
            let _ = write_config(DEFAULT_NPM_PROFILES.to_vec(), &properties);
        }
    }

    fn reset_mirrors(&self) {
        if let Ok((_, properties)) = read_config(DEFAULT_NPM_PROFILES.to_vec()) {
            let mut new_properties = String::new();
            for line in properties.lines() {
                if !line.starts_with("registry=") {
                    new_properties.push_str(line);
                    new_properties.push('\n');
                }
            }
            let _ = write_config(DEFAULT_NPM_PROFILES.to_vec(), &properties);
        }
    }

    fn test_mirror(&self, _mirror: NpmMirror) -> bool {
        todo!()
    }
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
