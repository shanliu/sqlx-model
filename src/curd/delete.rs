use sqlx::database::HasArguments;
use sqlx::{Arguments, Database, Error, Executor, IntoArguments, Pool};
use sqlx::query::{Query};
use super::TableName;
use super::{DbType,  ModelTableField, ModelTableName};

/// 删除操作
pub struct Delete<DB>
    where DB:Database
{
    pub table_name:TableName,
    _marker:std::marker::PhantomData<DB>
}
impl <DB> Delete<DB>
    where DB:Database
{
    pub fn type_new<T1>() -> Delete<DB>
    where
        T1: ModelTableField<DB>+ModelTableName
    {
        Delete {
           table_name:T1::table_name(),
           _marker:Default::default()
        }
    }
    pub fn new(table_name:TableName) -> Delete<DB>
    {
        Delete {
           table_name,
           _marker:Default::default()
        }
    }
    pub async fn execute_by_where_call<'c,RB>(
        &self,
        where_sql: &str,
        where_bind: RB,
        pool:&'c Pool<DB>
    ) -> Result<<DB as Database>::QueryResult, Error>
    where
        for<'q> RB: FnOnce(
            Query<'q,DB,<DB as HasArguments<'q>>::Arguments> ,
            &'q Delete<DB>,
        ) ->Query<'q,DB,<DB as HasArguments<'q>>::Arguments>,
        for<'n> <DB as HasArguments<'n>>::Arguments:
            Arguments<'n>+IntoArguments<'n,DB>,
        &'c Pool<DB>: Executor<'c, Database = DB>
    {
        let sql = format!("DELETE FROM {} WHERE {}", self.table_name.full_name(), where_sql);
        let mut res = sqlx::query(sql.as_str());
        res = where_bind(res,self);
        res.execute(pool).await
    }
    pub async fn execute_by_where<'c>(
        &self,
        where_sql: Option<String>,
        pool:&'c Pool<DB>
    ) -> Result<<DB as Database>::QueryResult, Error>
    where
        for<'n> <DB as HasArguments<'n>>::Arguments:
            Arguments<'n>+IntoArguments<'n,DB>,
        &'c Pool<DB>: Executor<'c, Database = DB>
    {
        let sql;
        match where_sql {
            Some(wsql)=>{
                sql = format!("DELETE FROM {} WHERE {}", self.table_name.full_name(), wsql);
            }
            None=>{
                sql = format!("DELETE FROM {} ", self.table_name.full_name());
            }
        }
        let res = sqlx::query(sql.as_str());
        res.execute(pool).await
    }
}

macro_rules! delete_execute_by {
    ($bind_type:ty) => {
        impl Delete<$bind_type> {
            pub async fn execute_by_pk<'c,T>(&self, source: &T,pool:&'c Pool<$bind_type>) -> Result<<$bind_type as Database>::QueryResult, Error>
            where
                T: ModelTableField<$bind_type>
            {
                let pkf = T::table_pk();
                let mut where_sql = vec![];
                for (pos, val) in pkf.0.iter().enumerate() {
                    let bst = DbType::type_new::<$bind_type>().mark(pos);
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
                res.execute(pool).await
            }
            pub async fn execute_by_scalar_pk<'c,T,PT>(&self, pk_scalar: PT,pool:&'c Pool<$bind_type>) -> Result<<$bind_type as Database>::QueryResult, Error>
                where 
                    T:ModelTableField<$bind_type>,
                    for<'q> PT:'q+ Send + sqlx::Encode<'q, $bind_type> + sqlx::Type<$bind_type>
        
            {
                let sql = format!(
                    "DELETE FROM {} WHERE {}",
                    self.table_name.full_name(),
                    scalar_pk_where!($bind_type,T::table_pk())
                );
                let mut res = sqlx::query(sql.as_str());
                res=res.bind(pk_scalar);
                res.execute(pool).await
            }
        }        
    };
}

#[cfg(feature = "sqlx-mysql")]
delete_execute_by!(sqlx::MySql);

#[cfg(feature = "sqlx-sqlite")]
delete_execute_by!(sqlx::Sqlite);

#[cfg(feature = "sqlx-postgres")]
delete_execute_by!(sqlx::Postgres);



#[cfg(feature = "sqlx-mssql")]
delete_execute_by!(sqlx::MsSql);