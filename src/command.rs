use crate::handle::{
    cargo::CargoPackageManager, gradle::GradlePackageManager, mvn::MavenPackageManager,
    npm::NpmPackageManager, pip::PipPackageManager, MirrorConfigurate,
};

/// 子命令处理命令行参数
pub trait ProcessArg: MirrorConfigurate {
    fn process(&self, subcs: &clap::ArgMatches);
}

macro_rules! command {
    (
        $( $pm:expr ),*
    ) => {
        {
            use clap::Command;
            use crate::handle::MirrorConfigurate;

            clap::Command::new("mir")
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
                            .subcommand(Command::new("config").args(
                                $pm.parse_args()
                            ))
                            .subcommand(Command::new("reset"))
                            .subcommand(Command::new("get")),
                    )
                )*
        }
    };
}

pub(crate) fn process() {
    let cargo = CargoPackageManager {};
    let mvn = MavenPackageManager {};
    let gradle = GradlePackageManager {};
    let npm = NpmPackageManager {};
    let pip = PipPackageManager {};
    let cmd = command!(cargo, mvn, gradle, npm, pip);

    match cmd.get_matches().subcommand() {
        Some(("config", _)) => {}
        Some(("list", _)) => {}
        Some(("reset", _)) => {}
        Some(("cargo", arg)) => cargo.process(arg),
        Some(("maven", arg)) => mvn.process(arg),
        Some(("gradle", arg)) => gradle.process(arg),
        Some(("npm", arg)) => npm.process(arg),
        Some(("pip", arg)) => pip.process(arg),
        Some((cmd, _)) => {
            println!("Unknown command: {}", cmd);
        }
        None => {}
    }
}
