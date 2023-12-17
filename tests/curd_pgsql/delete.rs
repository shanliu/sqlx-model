use crate::curd_pgsql::common::db_get;
use crate::curd_pgsql::common::UserModel;
use crate::curd_pgsql::common::UserModelRef;
use sqlx::Row;
use sqlx_model::{Delete, Insert, ModelTableName, Select};
#[tokio::test]
async fn curd_delete() {
    let db = db_get().await;

    //test
    let nike_name = "new vec delete".to_string();
    let gender = 1;
    let userinsert = sqlx_model::model_option_set!(UserModelRef,{

        nickname:nike_name,
        gender:gender,
        password_id:1
    });
    let row = Insert::<sqlx::Postgres, UserModel, _>::new(userinsert)
        .execute_return(&db, "id")
        .await
        .unwrap();
    let update_id: i32 = row.get("id");
    println!("add id:{}", update_id);

    let select = Select::type_new::<UserModel>();
    let user = select
        .fetch_one_by_where::<UserModel, _>(&sqlx_model::WhereOption::None, &db)
        .await
        .unwrap();
    let detete = Delete::<sqlx::Postgres>::new(UserModel::table_name())
        .execute_by_pk(&user, &db)
        .await
        .unwrap();
    assert_eq!(detete.rows_affected(), 1);

    //test
    let nike_name = "new vec delete".to_string();
    let gender = 1;
    let userinsert = sqlx_model::model_option_set!(UserModelRef,{
        nickname:nike_name,
        gender:gender,
        password_id:1
    });

    let row = Insert::<sqlx::Postgres, UserModel, _>::new(userinsert)
        .execute_return(&db, "id")
        .await
        .unwrap();
    let update_id: i32 = row.get("id");
    let detete = Delete::<sqlx::Postgres>::new(UserModel::table_name())
        .execute_by_scalar_pk::<UserModel, _, _>(update_id, &db)
        .await
        .unwrap();
    assert_eq!(detete.rows_affected(), 1);
}
