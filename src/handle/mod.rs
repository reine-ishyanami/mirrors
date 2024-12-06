#![allow(dead_code)]

use anyhow::Result;
use clap::Arg;

pub mod apt;
pub mod cargo;
pub mod docker;
pub mod gradle;
pub mod mvn;
pub mod npm;
pub mod pacman;
pub mod pip;

pub(super) trait Render {
    /// 参数输出到文件时的格式
    fn new_config(&self) -> Result<String>;
}

/// 镜像源配置接口
pub(super) trait MirrorConfigurate {
    type R: Render;
    ///
    /// 解析命令行参数
    ///
    fn parse_args(&self) -> Vec<Arg>;
    ///
    /// 获取配置名称
    ///
    fn name(&self) -> &'static str;
    ///
    /// 获取当前镜像源（如果没配置则返回 None）
    ///
    fn current_mirror(&self) -> Option<Self::R>;
    ///
    /// 获取所有镜像源
    ///
    fn get_mirrors(&self) -> Vec<Self::R>;
    ///
    /// 设置镜像源
    ///
    fn set_mirror(&self, args: &clap::ArgMatches);
    ///
    /// 移除镜像源
    ///
    fn remove_mirror(&self, mirror: Self::R);
    ///
    /// 重置镜像源
    ///
    fn reset_mirrors(&self);
    ///
    /// 测试镜像源
    ///
    fn test_mirror(&self, mirror: Self::R) -> bool;
}

pub(crate) struct MirrorConfigurator<T: MirrorConfigurate> {
    configurate: T,
}

impl<T: MirrorConfigurate> MirrorConfigurator<T> {
    pub fn new(configurate: T) -> Self {
        Self { configurate }
    }
}

impl<T: MirrorConfigurate> MirrorConfigurate for MirrorConfigurator<T> {
    type R = T::R;
    fn parse_args(&self) -> Vec<Arg> {
        self.configurate.parse_args()
    }
    fn name(&self) -> &'static str {
        self.configurate.name()
    }
    fn current_mirror(&self) -> Option<Self::R> {
        self.configurate.current_mirror()
    }
    fn get_mirrors(&self) -> Vec<Self::R> {
        self.configurate.get_mirrors()
    }

    fn set_mirror(&self, args: &clap::ArgMatches) {
        self.configurate.set_mirror(args)
    }

    fn remove_mirror(&self, mirror: Self::R) {
        self.configurate.remove_mirror(mirror)
    }

    fn reset_mirrors(&self) {
        self.configurate.reset_mirrors()
    }

    fn test_mirror(&self, mirror: Self::R) -> bool {
        self.configurate.test_mirror(mirror)
    }
}
