use crate::curd_mysql::common::db_mysql;
use crate::curd_mysql::common::UserModel;
use crate::curd_mysql::common::UserModelRef;
use sqlx_model::sql_format;
use sqlx_model::Insert;
use sqlx_model::{Select, SqlQuote};
#[tokio::test]
async fn curd_select() {
    let db = db_mysql().await;
    let nike_name = "new insert".to_string();
    let gender = 1;
    let userinsert = sqlx_model::model_option_set!(UserModelRef,{
        nickname:nike_name,
        gender:gender,
    });
    let _ = Insert::<sqlx::MySql, UserModel, _>::new(userinsert)
        .execute(&db)
        .await
        .unwrap();
    let userinsert = sqlx_model::model_option_set!(UserModelRef,{
        nickname:nike_name,
        gender:gender,
    });
    let iid = Insert::<sqlx::MySql, UserModel, _>::new(userinsert)
        .execute(&db)
        .await
        .unwrap()
        .last_insert_id();

    //test
    let select = Select::type_new::<UserModel>();
    let user = select
        .fetch_one_by_scalar_pk::<UserModel, _, _>(iid, &db)
        .await
        .unwrap();
    assert_eq!(user.id as u64, iid);
    let user = select.reload::<UserModel, _>(&user, &db).await.unwrap();
    assert_eq!(user.id as u64, iid);
    let user = select
        .fetch_one_by_where::<UserModel, _>(
            &sqlx_model::WhereOption::Where(sql_format!("id={}", iid)),
            &db,
        )
        .await
        .unwrap();
    assert_eq!(user.nickname, nike_name);

    let sql = format!("select {} from {} where id >= ?", "id", select.table_name);
    let mut res = sqlx::query_as::<_, UserModel>(sql.as_str());
    res = res.bind(0);
    let user = res.fetch_one(&db).await.unwrap();
    assert!(user.id > 0);
    assert!(user.gender == 0);

    //test
    let tuser = select
        .fetch_one_scalar_by_where::<String, _>(
            "nickname".to_string().as_str(),
            &sqlx_model::WhereOption::Where(format!("id={iid}")),
            &db,
        )
        .await
        .unwrap();
    assert_eq!(tuser, nike_name);

    let tuser = select
        .fetch_one_scalar_by_scalar_pk::<String, _, _>("nickname", iid, &db)
        .await
        .unwrap();
    assert_eq!(tuser, nike_name);

    //test
    let tuser = select
        .fetch_all_by_where::<UserModel, _>(
            &sqlx_model::WhereOption::Where(format!("id>={iid} order by id asc")),
            &db,
        )
        .await
        .unwrap();
    assert_eq!(tuser.get(0).unwrap().nickname, nike_name);

    //test

    let tuser = select
        .fetch_all_scalar_by_where::<String, _>(
            "nickname",
            &sqlx_model::WhereOption::Where(format!("id>={iid} order by id asc")),
            &db,
        )
        .await
        .unwrap();
    assert_eq!(tuser.get(0).unwrap(), &nike_name);
}
