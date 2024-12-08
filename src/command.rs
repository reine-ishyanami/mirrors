use clap::Command;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::handle::{
    cargo::CargoPackageManager, docker::DockerPackageManager, gradle::GradlePackageManager,
    maven::MavenPackageManager, npm::NpmPackageManager, pip::PipPackageManager, MirrorConfigurate,
};

/// 选择内置镜像源
pub trait SelectMirror: MirrorConfigurate {
    fn select(&self);
}

/// 子命令处理命令行参数
pub trait ProcessArg: SelectMirror {
    fn process(&self, subcs: &clap::ArgMatches, v: Option<serde_json::Value>);
}

macro_rules! parse_command {
    (
        $( $pm:expr ),*
    ) => {
        {
            let cmd = Command::new("mir")
                .about("Configure Package Manager Mirrors")
                .subcommand_required(true)
                .arg_required_else_help(true)
                .allow_external_subcommands(true)
                .subcommand(Command::new("config"))
                .subcommand(Command::new("list"))
                .subcommand(Command::new("reset"))
                $(
                    .subcommand(
                        Command::new($pm.name())
                            .args_conflicts_with_subcommands(true)
                            .flatten_help(true)
                            .subcommand(Command::new("custom").args(
                                $pm.parse_args()
                            ))
                            .subcommand(Command::new("select"))
                            .subcommand(Command::new("default"))
                            .subcommand(Command::new("reset"))
                            .subcommand(Command::new("get")),
                    )
                )*;

            let map = read_mix_config().mirror_map;
            match cmd.get_matches().subcommand() {
                Some(("config", _)) => {
                    $(
                        let v = map.get($pm.name()).cloned();
                        if let Some(v) = v {
                            $pm.set_mirror_by_value(v);
                            println!("{} mirror config updated", $pm.name());
                        }
                    )*
                }
                Some(("list", _)) => {
                    println!("==============================================");
                    $(
                        let mirror = $pm.current_mirror();
                        println!("{} mirror to \n {:#?}", $pm.name(), mirror);
                        println!("==============================================");
                    )*
                }
                Some(("reset", _)) => {
                    $(
                        $pm.reset_mirrors();
                        println!("{} mirror has reset", $pm.name());
                    )*
                }
                Some((cmd, arg)) => {
                    let mut matched = false;
                    $(
                        let v = map.get($pm.name()).cloned();
                        if cmd == $pm.name() {
                            matched = true;
                            $pm.process(arg, v)
                        }
                    )*
                    if matched == false {
                        println!("Unknown command: {}", cmd);
                    }
                }
                None => {}
            }
        }
    };
}

pub(crate) fn process() {
    let cargo = CargoPackageManager {};
    let mvn = MavenPackageManager {};
    let gradle = GradlePackageManager {};
    let npm = NpmPackageManager {};
    let pip = PipPackageManager {};
    let docker = DockerPackageManager {};

    parse_command!(cargo, mvn, gradle, npm, pip, docker);
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
