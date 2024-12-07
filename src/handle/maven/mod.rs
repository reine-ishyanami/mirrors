mod object;

use crate::utils::file_utils::{read_config, write_config};
use anyhow::Result;
use clap::arg;
use object::{Mirror, Mirrors};
use process_arg_derive::ProcessArg;

use std::{env, fmt::Display, path::PathBuf, sync::LazyLock, vec};

use super::{MirrorConfigurate, Reader};
use select_mirror_derive::SelectMirror;

const ENV_NAME: &str = "M2_HOME";

static DEFAULT_MAVEN_PROFILES: LazyLock<Vec<PathBuf>> = LazyLock::new(|| {
    let profile_path = match env::var(ENV_NAME) {
        Ok(value) => PathBuf::from(value),
        Err(_) => dirs::home_dir().unwrap().join(".m2"),
    };
    vec![profile_path.join("settings.xml")]
});

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

impl Display for MavenMirror {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "id: {}, name: {}, mirror-of: {}, url: {}",
            self.id, self.name, self.mirror_of, self.url
        )
    }
}

impl From<serde_json::Value> for MavenMirror {
    fn from(value: serde_json::Value) -> Self {
        let id = value["id"].as_str();
        let name = value["name"].as_str();
        let mirror_of = value["mirrorOf"].as_str();
        let url = value["url"].as_str();

        Self::new(
            id.unwrap_or_default().to_string(),
            name.unwrap_or_default().to_string(),
            mirror_of.unwrap_or_default().to_string(),
            url.unwrap_or_default().to_string(),
        )
    }
}

impl Reader for MavenMirror {
    fn new_config(&self) -> Result<String> {
        let str = match read_config(DEFAULT_MAVEN_PROFILES.to_vec()) {
            Ok((_, xml)) => {
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
                    let mut mirrors: Mirrors =
                        if let Ok(mirrors) = quick_xml::de::from_str(mirrors_xml) {
                            mirrors
                        } else {
                            Mirrors { mirror: vec![] }
                        };
                    // 删除原来的 mirrors 部分
                    xml.replace_range(start..end, "");
                    // 先删除与当前插入id相同的mirror，再插入新的mirror
                    mirrors.mirror.retain(|m| m.id != self.id);
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

#[derive(ProcessArg, SelectMirror, Clone, Copy)]
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
        match read_config(self.get_default_profile_vec()) {
            Ok((_, xml)) => {
                // 截取 mirrors 部分进行反序列化
                let start = xml.find("<mirrors>").unwrap_or_default();
                let end = xml.find("</mirrors>").unwrap_or_default() + "</mirrors>".len();
                // 如果没有 mirrors 段
                if start == 0 {
                    None
                } else {
                    let mirrors_xml = &xml[start..end];
                    let mirrors: Mirrors = if let Ok(mirrors) = quick_xml::de::from_str(mirrors_xml)
                    {
                        mirrors
                    } else {
                        Mirrors { mirror: vec![] }
                    };
                    // 删除原来的 mirrors 部分
                    mirrors.mirror.first().cloned()
                }
            }
            Err(_) => None,
        }
    }

    fn get_mirrors(&self) -> Vec<MavenMirror> {
        let mirrors = include_str!("../../../mirrors/maven.json");
        serde_json::from_str(mirrors).unwrap_or_default()
    }

    fn set_mirror_by_args(&self, args: &clap::ArgMatches) {
        let id = args.get_one::<String>("id").cloned().unwrap_or_default();
        let name = args.get_one::<String>("name").cloned().unwrap_or_default();
        let mirror_of = args
            .get_one::<String>("mirror_of")
            .cloned()
            .unwrap_or_default();
        let url = args.get_one::<String>("url").cloned().unwrap_or_default();
        let mirror = MavenMirror::new(id, name, mirror_of, url);
        self.set_mirror(mirror);
    }

    fn remove_mirror(&self, mirror: MavenMirror) {
        if let Ok((_, xml)) = read_config(self.get_default_profile_vec()) {
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
                let _ = write_config(self.get_default_profile_vec(), &xml);
            }
        }
    }

    fn reset_mirrors(&self) {
        if let Ok((_, xml)) = read_config(self.get_default_profile_vec()) {
            let mut xml = xml;
            // 截取 mirrors 部分进行反序列化
            let start = xml.find("<mirrors>").unwrap_or_default();
            let end = xml.find("</mirrors>").unwrap_or_default() + "</mirrors>".len();
            // 如果有 mirrors 段
            if start > 0 {
                // 删除原来的 mirrors 部分
                xml.replace_range(start..end, "");
                let mirror_str = "<mirrors>\n</mirrors>";
                xml.insert_str(start, mirror_str);
                let _ = write_config(self.get_default_profile_vec(), &xml);
            };
        }
    }

    fn test_mirror(&self, _mirror: MavenMirror) -> bool {
        todo!()
    }

    fn get_default_profile_vec(&self) -> Vec<PathBuf> {
        DEFAULT_MAVEN_PROFILES.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::{MavenMirror, Reader};

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
