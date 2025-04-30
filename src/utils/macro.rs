#[macro_export]
macro_rules! parse_command {
    (
        $cmd:expr,
        $( $pm:expr ),*
    ) => {
        {
            let map = read_mix_config().mirror_map;
            match $cmd.get_matches().subcommand() {
                Some(("config", _)) => {
                    $(
                        if $pm.support() {
                            let v = map.get($pm.name()).cloned();
                            if let Some(v) = v {
                                $pm.set_mirror_by_value(v);
                                println!("{} mirror config updated", $pm.name());
                            }
                        }
                    )*
                }
                Some(("list", _)) => {
                    println!("==============================================");
                    $(
                        if $pm.support() {
                            let mirror = $pm.current_mirror();
                            println!("{} mirror to \n {:#?}", $pm.name(), mirror);
                            println!("==============================================");
                        }
                    )*
                }
                Some(("reset", _)) => {
                    $(
                        if $pm.support() {
                            $pm.reset_mirrors();
                            println!("{} mirror has reset", $pm.name());
                        }
                    )*
                }
                Some((cmd, arg)) => {
                    let mut matched = false;
                    let mut support = true;
                    $(
                        let v = map.get($pm.name()).cloned();
                        if cmd == $pm.name() {
                            if $pm.support() {
                                matched = true;
                                $pm.process(arg, v);
                            } else {
                                support = false;
                            }
                        }
                    )*
                    if !support {
                        println!("{} package manager is not supported", uppercase_first_letter(cmd));
                    } else if !matched {
                        println!("Unknown command: {}", cmd);
                    }
                }
                None => {}
            }
        }
    };
}

#[macro_export]
macro_rules! generate_command {
    (
        $( $pm:expr ),*
    ) => {
        {
            use $crate::utils::string::uppercase_first_letter;
            use $crate::parse_command;

            let cmd = Command::new(std::env!("CARGO_PKG_NAME"))
                .about(std::env!("CARGO_PKG_DESCRIPTION"))
                .version(std::env!("CARGO_PKG_VERSION"))
                .subcommand_required(true)
                .arg_required_else_help(true)
                .allow_external_subcommands(true)
                .subcommand(
                    Command::new("config")
                        .about("Configurate mirrors for all support package managers")
                )
                .subcommand(
                    Command::new("list")
                        .about("List current mirrors for all support package managers")
                )
                .subcommand(
                    Command::new("reset")
                        .about("Reset mirrors for all support package managers")
                )
                $(
                    .subcommand(
                        Command::new($pm.name())
                            .about(
                                if $pm.support() {
                                    format!("Configure mirrors for {} package manager", $pm.name())
                                } else {
                                     format!("{} package manager is not supported", uppercase_first_letter($pm.name()))
                                }
                            )
                            .args_conflicts_with_subcommands(true)
                            .flatten_help(true)
                            .subcommand(
                                Command::new("custom")
                                    .args($pm.parse_args())
                            )
                            .subcommand(
                                Command::new("select")
                                    .about(format!("Select a mirror for {} package manager", $pm.name()))
                            )
                            .subcommand(
                                Command::new("default")
                                    .about(format!("Set default mirror for {} package manager", $pm.name()))
                            )
                            .subcommand(
                                Command::new("reset")
                                    .about(format!("Reset mirrors for {} package manager", $pm.name()))
                            )
                            .subcommand(
                                Command::new("get")
                                    .about(format!("Get current mirror of {} package manager", $pm.name()))
                            ),
                    )
                )*;

            parse_command!(cmd, $( $pm ),*);
        }
    };
}
