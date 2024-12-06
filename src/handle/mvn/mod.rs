mod object;

use crate::command::ProcessArg;
use anyhow::Result;
use clap::arg;
use object::{Mirror, Mirrors};
use process_arg_derive::ProcessArg;

use std::{env, path::PathBuf, sync::LazyLock, vec};

use super::{MirrorConfigurate, Render};

static DEFAULT_M2_HOME: LazyLock<PathBuf> = LazyLock::new(|| {
    let home = dirs::home_dir().unwrap();
    home.join(".m2")
});

const ENV_NAME: &str = "M2_HOME";

pub(crate) type MavenMirror = Mirror;

impl MavenMirror {
    pub fn new(id: String, name: String, mirror_of: String, url: String) -> Self {
        Self {
            id,
            name,
            mirror_of,
            url,
        }
    }
}

impl Render for MavenMirror {
    fn new_config(&self) -> Result<String> {
        let str = match old_config() {
            Ok(xml) => {
                let mut xml = xml;
                // 截取 mirrors 部分进行反序列化
                let start = xml.find("<mirrors>").unwrap_or_default();
                let end = xml.find("</mirrors>").unwrap_or_default() + "</mirrors>".len();
                // 如果没有 mirrors 段
                let (start, s) = if start == 0 {
                    let start = xml.find("</settings>").unwrap_or_default();
                    let mirrors = Mirrors {
                        mirror: vec![self.clone()],
                    };
                    let mut mirror_str = quick_xml::se::to_string(&mirrors).unwrap();
                    mirror_str = mirror_str.replace("Mirrors", "mirrors");
                    (start, mirror_str)
                } else {
                    let mirrors_xml = &xml[start..end];
                    let mut mirrors: Mirrors = quick_xml::de::from_str(mirrors_xml).unwrap();
                    // 删除原来的 mirrors 部分
                    xml.replace_range(start..end, "");
                    mirrors.mirror.insert(0, self.clone());

                    let mut mirror_str = quick_xml::se::to_string(&mirrors).unwrap();
                    mirror_str = mirror_str.replace("Mirrors", "mirrors");
                    (start, mirror_str)
                };
                xml.insert_str(start, s.as_str());
                xml = xml.replace("><", ">\n<");
                xml
            }
            Err(_) => {
                format!(
                    include_str!("../../../templates/settings.xml"),
                    self.id, self.name, self.mirror_of, self.url
                )
            }
        };
        Ok(str)
    }
}

#[derive(ProcessArg)]
pub(crate) struct MavenPackageManager {}

impl MirrorConfigurate for MavenPackageManager {
    type R = MavenMirror;
    fn parse_args(&self) -> Vec<clap::Arg> {
        vec![
            arg!(-i --id <id>)
                .help("The id of the mirror")
                .required(true),
            arg!(-n --name <name>)
                .help("The name of the mirror")
                .required(false),
            arg!(-m --mirror <mirror_of>)
                .help("The mirror-of of the mirror")
                .required(false),
            arg!(-u --url <url>)
                .help("The url of the mirror")
                .required(true),
        ]
    }

    fn name(&self) -> &'static str {
        "maven"
    }

    fn current_mirror(&self) -> Option<MavenMirror> {
        match old_config() {
            Ok(xml) => {
                // 截取 mirrors 部分进行反序列化
                let start = xml.find("<mirrors>").unwrap_or_default();
                let end = xml.find("</mirrors>").unwrap_or_default() + "</mirrors>".len();
                // 如果没有 mirrors 段
                if start == 0 {
                    None
                } else {
                    let mirrors_xml = &xml[start..end];
                    let mirrors: Mirrors = quick_xml::de::from_str(mirrors_xml).unwrap();
                    mirrors.mirror.first().cloned()
                }
            }
            Err(_) => None,
        }
    }

    fn get_mirrors(&self) -> Vec<MavenMirror> {
        let mirrors = include_str!("../../../mirrors/mvn.json");
        serde_json::from_str(mirrors).unwrap_or_default()
    }

    fn set_mirror(&self, args: &clap::ArgMatches) {
        let id = args.get_one::<String>("id").cloned().unwrap_or_default();
        let name = args.get_one::<String>("name").cloned().unwrap_or_default();
        let mirror_of = args
            .get_one::<String>("mirror_of")
            .cloned()
            .unwrap_or_default();
        let url = args.get_one::<String>("url").cloned().unwrap_or_default();
        let mirror = MavenMirror::new(id, name, mirror_of, url);
        if let Ok(xml) = mirror.new_config() {
            if !xml.is_empty() {
                let cargo_config_path = profile_path();
                std::fs::write(cargo_config_path, xml).unwrap();
            }
        }
    }

    fn remove_mirror(&self, mirror: MavenMirror) {
        if let Ok(xml) = old_config() {
            let mut xml = xml;
            // 截取 mirrors 部分进行反序列化
            let start = xml.find("<mirrors>").unwrap_or_default();
            let end = xml.find("</mirrors>").unwrap_or_default() + "</mirrors>".len();
            // 如果有 mirrors 段
            if start > 0 {
                let mirrors_xml = &xml[start..end];
                let mut mirrors: Mirrors = quick_xml::de::from_str(mirrors_xml).unwrap();
                // 删除原来的 mirrors 部分
                xml.replace_range(start..end, "");
                mirrors.mirror.retain(|m| m.id != mirror.id);

                let mut mirror_str = quick_xml::se::to_string(&mirrors).unwrap();
                mirror_str = mirror_str.replace("Mirrors", "mirrors");
                xml.insert_str(start, mirror_str.as_str());
                if !xml.is_empty() {
                    let cargo_config_path = profile_path();
                    std::fs::write(cargo_config_path, xml).unwrap();
                }
            }
        }
    }

    fn reset_mirrors(&self) {
        if let Ok(xml) = old_config() {
            let mut xml = xml;
            // 截取 mirrors 部分进行反序列化
            let start = xml.find("<mirrors>").unwrap_or_default();
            let end = xml.find("</mirrors>").unwrap_or_default() + "</mirrors>".len();
            // 如果没有 mirrors 段
            if start > 0 {
                let mirrors_xml = &xml[start..end];
                let mut mirrors: Mirrors = quick_xml::de::from_str(mirrors_xml).unwrap();
                // 删除原来的 mirrors 部分
                xml.replace_range(start..end, "");
                mirrors.mirror.clear();

                let mut mirror_str = quick_xml::se::to_string(&mirrors).unwrap();
                mirror_str = mirror_str.replace("Mirrors", "mirrors");
                xml.insert_str(start, mirror_str.as_str());
                if !xml.is_empty() {
                    let cargo_config_path = profile_path();
                    std::fs::write(cargo_config_path, xml).unwrap();
                }
            };
        }
    }

    fn test_mirror(&self, _mirror: MavenMirror) -> bool {
        todo!()
    }
}

fn old_config() -> Result<String> {
    let config_path = profile_path();
    let config = std::fs::read_to_string(&config_path)?;
    Ok(config)
}

fn profile_path() -> PathBuf {
    let mvn_home = match env::var(ENV_NAME) {
        Ok(value) => PathBuf::from(value),
        Err(_) => DEFAULT_M2_HOME.to_path_buf(),
    };
    mvn_home.join("settings.xml")
}

#[cfg(test)]
mod tests {
    use super::{MavenMirror, Render};

    #[test]
    fn test_gen() {
        let mirror = MavenMirror::new(
            "test".to_string(),
            "test".to_string(),
            "test".to_string(),
            "http://localhost:8081/repository/maven-public/".to_string(),
        );
        let xml = mirror.new_config().unwrap();
        println!("{}", xml);
    }
}
