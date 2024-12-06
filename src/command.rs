use crate::handle::{
    cargo::CargoPackageManager, gradle::GradlePackageManager, mvn::MavenPackageManager,
    npm::NpmPackageManager, pip::PipPackageManager, MirrorConfigurate,
};

/// 子命令处理命令行参数
pub trait ProcessArg: MirrorConfigurate {
    fn process(&self, subcs: &clap::ArgMatches);
}

macro_rules! parse_command {
    (
        $( $pm:expr ),*
    ) => {
        {
            use clap::Command;
            use crate::handle::MirrorConfigurate;

            let cmd = clap::Command::new("mir")
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
                )*;

            match cmd.get_matches().subcommand() {
                Some(("config", _)) => {}
                Some(("list", _)) => {}
                Some(("reset", _)) => {}
                Some((cmd, arg)) => {
                    let mut matched = false;
                    $(
                        if cmd == $pm.name() {
                            matched = true;
                            $pm.process(arg)
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
    parse_command!(cargo, mvn, gradle, npm, pip);
}
