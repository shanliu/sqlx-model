use crate::curd_pgsql::common::UserModelRef;
use crate::curd_pgsql::common::{db_get, UserModel};
use sqlx_model::{Insert, ModelTableName};

#[tokio::test]
async fn curd_insert() {
    let db = db_get().await;

    // //---
    let nike_name = "new vec insert".to_string();
    let gender = 1;
    let userinsert = vec![sqlx_model::model_option_set!(UserModelRef,{
        nickname:nike_name,
        gender:gender,
    })];
    Insert::<sqlx::Postgres, UserModel, _>::new_vec(userinsert)
        .execute(&db)
        .await
        .unwrap();

    let userinsert = sqlx_model::model_option_set!(UserModelRef,{
        nickname:nike_name,
        gender:gender,
    });
    let tmp = Insert::<sqlx::Postgres, UserModel, _>::new(userinsert);
    let sql = {
        let table = UserModel::table_name();
        let vals = tmp.sql_values();
        let sql = format!(
            "INSERT INTO {} ({})VALUES {}",
            table.full_name(),
            tmp.fields.to_vec().join(","),
            vals.join(",")
        );
        sql
    };
    let i7 = sqlx::query(sql.as_str()).execute(&db).await.unwrap();
    assert_eq!(i7.rows_affected(), 1);

    let nike_name = "new insert 8".to_string();
    let gender = 1;
    let userinsert = sqlx_model::model_option_set!(UserModelRef,{
        nickname:nike_name,
        gender:gender,
    });
    let tmp = Insert::<sqlx::Postgres, UserModel, _>::new(userinsert);
    let sql = {
        let table = UserModel::table_name();
        let vals = tmp.sql_values();
        let sql = format!(
            "INSERT INTO {} ({})VALUES {}",
            table.full_name(),
            tmp.fields.to_vec().join(","),
            vals.join(",")
        );
        sql
    };
    let mut res = sqlx::query(sql.as_str());
    res = tmp.bind_values(res);
    let i8 = res.execute(&db).await.unwrap();
    assert_eq!(i8.rows_affected(), 1);
}
