#![allow(unused_imports, unused_variables)]
mod object;

use std::{
    fmt::{Display, Formatter},
    path::PathBuf,
    sync::LazyLock,
};

use clap::arg;
use object::DockerConfig;
use process_arg_derive::ProcessArg;
use select_mirror_derive::SelectMirror;
use serde::{Deserialize, Serialize};

use crate::utils::file_utils::read_config;

use super::{write_config, MirrorConfigurate, Reader};
use anyhow::Result;

pub(crate) use os_specific::*;

#[cfg(target_os = "linux")]
mod os_specific {

    use super::*;

    static DEFAULT_DOCKER_PROFILE: LazyLock<Vec<PathBuf>> = LazyLock::new(|| {
        let path = PathBuf::from("/etc/docker/daemon.json");
        vec![path]
    });

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub(crate) struct DockerMirror {
        url: String,
    }

    impl DockerMirror {
        pub(crate) fn new(url: String) -> Self {
            Self { url }
        }
    }

    impl Display for DockerMirror {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.url)
        }
    }

    impl From<serde_json::Value> for DockerMirror {
        fn from(value: serde_json::Value) -> Self {
            let url = value["url"].as_str();
            Self::new(url.unwrap().to_string())
        }
    }

    impl Reader for DockerMirror {
        fn new_config(&self) -> Result<String> {
            if let Ok((_, json)) = read_config(DEFAULT_DOCKER_PROFILE.to_vec()) {
                if let Ok(mut old) = serde_json::from_str::<DockerConfig>(&json) {
                    old.registry_mirrors.insert(0, self.url.clone());
                    return Ok(serde_json::to_string(&old)?);
                }
            }
            Ok(format!(
                include_str!("../../../templates/daemon.json"),
                self.url
            ))
        }
    }

    #[derive(ProcessArg, SelectMirror, Clone, Copy)]
    pub(crate) struct DockerPackageManager {}

    impl MirrorConfigurate for DockerPackageManager {
        type R = DockerMirror;

        fn parse_args(&self) -> Vec<clap::Arg> {
            vec![arg!(-u --url <URL>).help("mirror url").required(true)]
        }

        fn name(&self) -> &'static str {
            "docker"
        }

        fn current_mirror(&self) -> Option<Self::R> {
            if let Ok((_, json)) = read_config(self.get_default_profile_vec()) {
                if let Ok(old) = serde_json::from_str::<DockerConfig>(&json) {
                    if let Some(url) = old.registry_mirrors.first().cloned() {
                        return Some(DockerMirror { url });
                    }
                }
            }
            None
        }

        fn get_mirrors(&self) -> Vec<Self::R> {
            let mirrors = include_str!("../../../mirrors/docker.json");
            serde_json::from_str(mirrors).unwrap_or_default()
        }

        fn set_mirror_by_args(&self, args: &clap::ArgMatches) {
            let url = args.get_one::<String>("url").cloned().unwrap_or_default();
            let mirror = DockerMirror::new(url);
            self.set_mirror(mirror);
        }

        fn get_default_profile_vec(&self) -> Vec<PathBuf> {
            DEFAULT_DOCKER_PROFILE.to_vec()
        }

        fn remove_mirror(&self, mirror: Self::R) {
            if let Ok((_, json)) = read_config(self.get_default_profile_vec()) {
                if let Ok(mut old) = serde_json::from_str::<DockerConfig>(&json) {
                    old.registry_mirrors.retain(|x| x != &mirror.url);
                    let json = toml::to_string(&old).unwrap();
                    let _ = write_config(self.get_default_profile_vec(), &json);
                }
            }
        }

        fn reset_mirrors(&self) {
            if let Ok((_, json)) = read_config(self.get_default_profile_vec()) {
                if let Ok(mut old) = serde_json::from_str::<DockerConfig>(&json) {
                    old.registry_mirrors.clear();
                    let json = toml::to_string(&old).unwrap();
                    let _ = write_config(self.get_default_profile_vec(), &json);
                }
            }
        }

        fn test_mirror(&self, mirror: Self::R) -> bool {
            todo!()
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::handle::{docker::DockerMirror, Reader};

        #[test]
        fn test_gen() {
            let config = DockerMirror::new("https://111.com".into());
            let new_config = config.new_config().unwrap();
            println!("{}", new_config);
        }
    }
}

#[cfg(not(target_os = "linux"))]
mod os_specific {

    use super::*;

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub(crate) struct DockerMirror {
        url: String,
    }

    impl DockerMirror {
        pub(crate) fn new(url: String) -> Self {
            Self { url }
        }
    }

    impl Display for DockerMirror {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.url)
        }
    }

    impl From<serde_json::Value> for DockerMirror {
        fn from(value: serde_json::Value) -> Self {
            let url = value["url"].as_str();
            Self::new(url.unwrap().to_string())
        }
    }

    impl Reader for DockerMirror {
        fn new_config(&self) -> Result<String> {
            unimplemented!("not support new_config for this platform")
        }
    }

    #[derive(ProcessArg, SelectMirror, Clone, Copy)]
    pub(crate) struct DockerPackageManager {}

    impl MirrorConfigurate for DockerPackageManager {
        type R = DockerMirror;

        fn parse_args(&self) -> Vec<clap::Arg> {
            vec![arg!(-u --url <URL>).help("mirror url").required(true)]
        }

        fn name(&self) -> &'static str {
            "docker"
        }

        fn current_mirror(&self) -> Option<Self::R> {
            unimplemented!("not support current_mirror for this platform")
        }

        fn get_mirrors(&self) -> Vec<Self::R> {
            unimplemented!("not support get_mirrors for this platform")
        }

        fn set_mirror_by_args(&self, _args: &clap::ArgMatches) {
            unimplemented!("not support set_mirror_by_args for this platform")
        }

        fn get_default_profile_vec(&self) -> Vec<PathBuf> {
            unimplemented!("not support get_default_profile_vec for this platform")
        }

        fn remove_mirror(&self, _mirror: Self::R) {
            unimplemented!("not support remove_mirror for this platform")
        }

        fn reset_mirrors(&self) {
            unimplemented!("not support reset_mirrors for this platform")
        }

        fn test_mirror(&self, _mirror: Self::R) -> bool {
            todo!()
        }
    }
}
