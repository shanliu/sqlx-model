use super::TableName;
use super::{DbType, ModelTableField, ModelTableName};
use sqlx::database::HasArguments;
use sqlx::query::Query;
use sqlx::{Arguments, Database, Error, Executor, IntoArguments};

/// 删除操作
pub struct Delete<DB>
where
    DB: Database,
{
    pub table_name: TableName,
    _marker: std::marker::PhantomData<DB>,
}
impl<DB> Delete<DB>
where
    DB: Database,
{
    pub fn type_new<T1>() -> Delete<DB>
    where
        T1: ModelTableField<DB> + ModelTableName,
    {
        Delete {
            table_name: T1::table_name(),
            _marker: Default::default(),
        }
    }
    pub fn new(table_name: TableName) -> Delete<DB> {
        Delete {
            table_name,
            _marker: Default::default(),
        }
    }
    pub async fn execute_by_where_call<'c, RB, E>(
        &self,
        where_sql: &str,
        where_bind: RB,
        executor: E,
    ) -> Result<<DB as Database>::QueryResult, Error>
    where
        for<'q> RB: FnOnce(
            Query<'q, DB, <DB as HasArguments<'q>>::Arguments>,
            &'q Delete<DB>,
        ) -> Query<'q, DB, <DB as HasArguments<'q>>::Arguments>,
        for<'n> <DB as HasArguments<'n>>::Arguments: Arguments<'n> + IntoArguments<'n, DB>,
        E: Executor<'c, Database = DB>,
    {
        let sql = format!(
            "DELETE FROM {} WHERE {}",
            self.table_name.full_name(),
            where_sql
        );
        let mut res = sqlx::query(sql.as_str());
        res = where_bind(res, self);
        executor.execute(res).await
    }
    pub async fn execute_by_where<'c, E>(
        &self,
        where_sql: Option<String>,
        executor: E,
    ) -> Result<<DB as Database>::QueryResult, Error>
    where
        for<'n> <DB as HasArguments<'n>>::Arguments: Arguments<'n> + IntoArguments<'n, DB>,
        E: Executor<'c, Database = DB>,
    {
        let sql = match where_sql {
            Some(wsql) => {
                format!("DELETE FROM {} WHERE {}", self.table_name.full_name(), wsql)
            }
            None => {
                format!("DELETE FROM {} ", self.table_name.full_name())
            }
        };
        let res = sqlx::query(sql.as_str());
        executor.execute(res).await
    }
    pub async fn execute_by_pk<'c, T, E>(
        &self,
        source: &T,
        executor: E,
    ) -> Result<<DB as Database>::QueryResult, Error>
    where
        for<'n> <DB as HasArguments<'n>>::Arguments: Arguments<'n> + IntoArguments<'n, DB>,
        T: ModelTableField<DB>,
        E: Executor<'c, Database = DB>,
    {
        let pkf = T::table_pk();
        let mut where_sql = vec![];
        for (pos, val) in pkf.0.iter().enumerate() {
            let bst = DbType::type_new::<DB>().mark(pos);
            where_sql.push(format!("{}={}", val.name, bst));
        }
        let sql = format!(
            "DELETE FROM {} WHERE {}",
            self.table_name.full_name(),
            where_sql.join(" and ")
        );
        let mut res = sqlx::query(sql.as_str());
        for val in pkf.0.iter() {
            res = source.query_sqlx_bind(val, res);
        }
        executor.execute(res).await
    }
    pub async fn execute_by_scalar_pk<'c, T, PT, E>(
        &self,
        pk_scalar: PT,
        executor: E,
    ) -> Result<<DB as Database>::QueryResult, Error>
    where
        for<'n> <DB as HasArguments<'n>>::Arguments: Arguments<'n> + IntoArguments<'n, DB>,
        T: ModelTableField<DB>,
        for<'q> PT: 'q + Send + sqlx::Encode<'q, DB> + sqlx::Type<DB>,
        E: Executor<'c, Database = DB>,
    {
        let sql = format!(
            "DELETE FROM {} WHERE {}",
            self.table_name.full_name(),
            scalar_pk_where!(DB, T::table_pk())
        );
        let mut res = sqlx::query(sql.as_str());
        res = res.bind(pk_scalar);
        executor.execute(res).await
    }
}
