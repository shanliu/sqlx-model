use super::TableName;
use super::{DbType, ModelTableField, ModelTableName, TableFields, WhereOption};
use sqlx::query::{QueryAs, QueryScalar};
use sqlx::{database::HasArguments, Database, Error, FromRow};
use sqlx::{Arguments, Executor, IntoArguments};
use std::vec;

macro_rules! fetch_by_where_call {
    ($self_var:ident,$bind_type:ty,$name:ident,$sql:literal,$query_type:ident,$fetch_type:ident,$out_type:ty)=>{
        impl Select<$bind_type>
        {
            /// M 为 Model  类型
            pub async fn $name<'c,M,RB,E>(
                &$self_var,
                where_sql:&str,
                where_bind:RB,
                executor:E
            )
                ->Result<$out_type,Error>
                where
                for<'t> RB: FnOnce( QueryAs<'t,$bind_type,M,<$bind_type as HasArguments>::Arguments>,&'t Select<$bind_type>) -> QueryAs<'t,$bind_type,M,<$bind_type as HasArguments<'t>>::Arguments>,
                for<'r> M:  FromRow<'r, <$bind_type as Database>::Row>+Send+Unpin+ModelTableField<$bind_type>,
                for<'n> <$bind_type as HasArguments<'n>>::Arguments:
                    Arguments<'n>+IntoArguments<'n,$bind_type>,
                E: Executor<'c, Database = $bind_type>
            {
                let sql=format!(
                    $sql,
                    $self_var.table_field.to_vec().join(","),
                    $self_var.table_name.full_name(),
                    where_sql
                );
                let mut res=sqlx::$query_type::<$bind_type, M>(sql.as_str());
                res=where_bind(res,&$self_var);
                res.$fetch_type(executor).await
            }
        }
    };
    ($bind_type:ty,$name:ident,$sql:literal,$query_type:ident,$fetch_type:ident,$out_type:ty)=>{
        fetch_by_where_call!(self,$bind_type,$name,$sql,$query_type,$fetch_type,$out_type);
    };
}

macro_rules! fetch_by_where {
    ($self_var:ident,$bind_type:ty,$name:ident,$sql:literal,$where_sql:literal,$query_type:ident,$fetch_type:ident,$out_type:ty)=>{
        impl Select<$bind_type>
        {
            /// M 为 Model  类型
            pub async fn $name<'c,M,E>(
                &$self_var,
                where_sql:&WhereOption,
                executor:E
            )
                ->Result<$out_type,Error>
                where
                for<'r> M:  FromRow<'r, <$bind_type as Database>::Row>+Send+Unpin+ModelTableField<$bind_type>,
                for<'n> <$bind_type as HasArguments<'n>>::Arguments:
                    Arguments<'n>+IntoArguments<'n,$bind_type>,
                E: Executor<'c, Database = $bind_type>
            {
                let sql=match where_sql {
                    WhereOption::NoWhere(other)=>{
                        format!(
                            "{} {}",
                            format!(
                                $sql,
                                $self_var.table_field.to_vec().join(","),
                                $self_var.table_name.full_name(),
                            ),
                            other
                        )
                    }

                    WhereOption::Where(wsql)=>{
                       format!(
                            $where_sql,
                            $self_var.table_field.to_vec().join(","),
                            $self_var.table_name.full_name(),
                            wsql
                        )
                    }
                    WhereOption::None=>{
                        format!(
                            $sql,
                            $self_var.table_field.to_vec().join(","),
                            $self_var.table_name.full_name()
                        )
                    }
                };
                let res=sqlx::$query_type::<$bind_type, M>(sql.as_str());
                res.$fetch_type(executor).await
            }
        }
    };
    ($bind_type:ty,$name:ident,$sql:literal,$where_sql:literal,$query_type:ident,$fetch_type:ident,$out_type:ty)=>{
        fetch_by_where!(self,$bind_type,$name,$sql,$where_sql,$query_type,$fetch_type,$out_type);
    };
}

macro_rules! fetch_by_where_scalar_call {
    ($self_var:ident,$bind_type:ty,$name:ident,$sql:literal,$fetch_type:ident,$out_type:ty)=>{
        impl Select<$bind_type>
        {
             /// M 为 返回某字段类型
            pub async fn $name<'c,M, RB,E>(
                &$self_var,
                field_name:&str,
                where_sql: &str,
                where_bind: RB,
                executor:E
            )
                -> Result<$out_type, Error>
                where
                        for<'t> RB: FnOnce(QueryScalar<'t,$bind_type,M,<$bind_type as HasArguments>::Arguments>,&'t Select<$bind_type>) ->QueryScalar<'t,$bind_type,M,<$bind_type as HasArguments<'t>>::Arguments>,
                        (M,): for<'r> FromRow<'r, <$bind_type as Database>::Row> + Send + Unpin,
                        M:Send + Unpin,
                        for<'n> <$bind_type as HasArguments<'n>>::Arguments:
                            Arguments<'n>+IntoArguments<'n,$bind_type>,
                        E: Executor<'c, Database = $bind_type>
                {
                let sql = format!(
                    $sql,
                    field_name,
                    $self_var.table_name.full_name(),
                    where_sql
                );
                let mut res = sqlx::query_scalar::<$bind_type, M,>(sql.as_str());
                res = where_bind(res,&$self_var);
                res.$fetch_type(executor).await
            }
        }
    };
    ($bind_type:ty,$name:ident,$sql:literal,$fetch_type:ident,$out_type:ty)=>{
        fetch_by_where_scalar_call!(self,$bind_type,$name,$sql,$fetch_type,$out_type);
    };
}

macro_rules! fetch_by_where_scalar {
    ($self_var:ident,$bind_type:ty,$name:ident,$sql:literal,$where_sql:literal,$fetch_type:ident,$out_type:ty)=>{
        impl Select<$bind_type>
        {
             /// M 为 返回某字段类型
            pub async fn $name<'c,M,E>(
                &$self_var,
                field_name:&str,
                where_sql:&WhereOption,
                executor:E
            )
                -> Result<$out_type, Error>
                where
                        (M,): for<'r> FromRow<'r, <$bind_type as Database>::Row> + Send + Unpin,
                        M:Send + Unpin,
                        for<'n> <$bind_type as HasArguments<'n>>::Arguments:
                            Arguments<'n>+IntoArguments<'n,$bind_type>,
                        E: Executor<'c, Database = $bind_type>
                {


                    let sql=match where_sql {
                        WhereOption::NoWhere(other)=>{
                            format!(
                                "{} {}",
                                format!(
                                    $sql,
                                    field_name,
                                    $self_var.table_name.full_name()
                                ),
                                other
                            )
                        }

                        WhereOption::Where(wsql)=>{
                            format!(
                                $where_sql,
                                field_name,
                                $self_var.table_name.full_name(),
                                wsql
                            )
                        }
                        WhereOption::None=>{
                           format!(
                                $sql,
                                field_name,
                                $self_var.table_name.full_name()
                            )
                        }
                    };
                let res = sqlx::query_scalar::<$bind_type, M,>(sql.as_str());
                res.$fetch_type(executor).await
            }
        }
    };
    ($bind_type:ty,$name:ident,$sql:literal,$where_sql:literal,$fetch_type:ident,$out_type:ty)=>{
        fetch_by_where_scalar!(self,$bind_type,$name,$sql,$where_sql,$fetch_type,$out_type);
    };
}

/// 查询操作
pub struct Select<DB>
where
    DB: Database,
{
    pub table_name: TableName,
    pub table_field: TableFields,
    pub table_pk: TableFields,
    _marker: std::marker::PhantomData<DB>,
}
impl<DB> Select<DB>
where
    DB: Database,
{
    pub fn new(table_name: TableName, table_field: TableFields, table_pk: TableFields) -> Self {
        Select {
            table_name,
            table_field,
            table_pk,
            _marker: Default::default(),
        }
    }
    pub fn type_new<T>() -> Self
    where
        T: ModelTableField<DB> + ModelTableName,
    {
        Select {
            table_name: T::table_name(),
            table_field: T::table_column(),
            table_pk: T::table_pk(),
            _marker: Default::default(),
        }
    }
    /// 非联合主键的表通过主键值查找某记录
    /// @field_name 需要获取的字段名
    /// @pk_scalar 主键值
    /// @executor Executor
    /// M 为 Model 类型
    pub async fn fetch_one_by_scalar_pk<'c, M, PT, E>(
        &self,
        pk_scalar: PT,
        executor: E,
    ) -> Result<M, Error>
    where
        for<'q> PT: 'q + Send + sqlx::Encode<'q, DB> + sqlx::Type<DB>,
        for<'r> M: FromRow<'r, DB::Row> + Send + Unpin + ModelTableField<DB>,
        for<'n> <DB as HasArguments<'n>>::Arguments: Arguments<'n> + IntoArguments<'n, DB>,
        E: Executor<'c, Database = DB>,
    {
        let where_sql = scalar_pk_where!(DB, self.table_pk);
        let sql = format!(
            "SELECT {} FROM {} WHERE {}",
            self.table_field.to_vec().join(","),
            self.table_name.full_name(),
            where_sql
        );
        let mut res = sqlx::query_as::<DB, M>(sql.as_str());
        res = res.bind(pk_scalar);
        res.fetch_one(executor).await
    }
    /// 非联合主键的表通过主键值查找某字段值
    /// @field_name 需要获取的字段名
    /// @pk_scalar 主键值
    /// @executor Executor
    /// M 为 @field_name 字段类型
    pub async fn fetch_one_scalar_by_scalar_pk<'c, M, PT, E>(
        &self,
        field_name: &str,
        pk_scalar: PT,
        executor: E,
    ) -> Result<M, Error>
    where
        for<'q> PT: 'q + Send + sqlx::Encode<'q, DB> + sqlx::Type<DB>,
        (M,): for<'r> FromRow<'r, DB::Row> + Send + Unpin,
        M: Send + Unpin,
        for<'n> <DB as HasArguments<'n>>::Arguments: Arguments<'n> + IntoArguments<'n, DB>,
        E: Executor<'c, Database = DB>,
    {
        let where_sql = scalar_pk_where!(DB, self.table_pk);
        let sql = format!(
            "SELECT {} FROM {} WHERE {}",
            field_name,
            self.table_name.full_name(),
            where_sql
        );
        let mut res = sqlx::query_scalar::<DB, M>(sql.as_str());
        res = res.bind(pk_scalar);
        res.fetch_one(executor).await
    }
    /// 从DB中重新加载Model里值
    /// @val Model 变量
    /// @executor Executor
    pub async fn reload<'c, M, E>(&self, val: &M, executor: E) -> Result<M, Error>
    where
        M: for<'r> FromRow<'r, DB::Row> + Send + Unpin + ModelTableField<DB>,
        for<'n> <DB as HasArguments<'n>>::Arguments: Arguments<'n> + IntoArguments<'n, DB>,
        E: Executor<'c, Database = DB>,
    {
        let pkf = M::table_pk();
        let mut where_sql = vec![];
        for (pos, val) in pkf.0.iter().enumerate() {
            let bst = DbType::type_new::<DB>().mark(pos);
            where_sql.push(format!("{}={}", val.name, bst));
        }
        let sql = format!(
            "SELECT {} FROM {} WHERE {}",
            self.table_field.to_vec().join(","),
            self.table_name.full_name(),
            where_sql.join(" and ")
        );
        let mut res = sqlx::query_as::<DB, M>(sql.as_str());
        for fval in pkf.0.iter() {
            res = val.query_as_sqlx_bind(fval, res);
        }
        res.fetch_one(executor).await
    }
}

#[cfg(feature = "sqlx-mysql")]
fetch_by_where!(
    sqlx::MySql,
    fetch_all_by_where,
    "SELECT {} FROM {} ",
    "SELECT {} FROM {} WHERE {} ",
    query_as,
    fetch_all,
    Vec<M>
);
#[cfg(feature = "sqlx-mysql")]
fetch_by_where!(
    sqlx::MySql,
    fetch_one_by_where,
    "SELECT {} FROM {} LIMIT 1",
    "SELECT {} FROM {} WHERE {} LIMIT 1",
    query_as,
    fetch_one,
    M
);
#[cfg(feature = "sqlx-sqlite")]
fetch_by_where!(
    sqlx::Sqlite,
    fetch_all_by_where,
    "SELECT {} FROM {} ",
    "SELECT {} FROM {} WHERE {} ",
    query_as,
    fetch_all,
    Vec<M>
);
#[cfg(feature = "sqlx-sqlite")]
fetch_by_where!(
    sqlx::Sqlite,
    fetch_one_by_where,
    "SELECT {} FROM {} LIMIT 1",
    "SELECT {} FROM {} WHERE {} LIMIT 1",
    query_as,
    fetch_one,
    M
);
#[cfg(feature = "sqlx-postgres")]
fetch_by_where!(
    sqlx::Postgres,
    fetch_all_by_where,
    "SELECT {} FROM {} ",
    "SELECT {} FROM {} WHERE {} ",
    query_as,
    fetch_all,
    Vec<M>
);
#[cfg(feature = "sqlx-postgres")]
fetch_by_where!(
    sqlx::Postgres,
    fetch_one_by_where,
    "SELECT {} FROM {} LIMIT 1",
    "SELECT {} FROM {} WHERE {} LIMIT 1",
    query_as,
    fetch_one,
    M
);
#[cfg(feature = "sqlx-mssql")]
fetch_by_where!(
    sqlx::Mssql,
    fetch_all_by_where,
    "SELECT {} FROM {} ",
    "SELECT {} FROM {} WHERE {} ",
    query_as,
    fetch_all,
    Vec<M>
);
#[cfg(feature = "sqlx-mssql")]
fetch_by_where!(
    sqlx::Mssql,
    fetch_one_by_where,
    "SELECT  TOP 1 {} FROM {}",
    "SELECT  TOP 1 {} FROM {} WHERE {}",
    query_as,
    fetch_one,
    M
);

#[cfg(feature = "sqlx-mysql")]
fetch_by_where_call!(
    sqlx::MySql,
    fetch_all_by_where_call,
    "SELECT {} FROM {} WHERE {}",
    query_as,
    fetch_all,
    Vec<M>
);
#[cfg(feature = "sqlx-mysql")]
fetch_by_where_call!(
    sqlx::MySql,
    fetch_one_by_where_call,
    "SELECT {} FROM {} WHERE {} LIMIT 1",
    query_as,
    fetch_one,
    M
);
#[cfg(feature = "sqlx-sqlite")]
fetch_by_where_call!(
    sqlx::Sqlite,
    fetch_all_by_where_call,
    "SELECT {} FROM {} WHERE {}",
    query_as,
    fetch_all,
    Vec<M>
);
#[cfg(feature = "sqlx-sqlite")]
fetch_by_where_call!(
    sqlx::Sqlite,
    fetch_one_by_where_call,
    "SELECT {} FROM {} WHERE {} LIMIT 1",
    query_as,
    fetch_one,
    M
);
#[cfg(feature = "sqlx-postgres")]
fetch_by_where_call!(
    sqlx::Postgres,
    fetch_all_by_where_call,
    "SELECT {} FROM {} WHERE {}",
    query_as,
    fetch_all,
    Vec<M>
);
#[cfg(feature = "sqlx-postgres")]
fetch_by_where_call!(
    sqlx::Postgres,
    fetch_one_by_where_call,
    "SELECT {} FROM {} WHERE {} LIMIT 1",
    query_as,
    fetch_one,
    M
);
#[cfg(feature = "sqlx-mssql")]
fetch_by_where_call!(
    sqlx::Mssql,
    fetch_all_by_where_call,
    "SELECT {} FROM {} WHERE {}",
    query_as,
    fetch_all,
    Vec<M>
);
#[cfg(feature = "sqlx-mssql")]
fetch_by_where_call!(
    sqlx::Mssql,
    fetch_one_by_where_call,
    "SELECT TOP 1 {} FROM {} WHERE {}",
    query_as,
    fetch_one,
    M
);

#[cfg(feature = "sqlx-mysql")]
fetch_by_where_scalar!(
    sqlx::MySql,
    fetch_all_scalar_by_where,
    "SELECT {} FROM {} ",
    "SELECT {} FROM {} WHERE {} ",
    fetch_all,
    Vec<M>
);
#[cfg(feature = "sqlx-mysql")]
fetch_by_where_scalar!(
    sqlx::MySql,
    fetch_one_scalar_by_where,
    "SELECT {} FROM {} LIMIT 1",
    "SELECT {} FROM {} WHERE {} LIMIT 1",
    fetch_one,
    M
);
#[cfg(feature = "sqlx-sqlite")]
fetch_by_where_scalar!(
    sqlx::Sqlite,
    fetch_all_scalar_by_where,
    "SELECT {} FROM {} ",
    "SELECT {} FROM {} WHERE {} ",
    fetch_all,
    Vec<M>
);
#[cfg(feature = "sqlx-sqlite")]
fetch_by_where_scalar!(
    sqlx::Sqlite,
    fetch_one_scalar_by_where,
    "SELECT {} FROM {} LIMIT 1",
    "SELECT {} FROM {} WHERE {} LIMIT 1",
    fetch_one,
    M
);
#[cfg(feature = "sqlx-postgres")]
fetch_by_where_scalar!(
    sqlx::Postgres,
    fetch_all_scalar_by_where,
    "SELECT {} FROM {} ",
    "SELECT {} FROM {} WHERE {} ",
    fetch_all,
    Vec<M>
);
#[cfg(feature = "sqlx-postgres")]
fetch_by_where_scalar!(
    sqlx::Postgres,
    fetch_one_scalar_by_where,
    "SELECT {} FROM {} LIMIT 1",
    "SELECT {} FROM {} WHERE {} LIMIT 1",
    fetch_one,
    M
);
#[cfg(feature = "sqlx-mssql")]
fetch_by_where_scalar!(
    sqlx::Mssql,
    fetch_all_scalar_by_where,
    "SELECT {} FROM {} ",
    "SELECT {} FROM {} WHERE {} ",
    fetch_all,
    Vec<M>
);
#[cfg(feature = "sqlx-mssql")]
fetch_by_where_scalar!(
    sqlx::Mssql,
    fetch_one_scalar_by_where,
    "SELECT TOP 1 {} FROM {}",
    "SELECT TOP 1 {} FROM {} WHERE {}",
    fetch_one,
    M
);

#[cfg(feature = "sqlx-mysql")]
fetch_by_where_scalar_call!(
    sqlx::MySql,
    fetch_all_scalar_by_where_call,
    "SELECT {} FROM {} WHERE {} ",
    fetch_all,
    Vec<M>
);
#[cfg(feature = "sqlx-mysql")]
fetch_by_where_scalar_call!(
    sqlx::MySql,
    fetch_one_scalar_by_where_call,
    "SELECT {} FROM {} WHERE {} LIMIT 1",
    fetch_one,
    M
);
#[cfg(feature = "sqlx-sqlite")]
fetch_by_where_scalar_call!(
    sqlx::Sqlite,
    fetch_all_scalar_by_where_call,
    "SELECT {} FROM {} WHERE {} ",
    fetch_all,
    Vec<M>
);
#[cfg(feature = "sqlx-sqlite")]
fetch_by_where_scalar_call!(
    sqlx::Sqlite,
    fetch_one_scalar_by_where_call,
    "SELECT {} FROM {} WHERE {} LIMIT 1",
    fetch_one,
    M
);
#[cfg(feature = "sqlx-postgres")]
fetch_by_where_scalar_call!(
    sqlx::Postgres,
    fetch_all_scalar_by_where_call,
    "SELECT {} FROM {} WHERE {} ",
    fetch_all,
    Vec<M>
);
#[cfg(feature = "sqlx-postgres")]
fetch_by_where_scalar_call!(
    sqlx::Postgres,
    fetch_one_scalar_by_where_call,
    "SELECT {} FROM {} WHERE {} LIMIT 1",
    fetch_one,
    M
);
#[cfg(feature = "sqlx-mssql")]
fetch_by_where_scalar_call!(
    sqlx::Mssql,
    fetch_all_scalar_by_where_call,
    "SELECT {} FROM {} WHERE {} ",
    fetch_all,
    Vec<M>
);
#[cfg(feature = "sqlx-mssql")]
fetch_by_where_scalar_call!(
    sqlx::Mssql,
    fetch_one_scalar_by_where_call,
    "SELECT TOP 1 {} FROM {} WHERE {}",
    fetch_one,
    M
);
