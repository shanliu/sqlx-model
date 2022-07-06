use std::{
    collections::BTreeMap,
    fmt::{Display, Formatter},
};

use config::{Config, ConfigError};
use heck::{
    ToKebabCase, ToLowerCamelCase, ToShoutyKebabCase, ToShoutySnakeCase, ToSnakeCase,
    ToUpperCamelCase,
};
use regex::Regex;
use serde::Serialize;
use serde_json::Value;
use std::fmt::Result as FmtResult;
use tinytemplate::{format_unescaped, TinyTemplate};
use tokio::io::AsyncWriteExt;

use crate::mysql::MySqlParse;

#[derive(Clone, PartialEq, Eq)]
pub struct DataField {
    pub field_name: String,
    pub type_name: String,
    pub is_null: bool,
    pub is_pk: bool,
    pub default: Option<String>,
    pub comment: String,
}

pub struct DataValue {
    pub table_name: String,
    pub columns: Vec<DataField>,
}

#[derive(Serialize)]
pub struct RenderField {
    field_name: String,
    column_name: String,
    type_name: String,
    default: String,
    is_pk: bool,
    is_null: bool,
    comment: String,
}
#[derive(Serialize, Clone)]
pub struct RenderTableInfo {
    model_name: String,
    table_name: String,
    multi_pk: bool,
}
#[derive(Serialize)]
pub struct RenderBody {
    table: RenderTableInfo,
    field_data: Vec<RenderField>,
}
#[derive(Serialize)]
pub struct RenderMergeItem {
    render_data: String,
    table: RenderTableInfo,
    field_data: Vec<RenderField>,
}
#[derive(Serialize)]
pub struct RenderMergeBody {
    items: Vec<RenderMergeItem>,
}

#[derive(Debug)]
pub enum ConfigParseError {
    Config(String),
    Regex(String),
    Tpl(String),
    Io(String),
}
impl Display for ConfigParseError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{:?}", self)
    }
}
impl From<ConfigError> for ConfigParseError {
    fn from(err: ConfigError) -> Self {
        ConfigParseError::Config(format!("{}", err))
    }
}
impl From<regex::Error> for ConfigParseError {
    fn from(err: regex::Error) -> Self {
        ConfigParseError::Regex(format!("{}", err))
    }
}
impl From<tinytemplate::error::Error> for ConfigParseError {
    fn from(err: tinytemplate::error::Error) -> Self {
        ConfigParseError::Tpl(format!("{}", err))
    }
}
impl From<std::io::Error> for ConfigParseError {
    fn from(err: std::io::Error) -> Self {
        ConfigParseError::Io(format!("{}", err))
    }
}
impl From<sqlx::Error> for ConfigParseError {
    fn from(err: sqlx::Error) -> Self {
        ConfigParseError::Io(format!("{}", err))
    }
}

#[async_trait::async_trait]
pub trait TableParseData {
    type OUT: TableParseData;
    async fn new(uri: &str) -> Result<Self::OUT, ConfigParseError>;
    async fn list_tables(&self) -> Result<Vec<String>, ConfigParseError>;
    async fn parse_table_column(&self, table: &str) -> Result<Vec<DataField>, ConfigParseError>;
}

pub struct ConfigParse {
    type_defalut: String,
    type_map: Vec<(String, Vec<String>)>,
    default_none: String,
    default_null: String,
    type_transform: bool,
    defalut_map: Vec<(String, String)>,
    uri: String,
    table_name_rule: (String, String),
    model_name_rule: (String, String, String),
    column_name_rule: (String, String, String),
    outfile_name_rule: (String, String, String),
    tables: Vec<String>,
    outfile_name: String,
    outfile_merge_tpl: String,
    outfile_merge: bool,
    outfile_overwrite: bool,
    tpl_body: String,
    table_rule: Option<String>,
}
impl ConfigParse {
    pub async fn run(config_file: &str) -> Result<(), ConfigParseError> {
        let settings = Config::builder()
            .add_source(config::File::with_name(config_file))
            .build()?;
        let config_parse = ConfigParse::parse(&settings)?;
        if config_parse.uri.starts_with("mysql") {
            config_parse
                .table_runder(MySqlParse::new(config_parse.uri.as_str()).await?)
                .await?
        } else {
            return Err(ConfigParseError::Config(
                "database type not support".to_string(),
            ));
        }
        Ok(())
    }
    fn add_formatter<'t>(&self, mut tpl: TinyTemplate<'t>) -> TinyTemplate<'t> {
        tpl.add_formatter("rmln", |v, s: &mut String| match v {
            Value::String(ts) => {
                use core::fmt::Write;
                let mut rs = ts.replace('\n', "");
                rs = rs.replace('\r', "");
                write!(s, "{}", rs)?;
                Ok(())
            }
            _ => Err(tinytemplate::error::Error::GenericError {
                msg: "line format only support in string.".to_string(),
            }),
        });
        tpl
    }
    async fn table_runder<PT>(&self, db_parse: PT) -> Result<(), ConfigParseError>
    where
        PT: TableParseData,
    {
        let mut tpl = TinyTemplate::new();
        tpl.set_default_formatter(&format_unescaped);
        tpl = self.add_formatter(tpl);
        tpl.add_template("body", self.tpl_body.as_str())?;
        if !self.outfile_name.is_empty() {
            tpl.add_template("path", self.outfile_name.as_str())?;
        }
        if !self.outfile_merge_tpl.is_empty() {
            tpl.add_template("body_merge", self.outfile_merge_tpl.as_str())?;
        }
        let mut tables = vec![];
        match &self.table_rule {
            Some(table_rule) => {
                let regex = Regex::new(table_rule)?;
                for t in db_parse.list_tables().await?.into_iter() {
                    if regex.is_match(t.as_str()) {
                        tables.push(t);
                    }
                }
            }
            None => {
                tables = self.tables.clone();
            }
        }
        if tables.is_empty() {
            return Err(ConfigParseError::Config(
                "No table matching the record was found".to_string(),
            ));
        }
        let mut merge_item = vec![];
        for table in tables.iter() {
            let columns = db_parse.parse_table_column(table).await?;
            let table_pk_num = columns.iter().filter(|e| e.is_pk).count();
            let table_name =
                self.replace_name((&self.table_name_rule.0, &self.table_name_rule.1), table);
            let data = DataValue {
                table_name,
                columns,
            };
            let model_name = self.parse_name(&self.model_name_rule, table);
            let mut name = RenderTableInfo {
                model_name,
                multi_pk: table_pk_num > 1,
                table_name: data.table_name.clone(),
            };
            let render_body = self.render_data(name.clone(), data).await?;
            let body_str = tpl.render("body", &render_body)?;
            if !self.outfile_merge && !self.outfile_name.is_empty() {
                name.model_name = self.parse_name(&self.outfile_name_rule, table);
                let res = tpl.render("path", &name)?;
                self.write_file(Some(res), body_str).await?;
            } else {
                merge_item.push(RenderMergeItem {
                    render_data: body_str,
                    table: render_body.table,
                    field_data: render_body.field_data,
                });
            }
        }
        let outbody = if !self.outfile_merge_tpl.is_empty() {
            tpl.render("body_merge", &RenderMergeBody { items: merge_item })?
        } else {
            merge_item
                .into_iter()
                .map(|e| e.render_data)
                .collect::<Vec<String>>()
                .join("\n")
        };
        if self.outfile_name.is_empty() {
            self.write_file(None, outbody).await?;
        } else if self.outfile_merge {
            let res = tpl.render("path", &Value::Null)?;
            self.write_file(Some(res), outbody).await?;
        }
        Ok(())
    }
    async fn write_file(
        &self,
        out_put: Option<String>,
        body: String,
    ) -> Result<(), ConfigParseError> {
        match out_put {
            Some(file_path) => match tokio::fs::File::open(file_path.as_str()).await {
                Result::Err(e) => match e.kind() {
                    tokio::io::ErrorKind::NotFound => {
                        let mut file = tokio::fs::File::create(file_path.as_str()).await?;
                        file.write_all(body.as_bytes()).await?;
                    }
                    _ => {
                        return Err(ConfigParseError::Io(
                            e.to_string() + " save in path:" + file_path.as_str(),
                        ));
                    }
                },
                Result::Ok(_) => {
                    if self.outfile_overwrite {
                        let mut file = tokio::fs::File::create(file_path.as_str()).await.unwrap();
                        file.write_all(body.as_bytes()).await.map_err(|e| {
                            ConfigParseError::Io(
                                e.to_string() + " write in file:" + file_path.as_str(),
                            )
                        })?;
                        file.sync_all().await.map_err(|e| {
                            ConfigParseError::Io(
                                e.to_string() + " sync in file:" + file_path.as_str(),
                            )
                        })?;
                    }
                }
            },
            _ => {
                println!("{}", body);
            }
        }
        Ok(())
    }
    pub fn parse(settings: &Config) -> Result<Self, ConfigParseError> {
        let db_url = settings.get_string("db_url")?;
        let tpl_body = settings.get_string("tpl_body")?;
        let outfile_name = settings.get_string("outfile_name").unwrap_or_default();
        let outfile_merge = settings.get_bool("outfile_merge").unwrap_or(false);
        let outfile_merge_tpl = settings.get_string("outfile_merge_tpl").unwrap_or_default();
        let outfile_overwrite = settings.get_bool("outfile_overwrite").unwrap_or(false);
        let type_transform = settings.get_bool("type_transform").unwrap_or(true);
        let type_defalut = settings.get_string("type_default").unwrap_or_default();
        let model_name_rule = settings.get_string("model_name_rule").unwrap_or_default();
        let model_name_start_replace = settings
            .get_string("model_name_start_replace")
            .unwrap_or_default();
        let model_name_end_replace = settings
            .get_string("model_name_end_replace")
            .unwrap_or_default();
        let table_name_start_replace = settings
            .get_string("table_name_start_replace")
            .unwrap_or_default();
        let table_name_end_replace = settings
            .get_string("table_name_end_replace")
            .unwrap_or_default();
        let column_name_rule = settings.get_string("column_name_rule").unwrap_or_default();
        let column_name_start_replace = settings
            .get_string("column_name_start_replace")
            .unwrap_or_default();
        let column_name_end_replace = settings
            .get_string("column_name_end_replace")
            .unwrap_or_default();
        let outfile_name_rule = settings.get_string("outfile_name_rule").unwrap_or_default();
        let outfile_name_start_replace = settings
            .get_string("outfile_name_start_replace")
            .unwrap_or_default();
        let outfile_name_end_replace = settings
            .get_string("outfile_name_end_replace")
            .unwrap_or_default();

        let mut type_map = vec![];
        if let Result::Ok(map_arr) = settings.get_table("type_map") {
            for (_, map_list) in map_arr
                .into_iter()
                .map(|(i, c)| (-i.parse::<i32>().unwrap_or(0), c))
                .collect::<BTreeMap<_, _>>()
            {
                for (out_type, val) in map_list.into_table()?.into_iter() {
                    let mut tmap = vec![];
                    for aval in val.into_array()? {
                        tmap.push(aval.into_string()?);
                    }
                    type_map.push((out_type, tmap));
                }
            }
        }

        let default_none = settings.get_string("default_none").unwrap_or_default();
        let default_null = settings.get_string("default_null").unwrap_or_default();

        let mut defalut_map = vec![];
        if let Result::Ok(set_defalut_map) = settings.get_table("default_map") {
            for (_, map_list) in set_defalut_map
                .into_iter()
                .map(|(i, c)| (-i.parse::<i32>().unwrap_or(0), c))
                .collect::<BTreeMap<_, _>>()
            {
                for (key, val) in map_list.into_table()?.into_iter() {
                    let out_type = val.into_string()?;
                    defalut_map.push((key, out_type));
                }
            }
        }
        let mut table_rule = None;
        let mut tables = vec![];
        if let Result::Ok(table) = settings.get_string("tables") {
            table_rule = Some(table);
        } else {
            let table = settings.get_array("tables")?;
            for map in table.into_iter() {
                tables.push(map.into_string()?);
            }
        }
        let config_parse = ConfigParse {
            outfile_name,
            outfile_merge,
            outfile_merge_tpl,
            outfile_overwrite,
            tpl_body,
            type_defalut,
            type_map,
            default_none,
            type_transform,
            default_null,
            defalut_map,
            table_name_rule: (table_name_start_replace, table_name_end_replace),
            model_name_rule: (
                model_name_rule,
                model_name_start_replace,
                model_name_end_replace,
            ),
            column_name_rule: (
                column_name_rule,
                column_name_start_replace,
                column_name_end_replace,
            ),
            outfile_name_rule: (
                outfile_name_rule,
                outfile_name_start_replace,
                outfile_name_end_replace,
            ),
            uri: db_url,
            table_rule,
            tables,
        };
        Ok(config_parse)
    }
    fn parse_column_type(&self, db_field: &str) -> Result<String, ConfigParseError> {
        for (out_type, out_reg) in self.type_map.iter() {
            for reg_str in out_reg {
                let regex = Regex::new(reg_str)?;
                if regex.is_match(db_field) {
                    return Ok(out_type.to_owned());
                }
            }
        }
        Ok(self.type_defalut.clone())
    }
    fn replace_name(&self, rule: (&String, &String), table_name: &String) -> String {
        let mut out_name = table_name.to_owned();
        if !rule.0.is_empty() {
            if let Some(data) = table_name.strip_prefix(rule.0.as_str()) {
                out_name = data.to_owned();
            }
        }
        if !rule.1.is_empty() {
            if let Some(data) = table_name.strip_prefix(rule.1.as_str()) {
                out_name = data.to_owned();
            }
        }
        out_name
    }
    fn parse_name(&self, rule: &(String, String, String), table_name: &String) -> String {
        let out_name = self.replace_name((&rule.1, &rule.2), table_name);
        match rule.0.as_str() {
            "lower" => out_name.to_lowercase(),
            "snake" => out_name.to_snake_case(),
            "upper" => out_name.to_uppercase(),
            "shouty_snake" => out_name.to_shouty_snake_case(),
            "shouty_kebab" => out_name.to_shouty_kebab_case(),
            "kebab" => out_name.to_kebab_case(),
            "upper_camel" => out_name.to_upper_camel_case(),
            "lower_camel" => out_name.to_lower_camel_case(),
            _ => out_name,
        }
    }
    pub async fn render_data(
        &self,
        name: RenderTableInfo,
        columndata: DataValue,
    ) -> Result<RenderBody, ConfigParseError> {
        let mut field_datas = vec![];
        for field in columndata.columns {
            let column_name = self.parse_name(&self.column_name_rule, &field.field_name);
            let ty = if self.type_transform {
                self.parse_column_type(field.type_name.as_str())?
            } else {
                field.type_name
            };
            let def = match field.default {
                Some(mut ts) => {
                    for (in_reg, out_data) in self.defalut_map.iter() {
                        let reg = Regex::new(in_reg)?;
                        if reg.is_match(ts.as_str()) {
                            ts = reg
                                .replace(ts.as_str(), regex::NoExpand(out_data))
                                .to_string();
                        }
                    }
                    ts
                }
                None => {
                    if field.is_null {
                        self.default_null.clone()
                    } else {
                        self.default_none.clone()
                    }
                }
            };
            let field_data = RenderField {
                field_name: field.field_name,
                column_name,
                type_name: ty,
                is_null: field.is_null,
                default: def,
                is_pk: field.is_pk,
                comment: field.comment,
            };
            field_datas.push(field_data);
        }
        Ok(RenderBody {
            table: name,
            field_data: field_datas,
        })
    }
}
