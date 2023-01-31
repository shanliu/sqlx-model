<div align="center">
<h3>根据数据库中表结构生成文档或代码的工具</h3>
</div>
<div align="center">
  <a href="https://github.com/shanliu/sqlx-model/edit/main/sqlx-model-tools/">
    <img src="https://img.shields.io/crates/v/sqlx-model-tools.svg?style=flat-square"
    alt="Crates.io version" />
  </a>
</div>

> 目前只支持mysql,预期sqlx支持的数据库都会支持。

> cargo 安装

```shell
cargo install sqlx-model-tools
```

> 源码安装

```shell
cargo install --path .
```

> 执行生成命令

```shell
db-to-code d2c_config.toml
```

> 配置说明 d2c_config.toml


```toml
# 数据库连接
db_url="mysql://user:xxx@0.0.0.0/db_name"
# 需要构建表过滤规则
tables="^y_"
# 或指定需要构建的表
# tables=[
#     "yaf_users"
# ]
# 输出文件模板 
# 可用变量 
#   表名 {table.table_name} 模型名 {table.model_name} 是否复合主键 {table.multi_pk}
#   表字段名 {field_data[].field_name} 显示字段名 {field_data[].column_name} 
#   是否是主键 {field_data[].is_pk} 是否NULL {field_data[].is_null} 
#   字段类型 {field_data[].type_name} 通过以下 type_map 映射后的值
#   字段默认值 {field_data[].default} 通过以下 default_map 映射后的值
#   字段注释 {field_data[].comment} 可通过 {field_data[].comment|rmln} 显示为单行
tpl_body="""
#[derive(sqlx::FromRow,sqlx_model::SqlxModel,Clone,Debug)]
#[sqlx_model(table_name="{table.table_name}")]
pub struct {table.model_name}Model \\{ {{ for field in field_data }} 
    {{ if field.comment }}/// {field.comment|rmln} {{ if field.default}} default:  {field.default} {{ endif }}{{ endif }}
    #[sqlx(default)]
    #[sqlx(rename="{field.field_name}")]
    pub {field.column_name}: {field.type_name},
{{ endfor }}}
"""
#{column_name}名转换规则: 参见规则列表
column_name_rule="lower_camel"
#{column_name}名前缀删除
column_name_start_replace=""
#{column_name}名后缀删除
column_name_end_replace=""
#{model_name}名转换规则: 参见规则列表
model_name_rule="camel"
#{model_name}名后缀删除
model_name_start_replace=""
#{model_name}名后缀删除
model_name_end_replace=""
#{table_name}名后缀删除
table_name_start_replace=""
#{table_name}名后缀删除
table_name_end_replace=""
# 是否每一个表合并成一个文件输出
outfile_merge=true
#文件存放名模板,为空时从标准输出,!!!注意!!!仅当 outfile_merge 为false时,才存在可用变量 {model_name} {table_name}
#outfile_name中{model_name}名转换规则: lower lower_camel kebab shouty_snake upper snake
#outfile_name="{model_name}Model.rs"
# 可用变量 
#   通过 tpl_body 渲染得到的内容 {items[].render_data}
#   表字段名 {items[].table.table_name} 显示字段名 {items[].table.model_name} 
#   字段数据,参考输出文件模板[tpl_body]的field_data变量 {items[].field_data.field_name ..等} 
outfile_merge_tpl="""
use sqlx::FromRow;
use sqlx_model::SqlxModel;
{{ for item in items }}  
// model : {item.table.model_name} [ {{ for field in item.field_data }} {field.field_name} {{endfor}}] {{endfor}}
{{ for item in items }} 
{item.render_data}
{{endfor}}
"""
outfile_name_rule="camel"
#outfile_name中{model_name}名后缀删除
outfile_name_start_replace=""
#outfile_name中{model_name}名后缀删除
outfile_name_end_replace=""
#文件存在时是否覆盖
outfile_overwrite=true
#默认NULL转换为指定类型
default_null="None"
#未设置默认值转换为指定类型
default_none="None"
#是否使用类型转换,默认:true
type_transform=true
#默认字段类型,当type_map都不匹配时使用此类型
type_default="String"
# 字段类型转换映射
[type_map.1]#.1 为优先级.越大越优先
#输出类型=[正则表达式,符合一个即使用]
"i32"=["int\\(\\d+\\)"]
[type_map.2]
"i8"=["tinyint\\(\\d+\\)","ENUM"]
[type_map.3]
"i16"=["smallint\\(\\d+\\)"]
[type_map.4]
"i64"=["bigint\\(\\d+\\)"]
[type_map.5]
"u32"=["int\\(\\d+\\)\\s+unsigned"]
[type_map.6]
"u8"=["tinyint\\(\\d+\\)\\s+unsigned"]
[type_map.7]
"u16"=["smallint\\(\\d+\\)\\s+unsigned"]
[type_map.8]
"u64"=["bigint\\(\\d+\\)\\s+unsigned"]
[type_map.9]
"f32"=["float"]
[type_map.10]
"f64"=["decimal"]
[type_map.11]
"f64"=["decimal","double"]
#匹配默认值并修改为指定格式
[default_map.1]#.1 为优先级.越大越优先
#正则表达式=转换输出结果
"^\\((-)?\\d+\\.\\d+\\)$"="$1"
[default_map.2]
"^\\((-)?\\d+\\)$"="$1"
[default_map.3]
"^\\(.*\\)$"="\"$1\""
[default_map.4]
"^\\s*$"="\"\""
```


> 名称转换规则列表 以下值用于 column_name_rule model_name_rule

```
lower 仅转为小写
upper 仅转为全部大写
snake 小写下划线
shouty_snake  大写下划线
shouty_kebab  大写中划线
kebab 中划线分割
upper_camel 转首字母大写 
lower_camel 转驼峰
```
