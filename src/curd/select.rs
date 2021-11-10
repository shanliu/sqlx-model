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
    ($self_var:ident,$name:ident,$query_type:ident,$fetch_type:ident,$out_type:ty,$ext_sql:literal)=>{
        pub async fn $name<'c,M,RB>(
            &$self_var,
            where_sql:String,
            where_bind:RB,
            pool:&'c Pool<DB>
        )
            ->Result<$out_type,Error>
            where
            for<'t> RB: FnOnce( QueryAs<'t,DB,M,<DB as HasArguments>::Arguments>,&'t Select<DB>) -> QueryAs<'t,DB,M,<DB as HasArguments<'t>>::Arguments>,
            for<'r> M:  FromRow<'r, DB::Row>+Send+Unpin+ModelTableField<DB>,
            for<'n> <DB as HasArguments<'n>>::Arguments:
                Arguments<'n>+IntoArguments<'n,DB>,
            &'c Pool<DB>: Executor<'c, Database = DB> 
        {
            let sql=format!(
                "SELECT {} FROM {} WHERE {} {}",
                $self_var.table_field.to_vec().join(","),
                $self_var.table_name.full_name(),
                where_sql,
                $ext_sql
            );
            let mut res=sqlx::$query_type::<DB, M>(sql.as_str());
            res=where_bind(res,&$self_var);
            res.$fetch_type(pool).await
        }
    };
    ($name:ident,$query_type:ident,$fetch_type:ident,$out_type:ty,$ext_sql:literal)=>{
        fetch_by_where_call!(self,$name,$query_type,$fetch_type,$out_type,$ext_sql);
    };
}



macro_rules! fetch_by_where {
    ($self_var:ident,$name:ident,$query_type:ident,$fetch_type:ident,$out_type:ty,$ext_sql:literal)=>{
        pub async fn $name<'c,M>(
            &$self_var,
            where_sql:Option<String>,
            pool:&'c Pool<DB>
        )
            ->Result<$out_type,Error>
            where
            for<'r> M:  FromRow<'r, DB::Row>+Send+Unpin+ModelTableField<DB>,
            for<'n> <DB as HasArguments<'n>>::Arguments:
                Arguments<'n>+IntoArguments<'n,DB>,
            &'c Pool<DB>: Executor<'c, Database = DB> 
        {
            let sql;
            match where_sql {
                Some(wsql)=>{
                    sql=format!(
                        "SELECT {} FROM {} WHERE {} {}",
                        $self_var.table_field.to_vec().join(","),
                        $self_var.table_name.full_name(),
                        wsql,
                        $ext_sql
                    );
                }
                None=>{
                    sql=format!(
                        "SELECT {} FROM {} {}",
                        $self_var.table_field.to_vec().join(","),
                        $self_var.table_name.full_name(),
                        $ext_sql
                    );
                }
            }
            let res=sqlx::$query_type::<DB, M>(sql.as_str());
            res.$fetch_type(pool).await
        }
    };
    ($name:ident,$query_type:ident,$fetch_type:ident,$out_type:ty,$ext_sql:literal)=>{
        fetch_by_where!(self,$name,$query_type,$fetch_type,$out_type,$ext_sql);
    };
}


macro_rules! fetch_by_where_scalar_call {
    ($self_var:ident,$name:ident,$fetch_type:ident,$out_type:ty,$ext_sql:literal)=>{
        pub async fn $name<'c,M, RB>(
            &$self_var,
            field_name:&str,
            where_sql: String,
            where_bind: RB,
            pool:&'c Pool<DB>
        )
            -> Result<$out_type, Error>
            where
                    for<'t> RB: FnOnce(QueryScalar<'t,DB,M,<DB as HasArguments>::Arguments>,&'t Select<DB>) ->QueryScalar<'t,DB,M,<DB as HasArguments<'t>>::Arguments>,
                    (M,): for<'r> FromRow<'r, DB::Row> + Send + Unpin,
                    M:Send + Unpin,
                    for<'n> <DB as HasArguments<'n>>::Arguments:
                        Arguments<'n>+IntoArguments<'n,DB>,
                    &'c Pool<DB>: Executor<'c, Database = DB> 
            {
            let sql = format!(
                "SELECT {} FROM {} WHERE {} {}",
                field_name,
                $self_var.table_name.full_name(),
                where_sql,
                $ext_sql
            );
            let mut res = sqlx::query_scalar::<DB, M,>(sql.as_str());
            res = where_bind(res,&$self_var);
            res.$fetch_type(pool).await
        }
    };
    ($name:ident,$fetch_type:ident,$out_type:ty,$ext_sql:literal)=>{
        fetch_by_where_scalar_call!(self,$name,$fetch_type,$out_type,$ext_sql);
    };
}


macro_rules! fetch_by_where_scalar {
    ($self_var:ident,$name:ident,$fetch_type:ident,$out_type:ty,$ext_sql:literal)=>{
        pub async fn $name<'c,M>(
            &$self_var,
            field_name:&str,
            where_sql: Option<String>,
            pool:&'c Pool<DB>
        )
            -> Result<$out_type, Error>
            where
                    (M,): for<'r> FromRow<'r, DB::Row> + Send + Unpin,
                    M:Send + Unpin,
                    for<'n> <DB as HasArguments<'n>>::Arguments:
                        Arguments<'n>+IntoArguments<'n,DB>,
                    &'c Pool<DB>: Executor<'c, Database = DB> 
            {

                let sql;
                match where_sql {
                    Some(wsql)=>{
                        sql = format!(
                            "SELECT {} FROM {} WHERE {} {}",
                            field_name,
                            $self_var.table_name.full_name(),
                            wsql,
                            $ext_sql
                        );
                    }
                    None=>{
                        sql = format!(
                            "SELECT {} FROM {} WHERE {}",
                            field_name,
                            $self_var.table_name.full_name(),
                            $ext_sql
                        );
                    }
                }
            let res = sqlx::query_scalar::<DB, M,>(sql.as_str());
            res.$fetch_type(pool).await
        }
    };
    ($name:ident,$fetch_type:ident,$out_type:ty,$ext_sql:literal)=>{
        fetch_by_where_scalar!(self,$name,$fetch_type,$out_type,$ext_sql);
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
    fetch_by_where!(fetch_one_by_where, query_as, fetch_one, M, "");
    fetch_by_where!(fetch_all_by_where, query_as, fetch_all, Vec<M>, "");
    fetch_by_where_call!(fetch_one_by_where_call, query_as, fetch_one, M, "");
    fetch_by_where_call!(fetch_all_by_where_call, query_as, fetch_all, Vec<M>, "");


    fetch_by_sql_scalar!(fetch_one_scalar_by_sql, fetch_one, M);
    fetch_by_sql_scalar!(fetch_all_scalar_by_sql, fetch_all, Vec<M>);
    fetch_by_sql_scalar_call!(fetch_one_scalar_by_sql_call, fetch_one, M);
    fetch_by_sql_scalar_call!(fetch_all_scalar_by_sql_call, fetch_all, Vec<M>);

    fetch_by_where_scalar!(fetch_one_scalar_by_where, fetch_one, M, "");
    fetch_by_where_scalar!(fetch_all_scalar_by_where, fetch_all, Vec<M>, "");
    fetch_by_where_scalar_call!(fetch_one_scalar_by_where_call, fetch_one, M, "");
    fetch_by_where_scalar_call!(fetch_all_scalar_by_where_call, fetch_all, Vec<M>, "");
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
            "SELECT {} FROM {} WHERE {} limit 1",
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
