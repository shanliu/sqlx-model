use crate::common::db_mysql;
use crate::common::UserModel;
use crate::common::UserModelRef;
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
        .fetch_one_by_where::<UserModel, _>(Some(sql_format!("id={}", iid)), &db)
        .await
        .unwrap();
    assert_eq!(user.nickname, nike_name);
    let (sql, bind_res) = sqlx_model::sql_bind!(sqlx::MySql, r#"nickname={nickname}"#);
    let user = select
        .fetch_one_by_where_call::<UserModel, _, _>(
            sql,
            |mut res, _| {
                sqlx_model::sql_bind_vars!(bind_res,res,{
                    "nickname":nike_name.clone()
                })
            },
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
            Some(format!("id={}", iid)),
            &db,
        )
        .await
        .unwrap();
    assert_eq!(tuser, nike_name);
    let tuser = select
        .fetch_one_scalar_by_where_call::<String, _, _>(
            "nickname",
            "id=?".to_string(),
            |mut res, _| {
                res = res.bind(iid);
                res
            },
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
        .fetch_all_by_where::<UserModel, _>(Some(format!("id>={} order by id asc", iid)), &db)
        .await
        .unwrap();
    assert_eq!(tuser.get(0).unwrap().nickname, nike_name);

    let tuser = select
        .fetch_all_by_where_call::<UserModel, _, _>(
            "id>=? order by id asc".to_string(),
            |mut b, _| {
                b = b.bind(iid);
                b
            },
            &db,
        )
        .await
        .unwrap();
    assert_eq!(tuser.get(0).unwrap().nickname, nike_name);

    //test

    let tuser = select
        .fetch_all_scalar_by_where::<String, _>(
            "nickname",
            Some(format!("id>={} order by id asc", iid)),
            &db,
        )
        .await
        .unwrap();
    assert_eq!(tuser.get(0).unwrap(), &nike_name);

    let tuser = select
        .fetch_all_scalar_by_where_call::<String, _, _>(
            "nickname",
            "id>=? order by id asc".to_string(),
            |mut b, _| {
                b = b.bind(iid);
                b
            },
            &db,
        )
        .await
        .unwrap();
    assert_eq!(tuser.get(0).unwrap(), &nike_name);
}
