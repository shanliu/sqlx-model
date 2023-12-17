pub async fn db_get() -> sqlx::Pool<sqlx::Postgres> {
    use std::str::FromStr;
    let table_prefix = "yaf_".to_string();
    sqlx_model::TableName::set_prefix(table_prefix);
    let database_url = "postgres://postgres:000000@127.0.0.1:5432/postgres";
    let option = sqlx::postgres::PgConnectOptions::from_str(database_url).unwrap();
    sqlx::pool::PoolOptions::<sqlx::Postgres>::new()
        .max_connections(5)
        .connect_with(option.to_owned())
        .await
        .unwrap()
}

// CREATE TABLE test.yaf_users (
//     id SERIAL PRIMARY KEY,
//     nickname text DEFAULT NULL,
//     gender SMALLINT  NOT NULL DEFAULT 0 ,
//     headimg text DEFAULT NULL,
//     password_id int  NOT NULL DEFAULT 0
//   )
#[derive(sqlx::FromRow, Clone, Debug)]
//#[sqlx(rename_all="lowercase")]
#[sqlx_model::sqlx_model(db_type = "Postgres", table_pk = "id", table_name = "test.users")]
pub struct UserModel {
    #[sqlx(default)]
    pub id: i32,
    #[sqlx(default)]
    pub nickname: String,
    #[sqlx(default)]
    pub gender: i16,
    #[sqlx(default)]
    pub headimg: Option<String>,
    #[sqlx(default)]
    #[sqlx(rename = "password_id")]
    pub password_id: i32,
}
