# mirrors

**mirrors** 是一个集合了常用包管理器镜像的管理工具

## 用法

```bash
# 给所有系统支持的包管理器设置默认镜像源
mirrors config
# 查看所有系统支持的包管理器的当前镜像源
mirrors list
# 重置所有系统支持的包管理器镜像源
mirrors reset
# 设置指定包管理器自定义镜像源
mirrors Xxx custom -x xx -y yy -z zz ...
# 设置指定包管理器镜像源
mirrors Xxx select
# 给指定包管理器设置默认配置的镜像源
mirrors XXX default
# 重置指定包管理器镜像源
mirrors Xxx reset
# 查看指定包管理器当前镜像源
mirrors Xxx get
```

## 目前支持的包管理器

- [ ] apt
- [x] cargo
- [x] docker (只支持Linux)
- [x] gradle (如果原来有其他配置慎用)
- [x] maven
- [x] npm
- [ ] pacman
- [x] pip

## 未来可能支持的功能

- [x] 增加对各个镜像源的延迟测试
- [ ] 自动选择最快的镜像源
- [ ] 自动剔除无效镜像源
- [ ] 增加对其他包管理器的支持
