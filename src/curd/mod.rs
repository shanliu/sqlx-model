#[macro_use]
mod macros;
mod delete;
mod insert;
mod select;
mod update;

use sqlx::database::HasArguments;
use sqlx::query::{Query, QueryAs};
use sqlx::{Database, FromRow};
use std::any::TypeId;
use std::fmt::Display;

/// 统一表前缀
static mut TABLE_PREFIX: String = String::new();
/// 表名
pub struct TableName {
    db: String,
    name: String,
}
impl Display for TableName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe { write!(f, "{}{}{}", self.db, TABLE_PREFIX, self.name) }
    }
}
impl SqlQuote<String> for TableName {
    fn sql_quote(&self) -> String {
        unsafe { format!("{}{}{}", self.db, TABLE_PREFIX, self.name) }
    }
}
impl TableName {
    /// 设置表前缀
    pub fn set_prefix(str: String) {
        unsafe {
            TABLE_PREFIX = str;
        }
    }
    /// 新建表名
    pub fn new(name: &str) -> Self {
        let (db, name) = match name.rfind('.') {
            Some(index) => name.split_at(index + 1),
            None => ("", name),
        };
        Self {
            db: db.to_string(),
            name: name.to_owned(),
        }
    }
    /// 得到完整表名
    pub fn full_name(&self) -> String {
        unsafe { format!("{}{}{}", self.db, TABLE_PREFIX, self.name) }
    }
}
#[derive(PartialEq, Eq)]
pub enum DbType {
    Mysql,
    Sqlite,
    Postgres,
    MsSql,
}
impl DbType {
    pub fn type_new<DB: sqlx::Database>() -> Self {
        #[cfg(feature = "sqlx-mysql")]
        if TypeId::of::<DB>() == TypeId::of::<sqlx::MySql>() {
            return DbType::Mysql;
        }
        #[cfg(feature = "sqlx-sqlite")]
        if TypeId::of::<DB>() == TypeId::of::<sqlx::Sqlite>() {
            return DbType::Mysql;
        }
        #[cfg(feature = "sqlx-postgres")]
        if TypeId::of::<DB>() == TypeId::of::<sqlx::Postgres>() {
            return DbType::Postgres;
        }
        #[cfg(feature = "sqlx-mssql")]
        if TypeId::of::<DB>() == TypeId::of::<sqlx::Mssql>() {
            return DbType::MsSql;
        }
        unimplemented!()
    }
    /// 得到不同数据库的绑定字符
    pub fn mark(&self, pos: usize) -> String {
        match self {
            DbType::Mysql => "?".to_string(),
            DbType::Sqlite => {
                format!("${pos}")
            }
            DbType::Postgres => {
                format!("${}", pos + 1)
            }
            DbType::MsSql => "?".to_string(),
        }
    }
}

/// model实现得到表名trait
pub trait ModelTableName {
    fn table_name() -> TableName;
}
/// model实现得到表字段和字段值绑定 trait
pub trait ModelTableField<DB>
where
    DB: Database,
{
    fn table_pk() -> TableFields;
    fn table_column() -> TableFields;
    fn query_sqlx_bind<'t>(
        &'t self,
        table_field_val: &FieldItem,
        res: Query<'t, DB, <DB as HasArguments<'t>>::Arguments>,
    ) -> Query<'t, DB, <DB as HasArguments<'t>>::Arguments>;
    fn query_as_sqlx_bind<'t, M>(
        &'t self,
        table_field_val: &FieldItem,
        res: QueryAs<'t, DB, M, <DB as HasArguments<'t>>::Arguments>,
    ) -> QueryAs<'t, DB, M, <DB as HasArguments<'t>>::Arguments>
    where
        for<'r> M: FromRow<'r, DB::Row> + Send + Unpin;
}

/// 表字段
#[derive(Clone, PartialEq, Eq)]
pub struct FieldItem {
    pub name: String,
    pub column_name: String,
}
impl Display for FieldItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
impl FieldItem {
    pub fn new(name: &str, column_name: &str) -> Self {
        FieldItem {
            name: name.to_string(),
            column_name: column_name.to_string(),
        }
    }
}

/// 表字段容器
pub struct TableFields(Vec<FieldItem>);
impl Display for TableFields {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fileds = self
            .0
            .iter()
            .map(|e| format!("{e}"))
            .collect::<Vec<String>>()
            .join(",");
        write!(f, "{fileds}")
    }
}
impl TableFields {
    pub fn new(fields: Vec<FieldItem>) -> Self {
        TableFields(fields)
    }
    /// 合并一批外部的表字段列表,去除重复
    pub fn marge(&mut self, field: &[FieldItem]) {
        for val in field.iter() {
            if !self.0.iter().any(|e| e.name == val.name) {
                self.0.push(val.to_owned())
            }
        }
    }
    /// 跟指定表字段列表取并集
    pub fn intersect(&mut self, field: &[FieldItem]) {
        self.0 = self
            .0
            .iter()
            .filter_map(|e| {
                if field.contains(e) {
                    Some(e.to_owned())
                } else {
                    None
                }
            })
            .collect();
    }
    /// 删除表指定字段
    pub fn del(&mut self, name: &str) {
        self.0 = self
            .0
            .iter()
            .filter_map(|e| {
                if name == e.name {
                    None
                } else {
                    Some(e.to_owned())
                }
            })
            .collect();
    }
    /// 得到字段列表
    pub fn to_vec(&self) -> Vec<String> {
        let field = self.0.iter();
        field
            .map(|e| e.column_name.clone())
            .collect::<Vec<String>>()
    }
}

pub enum WhereOption {
    None,            //无WHERE条件,且无排序等
    Where(String),   //有WHERE条件
    NoWhere(String), //无WHERE条件,但有排序等后续SQL
}

pub use delete::*;
pub use insert::*;
pub use select::*;
pub use update::*;

use crate::SqlQuote;
