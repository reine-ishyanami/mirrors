use clap::Command;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    generate_command,
    handle::{
        cargo::CargoPackageManager, docker::DockerPackageManager, gradle::GradlePackageManager,
        maven::MavenPackageManager, npm::NpmPackageManager, pip::PipPackageManager,
        MirrorConfigurate,
    },
};

/// 选择内置镜像源
pub trait SelectMirror: MirrorConfigurate {
    fn select(&self);
}

/// 子命令处理命令行参数
pub trait ProcessArg: SelectMirror {
    fn process(&self, subcs: &clap::ArgMatches, v: Option<serde_json::Value>);
}

pub(crate) fn process() {
    let cargo = CargoPackageManager {};
    let mvn = MavenPackageManager {};
    let gradle = GradlePackageManager {};
    let npm = NpmPackageManager {};
    let pip = PipPackageManager {};
    let docker = DockerPackageManager {};

    generate_command!(cargo, mvn, gradle, npm, pip, docker);
}

pub(crate) fn read_mix_config() -> MixConfig {
    let mix_json = include_str!("../mirrors/mix.json");
    serde_json::from_str(mix_json).unwrap()
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct MixConfig {
    #[serde(flatten)]
    pub(crate) mirror_map: HashMap<String, Value>,
}
