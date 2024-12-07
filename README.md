# mirrors

**mirrors** 是一个集合了常用包管理器镜像的管理工具

## 用法

```bash
# 查看指定包管理器当前镜像源
mirrors Xxx get
# 设置指定包管理器镜像源
mirrors Xxx config -x xx -y yy -z zz ...
# 重置指定包管理器镜像源
mirrors Xxx reset
```

## 目前支持的包管理器

- [ ] apt
- [x] cargo
- [ ] docker
- [x] gradle (如果原来有其他配置慎用)
- [x] maven
- [x] npm
- [ ] pacman
- [x] pip
