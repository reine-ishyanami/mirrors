use std::path::PathBuf;

use anyhow::{bail, Result};

///
/// 读取旧配置文件内容，配置文件应为一个数组，按优先级顺序排列，高优先级的配置文件优先
///
pub(crate) fn read_config(paths: Vec<PathBuf>) -> Result<(PathBuf, String)> {
    for path in paths {
        if path.exists() {
            let origin_config = std::fs::read_to_string(&path)?;
            return Ok((path, origin_config));
        }
    }
    bail!("No config file found")
}

///
/// 写入新配置文件内容，配置文件应为一个数组，按优先级顺序排列，取第一个配置文件地址作为配置文件地址
///
pub(crate) fn write_config(paths: Vec<PathBuf>, new_config: &str) -> Result<()> {
    let path = paths.first().unwrap();
    if !new_config.is_empty() {
        let dir_name = path.parent().unwrap();
        if !dir_name.exists() {
            // 如果配置文件所处文件夹不存在，则创建文件夹
            std::fs::create_dir_all(dir_name)?;
        }
        // 写入新配置文件
        std::fs::write(path, new_config)?;
    } else {
        // 若配置文件内容为空，则删除原配置文件
        std::fs::remove_file(path)?;
    }
    Ok(())
}
