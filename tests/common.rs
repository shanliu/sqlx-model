pub async fn db_mysql() -> sqlx::Pool<sqlx::MySql> {
    use std::str::FromStr;
    let table_prefix = "yaf_".to_string();
    sqlx_model::TableName::set_prefix(table_prefix);
    let database_url = "mysql://root:000@127.0.0.1/test";
    let option = sqlx::mysql::MySqlConnectOptions::from_str(database_url).unwrap();
    sqlx::pool::PoolOptions::<sqlx::MySql>::new()
        .max_connections(5)
        .connect_with(option.to_owned())
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

#[derive(sqlx::FromRow, sqlx_model::SqlxModel, Clone, Debug)]
//#[sqlx(rename_all="lowercase")]
#[sqlx_model(table_pk = "id")]
#[sqlx_model(table_name = "users")]
pub struct UserModel {
    #[sqlx(default)]
    pub id: u32,
    #[sqlx(default)]
    pub nickname: String,
    #[sqlx(default)]
    pub gender: u8,
    #[sqlx(default)]
    pub headimg: Option<String>,
    #[sqlx(default)]
    #[sqlx(rename = "password_id")]
    pub password_id: u32,
}
