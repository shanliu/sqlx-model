use sqlx::Row;
use sqlx_model::{TableName};
pub async fn test_db()->sqlx::Pool<sqlx::MySql>{
    use std::str::FromStr;
    let table_prefix = "yaf_".to_string();
    TableName::set_prefix(table_prefix);
    let database_url = "mysql://root:@127.0.0.1/test";
    let option =sqlx::mysql::MySqlConnectOptions::from_str(&database_url)
        .unwrap();
    sqlx::pool::PoolOptions::<sqlx::MySql>::new()
        .max_connections(5)
        .connect_with(
            option.to_owned()
        )
        .await
        .unwrap()
}

#[tokio::test]
async fn main()  {
    let db=test_db().await;
    let res = sqlx::query("desc yaf_users");
    let rows=res.fetch_all(&db).await.unwrap();
    let mut fields=vec![];
    for row in rows{
        let mut st="String";
        let ty=row.get::<&str,_>("Type");
        if !ty.find("int").is_none(){
            if !ty.find("unsigned").is_none(){
                st="u32";
            }else{
                st="i32";
            }
        }
        let tpl=format!(r#"
            #[sqlx(default)]
            pub {name}: {type}
            "#,
            name=row.get::<&str,_>("Field"),
            r#type=st
        );
        let tpl=tpl.trim_end().to_owned();
        fields.push(tpl);
        // println!("{:?}",);
        // println!("{:?}",row.try_get::<&str,_>("Null"));
        // println!("{:?}",row.try_get::<&str,_>("Key"));
    }
   
    let struct_data=format!(r#"
        #[derive(sqlx::FromRow,sqlx_model::SqlxModel,Clone,Debug)]
        {pk}
        {tablename}
        pub struct {name}Model {{
            {fields}
        }}
    "#,
        tablename="#[sqlx_model(table_name=\"yaf_user\")]",
        pk="#[sqlx_model(pk=\"id\")]",
        name="User",
        fields=fields.join(",\n")
    );


    println!("{}",struct_data);

}


 