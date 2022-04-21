use sqlx_model::{Select, TableName,sql_format,sql_array_str,sql_option_str,SqlQuote};
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

// CREATE TABLE `yaf_users` (
//     `id` int(11) unsigned NOT NULL AUTO_INCREMENT COMMENT '用户ID',
//     `nickname` varchar(32) DEFAULT NULL COMMENT '昵称',
//     `gender` tinyint(3) unsigned NOT NULL DEFAULT 0 COMMENT '性别 1 男 2 女',
//     `headimg` varchar(64) DEFAULT NULL COMMENT '头像地址',
//     `password_id` int(10) unsigned NOT NULL DEFAULT 0 COMMENT '密码ID',
//     `add_time` int(10) unsigned NOT NULL COMMENT '添加时间',
//     PRIMARY KEY (`id`)
//   ) ENGINE=InnoDB 

#[derive(sqlx::FromRow,sqlx_model::SqlxModel,Clone,Debug)]
#[sqlx_model(table_name="users")]
pub struct UserModel {
    pub id: u32,
    pub nickname: Option<String>,
    pub gender: Option<u8>,
    pub headimg: Option<String>,
    pub password_id: Option<u32>,
}


#[async_std::main]
async fn main()  {
     let data=["dd","b'bb"];
     let password_id=Some(1);
     let sql=sql_format!(
            "select * from yaf_users where id>{id} {in_grade} and password_id {password_where} ",
            id=1,
            in_grade=sql_array_str!("and grade in ({})",data),
            password_where=sql_option_str!("= {}","is {}",password_id)
        );
     println!("{}",sql);//select * from yaf_users where id>1 and grade in ('dd','bbb') and password_id = 1
     let pool=test_db().await;
     let _user=UserModel{
        id:1,
        nickname: Some("11".to_string()),
        gender: None,
        headimg: None,
        password_id: None,
    };
    //sql select
    let _= Select::type_new::<UserModel>().fetch_one_by_where::<UserModel,_>(Some("id>1".to_string()), &pool).await.unwrap();

    //bind sql select
    let (sql,bind_res)=sqlx_model::sql_bind!(sqlx::MySql,"id>{id}");
    let _= Select::type_new::<UserModel>().fetch_one_by_where_call::<UserModel,_,_>(sql,|mut query_res,_|{
        sqlx_model::sql_bind_vars!(bind_res,query_res,{
            "id":1
        })
    }, &pool).await.unwrap();
    //more example,see tests
 }


 