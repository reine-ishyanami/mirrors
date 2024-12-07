# 如果需要加入新的包管理器镜像管理，请按照以下步骤进行：

1. Fork 本项目，并克隆到本地。

2. 在 `mirrors/{包管理器名称}.json` 文件中创建一个数组，并至少添加一项，该项内容必须有设置新镜像源所需的全部属性。

3. 在 `templates/{对应包管理器配置文件名称}` 文件中添加默认的配置模板，使用 `rust` 的插值语法。

4. 在 `src/handle/{包管理器名称}/mod.rs` 文件中添加具体逻辑代码。

5. 在 `src/command.rs` 文件中创建新定义的包管理器镜像操作对象，并添加到指令生成宏中。

6. 在 `README.md` 文件中添加新包管理器的说明。

7. 提交代码，并创建 Pull Request。

## 新建包管理器镜像操作对象所需代码详解

### `object.rs` 文件中定义包管理器配置文件的序列化对象。

### `mod.rs` 代码片段解释说明

1. `ENV_NAME` 表示该包管理器对应环境变量名称（有些包管理器根据此环境变量的位置存放配置文件）

2. `DEFAULT_XXX_PROFILES` 表示该包管理器对应配置文件路径，由于配置文件可能存在于多处，所以需要用数组表示，以数组中出现的顺序作为优先级。

3. `mirrors/{包管理器名称}.json` 对应的序列化结构体，需要为此结构体创建 `new` 方法和实现 `Reader`, `From<serde_json::Value>`, `Display` trait, 并派生 `Debug`, `Deserialize`, `Serialize`, `Clone` trait.

5. `XxxPackageManager` 结构体，需实现 `MirrorConfigurate` trait, 并派生 `ProcessArg`, `SelectMirror`, `Clone`, `Copy`.
