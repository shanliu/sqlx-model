macro_rules! execute_by_sql{
    ($self_var:ident,$self_type:ty)=>{
        pub async fn execute_by_sql_call<'c,SQ,RB>(
            $self_var
            ,sql_call:SQ
            ,bind_call:RB
            ,pool:&'c Pool<DB>
            )->Result<<DB as Database>::QueryResult,Error>
        where
            SQ:FnOnce(&$self_type)->String,
            for<'ct> RB:FnOnce(Query<'ct,DB,<DB as HasArguments>::Arguments>,&'ct $self_type)->Query<'ct,DB,<DB as HasArguments<'ct>>::Arguments>,
            for<'n> <DB as HasArguments<'n>>::Arguments:
                Arguments<'n>+IntoArguments<'n,DB>,
            &'c Pool<DB>: Executor<'c, Database = DB>
        {
            let sql=sql_call(&$self_var);
            let mut res =sqlx::query(sql.as_str());
            res=bind_call(res,&$self_var);
            res.execute(pool).await
        }
        pub async fn execute_by_sql<'c,SQ>(
            $self_var
            ,sql_call:SQ
            ,pool:&'c Pool<DB>
            )->Result<<DB as Database>::QueryResult,Error>
        where
            SQ:FnOnce(&$self_type)->String,
            for<'n> <DB as HasArguments<'n>>::Arguments:
                Arguments<'n>+IntoArguments<'n,DB>,
            &'c Pool<DB>: Executor<'c, Database = DB>
        {
            let sql=sql_call(&$self_var);
            let res =sqlx::query(sql.as_str());
            res.execute(pool).await
        }
    };
    ($ret_type:ty)=>{
        execute_by_sql!(self,$ret_type);
    };
}

macro_rules! scalar_pk_where {
    ($db_type:ty,$table_pk:expr) => {
        match $table_pk.0.get(0){
            Some(ref pkfield)=>{
                let bst = DbType::type_new::<$db_type>().mark(0);
                format!("{}={}", pkfield.name, bst)
            }
            None=>{
                "0".to_string()
            }
        }
    };
}
