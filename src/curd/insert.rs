use sqlx::database::HasArguments;
use sqlx::query::Query;
use sqlx::{Database, Error};
use std::vec;
use super::{DbType, FieldItem, ModelTableField, ModelTableName,TableFields,ModelUpdateData,Update,UpdateData};
use sqlx::{Arguments, Executor, IntoArguments};

/// 插入操作
pub trait InsertData<'t,DB>
where DB:Database
{
    fn columns(&self)-> Vec<FieldItem>;
    fn sqlx_bind<'q>(
        &'q self,
        field:&FieldItem,
        res:Query<'q,DB,<DB as HasArguments<'q>>::Arguments>,
    ) -> Query<'q,DB,<DB as HasArguments<'q>>::Arguments>;
    fn sqlx_string(
        &self,
        field:&FieldItem
    ) ->  Option<String>;
    
}
pub trait ModelInsertData<'t, DB,DT>: ModelTableField<DB>+ModelTableName
where
DT: InsertData<'t,DB>,
DB:Database
{
    fn insert_data(&'t self) -> DT;
}

pub struct Insert<'q,DB,T,DT>
where
    T:ModelTableName,
    DB:Database,
    DT:InsertData<'q,DB>,
{
    pub val: Vec<DT>,
    pub fields:TableFields,
    _marker:(std::marker::PhantomData<T>,std::marker::PhantomData<&'q DT>,std::marker::PhantomData<DB>)
}
impl<'q,DB,T,DT> Insert<'q,DB,T,DT>
where
    T:ModelTableName,
    DT:InsertData<'q,DB>,
    DB:Database,
{
    pub fn new(val: DT) -> Self {
        let column=val.columns();
        return Self{
            val:vec![val],
            fields:TableFields(column),
            _marker:Default::default()
        }
    }
    pub fn new_vec(val: Vec<DT>) -> Self {
        let mut fields=TableFields::new(vec![]);
        for tmp in val.iter(){
            fields.marge(tmp.columns());
        }
        return Self{
            val:val,
            fields:fields,
            _marker:Default::default()
        }
    }
    pub fn model<'t:'q,MI>(val: &'t MI) -> Self 
    where MI:ModelInsertData<'q,DB,DT>
    {
        let ival=val.insert_data();
        let column=ival.columns();
        return Self{
            val:vec![ival],
            fields:TableFields(column),
            _marker:Default::default()
        }
    }
    pub fn model_vec<'t:'q,MI>(val:&'t Vec<MI>) -> Self 
    where MI:ModelInsertData<'q,DB,DT>
    {
        let mut vals=vec![];
        let mut fields=TableFields::new(vec![]);
        for tmp in val{
            let ival=tmp.insert_data();
            fields.marge(ival.columns());
            vals.push(ival);
        }
        return Self{
            val:vals,
            fields:fields,
            _marker:Default::default()
        }
    }
    pub fn sql_param(&self) -> Vec<String>{
        let mut values = Vec::<String>::with_capacity(self.val.len());
        for (gid,_) in self.val.iter().enumerate() {
            let len = self.fields.0.len();
            let mut value = Vec::with_capacity(len);
            for i in 0..len {
                let pos = gid * len + i;
                let str = DbType::type_new::<DB>().mark(pos);
                value.push(str);
            }
            let val: String = value.join(",");
            let val = "(".to_string() + val.as_str() + ")";
            values.push(val);
        }
        values
    }
    pub fn sql_values(&self) -> Vec<String>{
        let mut values = Vec::<String>::with_capacity(self.val.len());
        for val in self.val.iter() {
            let mut value = Vec::with_capacity(self.fields.0.len());
            for field in &self.fields.0 {
                if let Some(ival)=val.sqlx_string(field){
                    value.push(ival);
                }
            }
            let val: String = value.join(",");
            let val = "(".to_string() + val.as_str() + ")";
            values.push(val);
        }
        values
    }
    pub fn bind_values<'t>(
        &'t self,
        mut res:Query<'t,DB,<DB as HasArguments<'t>>::Arguments>,
    ) ->Query<'t,DB,<DB as HasArguments<'t>>::Arguments>{
        for val in self.val.iter() {
            for field in &self.fields.0 {
                res = val.sqlx_bind(field, res);
            }
        }
        res
    }
    pub async fn execute<'c,E>(self,executor:E,) -> Result<<DB as Database>::QueryResult, Error> 
    where
        for<'n> <DB as HasArguments<'n>>::Arguments:
            Arguments<'n>+IntoArguments<'n,DB>,
        E: Executor<'c, Database = DB>
     {
        let table = T::table_name();
        let vals = self.sql_param();
        let sql = format!(
            "INSERT INTO {} ({})VALUES {}",
            table.full_name(),
            self.fields.to_vec().join(","),
            vals.join(",")
        );
        let mut res = sqlx::query(sql.as_str());
        res = self.bind_values(res);
        executor.execute(res).await
    }
    #[cfg(feature = "sqlx-mysql")]
    pub async fn execute_update<'c,'t, CT, IT,E>(
        self,
        update: Update<'t,DB, IT, CT>,
        executor:E,
    ) -> Result<<DB as Database>::QueryResult, Error>
    where
        IT: ModelUpdateData<'t,DB, CT>,
        CT: UpdateData<'t,DB>,
        for<'n> <DB as HasArguments<'n>>::Arguments:
            Arguments<'n>+IntoArguments<'n,DB>,
        E: Executor<'c, Database = DB>,
    {
        let table = T::table_name();
        let vals = self.sql_param();
        let sql = format!(
            "INSERT INTO {} ({})VALUES {} ON DUPLICATE KEY UPDATE {}",
            table.full_name(),
            self.fields.to_vec().join(","),
            vals.join(","),
            update.sql_sets()
        );
        let mut res = sqlx::query(sql.as_str());
        res = self.bind_values(res);
        res = update.bind_values(res);
        executor.execute(res).await
    }
    execute_by_sql!(Insert<DB,T,DT>);
}