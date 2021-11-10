use sqlx_model::{Insert,Delete, ModelTableName, Select,sql_format,SqlQuote};
use crate::common::db_mysql;
use crate::common::UserModel;
use crate::common::UserModelRef;
#[tokio::test]
async fn curd_delete(){
    
    let db=db_mysql().await;

    //test
    let nike_name="new vec delete".to_string();
    let gender=1;
    let userinsert=sqlx_model::model_option_set!(UserModelRef,{
        nickname:nike_name,
        gender:gender,
        password_id:1
    });
    let _=Insert::<sqlx::MySql,UserModel,_>::new(userinsert).execute(&db).await.unwrap().last_insert_id();

    let select=Select::type_new::<UserModel>();
    let user=select.fetch_one_by_where::<UserModel>(None, &db).await.unwrap();
    let detete=Delete::<sqlx::MySql>::new(UserModel::table_name())
        .execute_by_pk(&user, &db)
        .await.unwrap();
    assert_eq!(detete.rows_affected(),1);
    
    //test
    let nike_name="new vec delete".to_string();
    let gender=1;
    let userinsert=sqlx_model::model_option_set!(UserModelRef,{
        nickname:nike_name,
        gender:gender,
        password_id:1
    });
    let update_id=Insert::<sqlx::MySql,UserModel,_>::new(userinsert).execute(&db).await.unwrap().last_insert_id();
    let detete=Delete::<sqlx::MySql>::new(UserModel::table_name())
        .execute_by_scalar_pk::<UserModel,_>(update_id, &db)
        .await.unwrap();
    assert_eq!(detete.rows_affected(),1);


    //test
    let nike_name="new vec delete".to_string();
    let gender=1;
    let userinsert=sqlx_model::model_option_set!(UserModelRef,{
        nickname:nike_name,
        gender:gender,
        password_id:1
    });
    let update_id=Insert::<sqlx::MySql,UserModel,_>::new(userinsert).execute(&db).await.unwrap().last_insert_id();

    let sql=sql_format!("id={}",update_id);
    let detete=Delete::<sqlx::MySql>::new(UserModel::table_name())
        .execute_by_where(Some(sql), &db)
    .await.unwrap();
    assert_eq!(detete.rows_affected(),1);

    //test
    let nike_name="new vec delete".to_string();
    let gender=1;
    let userinsert=sqlx_model::model_option_set!(UserModelRef,{
        nickname:nike_name,
        gender:gender,
        password_id:1
    });
    let update_id=Insert::<sqlx::MySql,UserModel,_>::new(userinsert).execute(&db).await.unwrap().last_insert_id();
    let detete=Delete::<sqlx::MySql>::new(UserModel::table_name())
        .execute_by_where_call( "id=?", |mut res,_|{
            res = res.bind(update_id);
            res
        },&db)
    .await.unwrap();
    assert_eq!(detete.rows_affected(),1);

}