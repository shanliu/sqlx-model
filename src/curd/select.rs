use sqlx::query::{QueryAs, QueryScalar};
use sqlx::{Database, Error, FromRow, Pool,database::HasArguments};
use std::vec;
use super::TableName;
use super::{DbType, ModelTableField, ModelTableName, TableFields};
use sqlx::{Arguments, Executor, IntoArguments};


macro_rules! fetch_by_sql_call_row {
    ($self_var:ident,$name:ident,$query_type:ident,$fetch_type:ident,$out_type:ty)=>{
        pub async fn $name<'c,M,SQ,RB>(
            &$self_var
            ,sql_call:SQ
            ,bind_call:RB
            ,pool:&'c Pool<DB>
        )
        ->Result<$out_type,Error>
        where
            SQ: FnOnce(&Select<DB>) -> String,
            RB:for<'t> FnOnce( QueryAs<'t,DB,M,<DB as HasArguments>::Arguments>,&'t Select<DB>) ->QueryAs<'t,DB,M,<DB as HasArguments<'t>>::Arguments>,
            M: for<'r> FromRow<'r, DB::Row>+Send+Unpin,
            for<'n> <DB as HasArguments<'n>>::Arguments:
                Arguments<'n>+IntoArguments<'n,DB>,
            &'c Pool<DB>: Executor<'c, Database = DB> 
        {
            let sql=sql_call(&$self_var);
            let mut res=sqlx::$query_type::<DB,M>(sql.as_str());
            res=bind_call(res,&$self_var);
            res.$fetch_type(pool).await
        }
    };
    ($name:ident,$query_type:ident,$fetch_type:ident,$out_type:ty)=>{
        fetch_by_sql_call_row!(self,$name,$query_type,$fetch_type,$out_type);
    };
}

macro_rules! fetch_by_sql_row {
    ($self_var:ident,$name:ident,$query_type:ident,$fetch_type:ident,$out_type:ty)=>{
        pub async fn $name<'c,M,SQ>(
            &$self_var
            ,sql_call:SQ
            ,pool:&'c Pool<DB>
        )
        ->Result<$out_type,Error>
        where
            SQ: FnOnce(&Select<DB>) -> String,
            M: for<'r> FromRow<'r, DB::Row>+Send+Unpin,
            for<'n> <DB as HasArguments<'n>>::Arguments:
                Arguments<'n>+IntoArguments<'n,DB>,
            &'c Pool<DB>: Executor<'c, Database = DB> 
        {
            let sql=sql_call(&$self_var);
            let res=sqlx::$query_type::<DB,M>(sql.as_str());
            res.$fetch_type(pool).await
        }
    };
    ($name:ident,$query_type:ident,$fetch_type:ident,$out_type:ty)=>{
        fetch_by_sql_row!(self,$name,$query_type,$fetch_type,$out_type);
    };
}

macro_rules! fetch_by_sql_scalar_call {
    ($self_var:ident,$name:ident,$fetch_type:ident,$out_type:ty)=>{
        /// M 为 Model 类型
        pub async fn $name<'c,M,SQ, RB>(
            &$self_var
            , sql_call: SQ
            , bind_call: RB
            ,pool:&'c Pool<DB>
        )
            -> Result<$out_type, Error>
            where
                SQ: FnOnce(&Select<DB>) -> String,
                for<'t> RB: FnOnce( QueryScalar<'t,DB,M,<DB as HasArguments>::Arguments>,&'t Select<DB>) -> QueryScalar<'t,DB,M,<DB as HasArguments<'t>>::Arguments>,
                (M,): for<'r> FromRow<'r, DB::Row> + Send + Unpin,
                M:Send + Unpin,
                for<'n> <DB as HasArguments<'n>>::Arguments:
                    Arguments<'n>+IntoArguments<'n,DB>,
                &'c Pool<DB>: Executor<'c, Database = DB> 
        {
            let sql = sql_call(&$self_var);
            let mut res = sqlx::query_scalar::<DB, M,>(sql.as_str());
            res = bind_call(res,&$self_var);
            res.$fetch_type(pool).await
        }
    };
    ($name:ident,$fetch_type:ident,$out_type:ty)=>{
        fetch_by_sql_scalar_call!(self,$name,$fetch_type,$out_type);
    };
}
macro_rules! fetch_by_sql_scalar {
    ($self_var:ident,$name:ident,$fetch_type:ident,$out_type:ty)=>{
        /// M 为 返回某字段类型
        pub async fn $name<'c,M,SQ>(
            &$self_var
            , sql_call: SQ
            ,pool:&'c Pool<DB>
        )
            -> Result<$out_type, Error>
            where
                SQ: FnOnce(&Select<DB>) -> String,
                (M,): for<'r> FromRow<'r, DB::Row> + Send + Unpin,
                M:Send + Unpin,
                for<'n> <DB as HasArguments<'n>>::Arguments:
                    Arguments<'n>+IntoArguments<'n,DB>,
                &'c Pool<DB>: Executor<'c, Database = DB> 
        {
            let sql = sql_call(&$self_var);
            let res = sqlx::query_scalar::<DB, M,>(sql.as_str());
            res.$fetch_type(pool).await
        }
    };
    ($name:ident,$fetch_type:ident,$out_type:ty)=>{
        fetch_by_sql_scalar!(self,$name,$fetch_type,$out_type);
    };
}



macro_rules! fetch_by_where_call {
    ($self_var:ident,$bind_type:ty,$name:ident,$sql:literal,$query_type:ident,$fetch_type:ident,$out_type:ty)=>{
        impl Select<$bind_type>
        {
            /// M 为 Model  类型
            pub async fn $name<'c,M,RB>(
                &$self_var,
                where_sql:String,
                where_bind:RB,
                pool:&'c Pool<$bind_type>
            )
                ->Result<$out_type,Error>
                where
                for<'t> RB: FnOnce( QueryAs<'t,$bind_type,M,<$bind_type as HasArguments>::Arguments>,&'t Select<$bind_type>) -> QueryAs<'t,$bind_type,M,<$bind_type as HasArguments<'t>>::Arguments>,
                for<'r> M:  FromRow<'r, <$bind_type as Database>::Row>+Send+Unpin+ModelTableField<$bind_type>,
                for<'n> <$bind_type as HasArguments<'n>>::Arguments:
                    Arguments<'n>+IntoArguments<'n,$bind_type>,
                &'c Pool<$bind_type>: Executor<'c, Database = $bind_type> 
            {
                let sql=format!(
                    $sql,
                    $self_var.table_field.to_vec().join(","),
                    $self_var.table_name.full_name(),
                    where_sql
                );
                let mut res=sqlx::$query_type::<$bind_type, M>(sql.as_str());
                res=where_bind(res,&$self_var);
                res.$fetch_type(pool).await
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
            pub async fn $name<'c,M>(
                &$self_var,
                where_sql:Option<String>,
                pool:&'c Pool<$bind_type>
            )
                ->Result<$out_type,Error>
                where
                for<'r> M:  FromRow<'r, <$bind_type as Database>::Row>+Send+Unpin+ModelTableField<$bind_type>,
                for<'n> <$bind_type as HasArguments<'n>>::Arguments:
                    Arguments<'n>+IntoArguments<'n,$bind_type>,
                &'c Pool<$bind_type>: Executor<'c, Database = $bind_type> 
            {
                let sql;
                match where_sql {
                    Some(wsql)=>{
                        sql=format!(
                            $where_sql,
                            $self_var.table_field.to_vec().join(","),
                            $self_var.table_name.full_name(),
                            wsql
                        );
                    }
                    None=>{
                        sql=format!(
                            $sql,
                            $self_var.table_field.to_vec().join(","),
                            $self_var.table_name.full_name()
                        );
                    }
                }
                let res=sqlx::$query_type::<$bind_type, M>(sql.as_str());
                res.$fetch_type(pool).await
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
            pub async fn $name<'c,M, RB>(
                &$self_var,
                field_name:&str,
                where_sql: String,
                where_bind: RB,
                pool:&'c Pool<$bind_type>
            )
                -> Result<$out_type, Error>
                where
                        for<'t> RB: FnOnce(QueryScalar<'t,$bind_type,M,<$bind_type as HasArguments>::Arguments>,&'t Select<$bind_type>) ->QueryScalar<'t,$bind_type,M,<$bind_type as HasArguments<'t>>::Arguments>,
                        (M,): for<'r> FromRow<'r, <$bind_type as Database>::Row> + Send + Unpin,
                        M:Send + Unpin,
                        for<'n> <$bind_type as HasArguments<'n>>::Arguments:
                            Arguments<'n>+IntoArguments<'n,$bind_type>,
                        &'c Pool<$bind_type>: Executor<'c, Database = $bind_type> 
                {
                let sql = format!(
                    $sql,
                    field_name,
                    $self_var.table_name.full_name(),
                    where_sql
                );
                let mut res = sqlx::query_scalar::<$bind_type, M,>(sql.as_str());
                res = where_bind(res,&$self_var);
                res.$fetch_type(pool).await
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
            pub async fn $name<'c,M>(
                &$self_var,
                field_name:&str,
                where_sql: Option<String>,
                pool:&'c Pool<$bind_type>
            )
                -> Result<$out_type, Error>
                where
                        (M,): for<'r> FromRow<'r, <$bind_type as Database>::Row> + Send + Unpin,
                        M:Send + Unpin,
                        for<'n> <$bind_type as HasArguments<'n>>::Arguments:
                            Arguments<'n>+IntoArguments<'n,$bind_type>,
                        &'c Pool<$bind_type>: Executor<'c, Database = $bind_type> 
                {

                    let sql;
                    match where_sql {
                        Some(wsql)=>{
                            sql = format!(
                                $where_sql,
                                field_name,
                                $self_var.table_name.full_name(),
                                wsql
                            );
                        }
                        None=>{
                            sql = format!(
                                $sql,
                                field_name,
                                $self_var.table_name.full_name()
                            );
                        }
                    }
                let res = sqlx::query_scalar::<$bind_type, M,>(sql.as_str());
                res.$fetch_type(pool).await
            }
        }
    };
    ($bind_type:ty,$name:ident,$sql:literal,$where_sql:literal,$fetch_type:ident,$out_type:ty)=>{
        fetch_by_where_scalar!(self,$bind_type,$name,$sql,$where_sql,$fetch_type,$out_type);
    };
}


/// 查询操作
pub struct Select<DB>
where DB:Database
{
    pub table_name: TableName,
    pub table_field: TableFields,
    pub table_pk: TableFields,
    _marker:std::marker::PhantomData<DB>
}
impl <DB> Select<DB>
where DB:Database
{
    pub fn new(table_name:TableName,table_field:TableFields,table_pk:TableFields) -> Self
    {
        Select {table_name,table_field,table_pk,_marker:Default::default()}
    }
    pub fn type_new<T>() -> Self
    where
    T: ModelTableField<DB>+ModelTableName
    {
        Select {
            table_name:T::table_name(),
            table_field: T::table_column(),
            table_pk: T::table_pk(),
            _marker:Default::default()
        }
    }
    fetch_by_sql_row!(fetch_one_by_sql, query_as, fetch_one, M);
    fetch_by_sql_row!(fetch_all_by_sql, query_as, fetch_all, Vec<M>);
    fetch_by_sql_call_row!(fetch_one_by_sql_call, query_as, fetch_one, M);
    fetch_by_sql_call_row!(fetch_all_by_sql_call, query_as, fetch_all, Vec<M>);
    fetch_by_sql_scalar!(fetch_one_scalar_by_sql, fetch_one, M);
    fetch_by_sql_scalar!(fetch_all_scalar_by_sql, fetch_all, Vec<M>);
    fetch_by_sql_scalar_call!(fetch_one_scalar_by_sql_call, fetch_one, M);
    fetch_by_sql_scalar_call!(fetch_all_scalar_by_sql_call, fetch_all, Vec<M>);

    /// 非联合主键的表通过主键值查找某记录
    /// @field_name 需要获取的字段名
    /// @pk_scalar 主键值
    /// @pool DB连接
    /// M 为 Model 类型
    pub async fn fetch_one_by_scalar_pk<'c,M,PT>(
        &self,
        pk_scalar: PT,
        pool:&'c Pool<DB>
    )
        ->Result<M,Error>
        where
        for<'q> PT:'q+ Send + sqlx::Encode<'q, DB> + sqlx::Type<DB>,
        for<'r> M:  FromRow<'r, DB::Row>+Send+Unpin+ModelTableField<DB>,
        for<'n> <DB as HasArguments<'n>>::Arguments:
            Arguments<'n>+IntoArguments<'n,DB>,
        &'c Pool<DB>: Executor<'c, Database = DB>,
    {
        let where_sql=scalar_pk_where!(DB,self.table_pk);
        let sql=format!(
            "SELECT {} FROM {} WHERE {}",
            self.table_field.to_vec().join(","),
            self.table_name.full_name(),
            where_sql
        );
        let mut res=sqlx::query_as::<DB, M,>(sql.as_str());
        res=res.bind(pk_scalar);
        res.fetch_one(pool).await
    }
    /// 非联合主键的表通过主键值查找某字段值
    /// @field_name 需要获取的字段名
    /// @pk_scalar 主键值
    /// @pool DB连接
    /// M 为 @field_name 字段类型
    pub async fn fetch_one_scalar_by_scalar_pk<'c,M,PT>(
        &self,
        field_name:&str,
        pk_scalar: PT,
        pool:&'c Pool<DB>
    )
        ->Result<M,Error>
        where
        for<'q> PT:'q+ Send + sqlx::Encode<'q, DB> + sqlx::Type<DB>,
        (M,): for<'r> FromRow<'r, DB::Row> + Send + Unpin,
        M:Send + Unpin,
        for<'n> <DB as HasArguments<'n>>::Arguments:
            Arguments<'n>+IntoArguments<'n,DB>,
        &'c Pool<DB>: Executor<'c, Database = DB>, 
    {
        let where_sql=scalar_pk_where!(DB,self.table_pk);
        let sql=format!(
            "SELECT {} FROM {} WHERE {}",
            field_name,
            self.table_name.full_name(),
            where_sql
        );
        let mut res=sqlx::query_scalar::<DB, M,>(sql.as_str());
        res=res.bind(pk_scalar);
        res.fetch_one(pool).await
    }
    /// 从DB中重新加载Model里值
    /// @val Model 变量
    /// @pool DB连接
    pub async fn reload<'c,M>(&self, val: &M,pool:&'c Pool<DB>)->Result<M,Error>
    where
        M: for<'r> FromRow<'r, DB::Row> + Send + Unpin+ModelTableField<DB>,
        for<'n> <DB as HasArguments<'n>>::Arguments:
            Arguments<'n>+IntoArguments<'n,DB>,
        &'c Pool<DB>: Executor<'c, Database = DB> 
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
        res.fetch_one(pool).await
    }
}

#[cfg(feature = "sqlx-mysql")]
fetch_by_where!(sqlx::MySql,fetch_all_by_where, "SELECT {} FROM {} ","SELECT {} FROM {} WHERE {} ",query_as, fetch_all, Vec<M>);
#[cfg(feature = "sqlx-mysql")]
fetch_by_where!(sqlx::MySql,fetch_one_by_where,"SELECT {} FROM {} LIMIT 1","SELECT {} FROM {} WHERE {} LIMIT 1", query_as, fetch_one, M);
#[cfg(feature = "sqlx-sqlite")]
fetch_by_where!(sqlx::Sqlite,fetch_all_by_where, "SELECT {} FROM {} ","SELECT {} FROM {} WHERE {} ",query_as, fetch_all, Vec<M>);
#[cfg(feature = "sqlx-sqlite")]
fetch_by_where!(sqlx::Sqlite,fetch_one_by_where,"SELECT {} FROM {} LIMIT 1","SELECT {} FROM {} WHERE {} LIMIT 1", query_as, fetch_one, M);
#[cfg(feature = "sqlx-postgres")]
fetch_by_where!(sqlx::Postgres,fetch_all_by_where, "SELECT {} FROM {} ","SELECT {} FROM {} WHERE {} ",query_as, fetch_all, Vec<M>);
#[cfg(feature = "sqlx-postgres")]
fetch_by_where!(sqlx::Postgres,fetch_one_by_where,"SELECT {} FROM {} LIMIT 1","SELECT {} FROM {} WHERE {} LIMIT 1", query_as, fetch_one, M);
#[cfg(feature = "sqlx-mssql")]
fetch_by_where!(sqlx::Mssql,fetch_all_by_where, "SELECT {} FROM {} ","SELECT {} FROM {} WHERE {} ",query_as, fetch_all, Vec<M>);
#[cfg(feature = "sqlx-mssql")]
fetch_by_where!(sqlx::Mssql,fetch_one_by_where,"SELECT  TOP 1 {} FROM {}","SELECT  TOP 1 {} FROM {} WHERE {}", query_as, fetch_one, M);


#[cfg(feature = "sqlx-mysql")]
fetch_by_where_call!(sqlx::MySql,fetch_all_by_where_call,"SELECT {} FROM {} WHERE {}", query_as, fetch_all, Vec<M>);
#[cfg(feature = "sqlx-mysql")]
fetch_by_where_call!(sqlx::MySql,fetch_one_by_where_call,"SELECT {} FROM {} WHERE {} LIMIT 1", query_as, fetch_one, M);
#[cfg(feature = "sqlx-sqlite")]
fetch_by_where_call!(sqlx::Sqlite,fetch_all_by_where_call,"SELECT {} FROM {} WHERE {}", query_as, fetch_all, Vec<M>);
#[cfg(feature = "sqlx-sqlite")]
fetch_by_where_call!(sqlx::Sqlite,fetch_one_by_where_call,"SELECT {} FROM {} WHERE {} LIMIT 1", query_as, fetch_one, M);
#[cfg(feature = "sqlx-postgres")]
fetch_by_where_call!(sqlx::Postgres,fetch_all_by_where_call,"SELECT {} FROM {} WHERE {}", query_as, fetch_all, Vec<M>);
#[cfg(feature = "sqlx-postgres")]
fetch_by_where_call!(sqlx::Postgres,fetch_one_by_where_call,"SELECT {} FROM {} WHERE {} LIMIT 1", query_as, fetch_one, M);
#[cfg(feature = "sqlx-mssql")]
fetch_by_where_call!(sqlx::Mssql,fetch_all_by_where_call,"SELECT {} FROM {} WHERE {}", query_as, fetch_all, Vec<M>);
#[cfg(feature = "sqlx-mssql")]
fetch_by_where_call!(sqlx::Mssql,fetch_one_by_where_call,"SELECT TOP 1 {} FROM {} WHERE {}", query_as, fetch_one, M);

#[cfg(feature = "sqlx-mysql")]
fetch_by_where_scalar!(sqlx::MySql,fetch_all_scalar_by_where,"SELECT {} FROM {} ","SELECT {} FROM {} WHERE {} ", fetch_all, Vec<M>);
#[cfg(feature = "sqlx-mysql")]
fetch_by_where_scalar!(sqlx::MySql,fetch_one_scalar_by_where,"SELECT {} FROM {} LIMIT 1","SELECT {} FROM {} WHERE {} LIMIT 1", fetch_one, M);
#[cfg(feature = "sqlx-sqlite")]
fetch_by_where_scalar!(sqlx::Sqlite,fetch_all_scalar_by_where,"SELECT {} FROM {} ","SELECT {} FROM {} WHERE {} ", fetch_all, Vec<M>);
#[cfg(feature = "sqlx-sqlite")]
fetch_by_where_scalar!(sqlx::Sqlite,fetch_one_scalar_by_where,"SELECT {} FROM {} LIMIT 1","SELECT {} FROM {} WHERE {} LIMIT 1", fetch_one, M);
#[cfg(feature = "sqlx-postgres")]
fetch_by_where_scalar!(sqlx::Postgres,fetch_all_scalar_by_where,"SELECT {} FROM {} ","SELECT {} FROM {} WHERE {} ", fetch_all, Vec<M>);
#[cfg(feature = "sqlx-postgres")]
fetch_by_where_scalar!(sqlx::Postgres,fetch_one_scalar_by_where,"SELECT {} FROM {} LIMIT 1","SELECT {} FROM {} WHERE {} LIMIT 1", fetch_one, M);
#[cfg(feature = "sqlx-mssql")]
fetch_by_where_scalar!(sqlx::Mssql,fetch_all_scalar_by_where,"SELECT {} FROM {} ","SELECT {} FROM {} WHERE {} ", fetch_all, Vec<M>);
#[cfg(feature = "sqlx-mssql")]
fetch_by_where_scalar!(sqlx::Mssql,fetch_one_scalar_by_where,"SELECT TOP 1 {} FROM {}","SELECT TOP 1 {} FROM {} WHERE {}", fetch_one, M);



#[cfg(feature = "sqlx-mysql")]
fetch_by_where_scalar_call!(sqlx::MySql,fetch_all_scalar_by_where_call,"SELECT {} FROM {} WHERE {} ", fetch_all, Vec<M>);
#[cfg(feature = "sqlx-mysql")]
fetch_by_where_scalar_call!(sqlx::MySql,fetch_one_scalar_by_where_call,"SELECT {} FROM {} WHERE {} LIMIT 1", fetch_one, M);
#[cfg(feature = "sqlx-sqlite")]
fetch_by_where_scalar_call!(sqlx::Sqlite,fetch_all_scalar_by_where_call,"SELECT {} FROM {} WHERE {} ", fetch_all, Vec<M>);
#[cfg(feature = "sqlx-sqlite")]
fetch_by_where_scalar_call!(sqlx::Sqlite,fetch_one_scalar_by_where_call,"SELECT {} FROM {} WHERE {} LIMIT 1", fetch_one, M);
#[cfg(feature = "sqlx-postgres")]
fetch_by_where_scalar_call!(sqlx::Postgres,fetch_all_scalar_by_where_call,"SELECT {} FROM {} WHERE {} ", fetch_all, Vec<M>);
#[cfg(feature = "sqlx-postgres")]
fetch_by_where_scalar_call!(sqlx::Postgres,fetch_one_scalar_by_where_call,"SELECT {} FROM {} WHERE {} LIMIT 1", fetch_one, M);
#[cfg(feature = "sqlx-mssql")]
fetch_by_where_scalar_call!(sqlx::Mssql,fetch_all_scalar_by_where_call,"SELECT {} FROM {} WHERE {} ", fetch_all, Vec<M>);
#[cfg(feature = "sqlx-mssql")]
fetch_by_where_scalar_call!(sqlx::Mssql,fetch_one_scalar_by_where_call,"SELECT TOP 1 {} FROM {} WHERE {}", fetch_one, M);
