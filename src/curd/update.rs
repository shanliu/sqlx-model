use super::{DbType, FieldItem, ModelTableField, ModelTableName};
use sqlx::database::HasArguments;
use sqlx::query::Query;
use sqlx::{Arguments, Executor, IntoArguments};
use sqlx::{Database, Error};

pub trait UpdateData<'t, DB>
where
    DB: Database,
{
    fn diff_columns(&self) -> Vec<FieldItem>;
    fn sqlx_bind<'q>(
        &'q self,
        res: Query<'q, DB, <DB as HasArguments>::Arguments>,
    ) -> Query<'q, DB, <DB as HasArguments>::Arguments>;
    fn sqlx_string(&self, field: &FieldItem) -> Option<String>;
}
//得到需要更改的设置数据集
pub trait ModelUpdateData<'t, DB, CT>: ModelTableField<DB> + ModelTableName
where
    CT: UpdateData<'t, DB>,
    DB: Database,
{
    fn diff(&'t self, source: Option<&Self>) -> CT;
}

/// 更新操作
pub struct Update<'t, DB, T, CT>
where
    T: ModelUpdateData<'t, DB, CT>,
    CT: UpdateData<'t, DB>,
    DB: Database,
{
    pub change: CT,
    _marker: (
        std::marker::PhantomData<&'t CT>,
        std::marker::PhantomData<&'t T>,
        std::marker::PhantomData<DB>,
    ),
}
impl<'t, DB, T, CT> Update<'t, DB, T, CT>
where
    T: ModelUpdateData<'t, DB, CT>,
    CT: UpdateData<'t, DB>,
    DB: Database,
{
    pub fn new(val: CT) -> Update<'t, DB, T, CT> {
        Update {
            change: val,
            _marker: Default::default(),
        }
    }
    pub fn model<'q: 't>(val: &'q T, source: Option<&T>) -> Update<'t, DB, T, CT> {
        Update {
            change: val.diff(source),
            _marker: Default::default(),
        }
    }
    pub fn empty_change(&self) -> bool {
        self.change.diff_columns().is_empty()
    }
    pub fn sql_sets(&self) -> String {
        let diff = self.change.diff_columns();
        let mut values = Vec::<String>::with_capacity(diff.len());
        for (pos, val) in diff.iter().enumerate() {
            let bst = DbType::type_new::<DB>().mark(pos);
            values.push(format!("{}={}", val.name, bst));
        }
        values.join(",")
    }
    pub fn sql_values_sets(&self) -> String {
        let diff = self.change.diff_columns();
        let mut values = Vec::<String>::with_capacity(diff.len());
        for val in diff.iter() {
            if let Some(setval) = self.change.sqlx_string(val) {
                values.push(format!("{}={}", val.name, setval));
            }
        }
        values.join(",")
    }
    pub fn bind_values<'q>(
        &'q self,
        res: Query<'q, DB, <DB as HasArguments>::Arguments>,
    ) -> Query<'q, DB, <DB as HasArguments>::Arguments> {
        self.change.sqlx_bind(res)
    }
    pub async fn execute_by_scalar_pk<'c, PT, E>(
        &self,
        pk_scalar: PT,
        executor: E,
    ) -> Result<<DB as Database>::QueryResult, Error>
    where
        for<'q> PT: 'q + Send + sqlx::Encode<'q, DB> + sqlx::Type<DB>,
        for<'n> <DB as HasArguments<'n>>::Arguments: Arguments<'n> + IntoArguments<'n, DB>,
        E: Executor<'c, Database = DB>,
    {
        if self.empty_change() {
            return Ok(<DB as Database>::QueryResult::default());
        }
        let table = T::table_name();
        let values = self.sql_sets();
        let sql = format!(
            "UPDATE {} SET {} WHERE {}",
            table.full_name(),
            values,
            scalar_pk_where!(DB, T::table_pk())
        );
        let mut res = sqlx::query(sql.as_str());
        res = self.bind_values(res);
        res = res.bind(pk_scalar);
        executor.execute(res).await
    }
    pub async fn execute_by_where_call<'c, RB, E>(
        &self,
        where_sql: &str,
        where_bind: RB,
        executor: E,
    ) -> Result<<DB as Database>::QueryResult, Error>
    where
        for<'q> RB: FnOnce(
            Query<'q, DB, <DB as HasArguments>::Arguments>,
            &'q Update<DB, T, CT>,
        ) -> Query<'q, DB, <DB as HasArguments<'q>>::Arguments>,
        for<'n> <DB as HasArguments<'n>>::Arguments: Arguments<'n> + IntoArguments<'n, DB>,
        E: Executor<'c, Database = DB>,
    {
        if self.empty_change() {
            return Ok(<DB as Database>::QueryResult::default());
        }
        let table = T::table_name();
        let values = self.sql_sets();
        let sql = format!(
            "UPDATE {} SET {} WHERE {}",
            table.full_name(),
            values,
            where_sql
        );
        let mut res = sqlx::query(sql.as_str());
        res = self.bind_values(res);
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
        if self.empty_change() {
            return Ok(<DB as Database>::QueryResult::default());
        }
        let table = T::table_name();
        let values = self.sql_sets();
        let sql = match where_sql {
            Some(wsql) => {
                format!("UPDATE {} SET {} WHERE {}", table.full_name(), values, wsql)
            }
            None => {
                format!("UPDATE {} SET {}", table.full_name(), values)
            }
        };
        let mut res = sqlx::query(sql.as_str());
        res = self.bind_values(res);
        executor.execute(res).await
    }
    // execute_by_sql!(Update<DB,T,CT>);
    pub async fn execute_by_pk<'c, E>(
        &self,
        source: &T,
        executor: E,
    ) -> Result<<DB as Database>::QueryResult, Error>
    where
        for<'n> <DB as HasArguments<'n>>::Arguments: Arguments<'n> + IntoArguments<'n, DB>,
        E: Executor<'c, Database = DB>,
    {
        if self.empty_change() {
            return Ok(<DB as Database>::QueryResult::default());
        }
        let table = T::table_name();
        let pkf = T::table_pk();
        let mut where_sql = vec![];
        for (pos, val) in pkf.0.iter().enumerate() {
            let bst = DbType::type_new::<DB>().mark(pos);
            where_sql.push(format!("{}={}", val.name, bst));
        }
        let values = self.sql_sets();
        let sql = format!(
            "UPDATE {} SET {} WHERE {}",
            table.full_name(),
            values,
            where_sql.join(" and ")
        );
        let mut res = sqlx::query(sql.as_str());
        res = self.bind_values(res);
        for val in pkf.0.iter() {
            res = source.query_sqlx_bind(val, res);
        }
        executor.execute(res).await
    }
}
