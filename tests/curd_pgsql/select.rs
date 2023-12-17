use crate::curd_pgsql::common::db_get;
use crate::curd_pgsql::common::UserModel;
use crate::curd_pgsql::common::UserModelRef;
use sqlx::Row;
use sqlx_model::sql_format;
use sqlx_model::Insert;
use sqlx_model::{Select, SqlQuote};
#[tokio::test]
async fn curd_select() {
    let db = db_get().await;
    let nike_name = "new insert".to_string();
    let gender = 1;
    let userinsert = sqlx_model::model_option_set!(UserModelRef,{
        nickname:nike_name,
        gender:gender,
    });
    Insert::<sqlx::Postgres, UserModel, _>::new(userinsert)
        .execute_return(&db, "id")
        .await
        .unwrap();
    let userinsert = sqlx_model::model_option_set!(UserModelRef,{
        nickname:nike_name,
        gender:gender,
    });
    let row = Insert::<sqlx::Postgres, UserModel, _>::new(userinsert)
        .execute_return(&db, "id")
        .await
        .unwrap();
    let addid: i32 = row.get("id");
    //test
    let select = Select::type_new::<UserModel>();
    let user = select
        .fetch_one_by_scalar_pk::<UserModel, _, _>(addid, &db)
        .await
        .unwrap();
    assert_eq!(user.id, addid);
    let user = select.reload::<UserModel, _>(&user, &db).await.unwrap();
    assert_eq!(user.id, addid);
    let user = select
        .fetch_one_by_where::<UserModel, _>(
            &sqlx_model::WhereOption::Where(sql_format!("id={}", addid)),
            &db,
        )
        .await
        .unwrap();
    assert_eq!(user.nickname, nike_name);

    let sql = format!("select {} from {} where id >= $1", "id", select.table_name);
    let mut res = sqlx::query_as::<_, UserModel>(sql.as_str());
    res = res.bind(0);
    let user = res.fetch_one(&db).await.unwrap();
    assert!(user.id > 0);
    assert!(user.gender == 0);

    //test
    let tuser = select
        .fetch_one_scalar_by_where::<String, _>(
            "nickname".to_string().as_str(),
            &sqlx_model::WhereOption::Where(format!("id={addid}")),
            &db,
        )
        .await
        .unwrap();
    assert_eq!(tuser, nike_name);

    let tuser = select
        .fetch_one_scalar_by_scalar_pk::<String, _, _>("nickname", addid, &db)
        .await
        .unwrap();
    assert_eq!(tuser, nike_name);

    //test
    let tuser = select
        .fetch_all_by_where::<UserModel, _>(
            &sqlx_model::WhereOption::Where(format!("id>={addid} order by id asc")),
            &db,
        )
        .await
        .unwrap();
    assert_eq!(tuser.get(0).unwrap().nickname, nike_name);

    //test

    let tuser = select
        .fetch_all_scalar_by_where::<String, _>(
            "nickname",
            &sqlx_model::WhereOption::Where(format!("id>={addid} order by id asc")),
            &db,
        )
        .await
        .unwrap();
    assert_eq!(tuser.get(0).unwrap(), &nike_name);
}
