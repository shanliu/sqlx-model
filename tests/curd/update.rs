use sqlx_model::{ModelTableName, Select};
use sqlx_model::{Insert,Update,SqlQuote,sql_format};
use crate::common::db_mysql;
use crate::common::UserModelRef;
use crate::common::UserModel;
#[tokio::test]
async fn curd_update(){
    let db=db_mysql().await;
    //---
    let nike_name="new vec update".to_string();
    let gender=1;
    let userinsert=sqlx_model::model_option_set!(UserModelRef,{
        nickname:nike_name,
        gender:gender,
        password_id:1
    });
    let update_id=Insert::<sqlx::MySql,UserModel,_>::new(userinsert).execute(&db).await.unwrap().last_insert_id();


     //test example
     let nike_name="change to 1".to_string();
     let userchange=sqlx_model::model_option_set!(UserModelRef,{
        nickname:nike_name,
     });
    let update=Update::<sqlx::MySql,UserModel,_>::new(userchange);
    let update=update.execute_by_scalar_pk(update_id,&db).await.unwrap();
    assert_eq!(update.rows_affected(),1);


     //test example
     let nike_name="change to 2".to_string();
     let userchange=sqlx_model::model_option_set!(UserModelRef,{
        nickname:nike_name,
     });
     let update=Update::<sqlx::MySql,UserModel,_>::new(userchange);
     let update=update.execute_by_where(Some(sql_format!("id={}",update_id)),&db).await.unwrap();
     assert_eq!(update.rows_affected(),1);


    
    //test example
    let nike_name="change to 3".to_string();
    let userchange=sqlx_model::model_option_set!(UserModelRef,{
        nickname:nike_name,
    });
    let update=Update::<sqlx::MySql,UserModel,_>::new(userchange);
    let update=update.execute_by_where_call("id=? and password_id=?",|mut res,_|{
        res = res.bind(update_id);
        res = res.bind(1);
        res
    },&db).await.unwrap();
    assert_eq!(update.rows_affected(),1);


     //test find and change
    let select=Select::type_new::<UserModel>();
    let source=select.fetch_one_by_scalar_pk::<UserModel,_,_>(update_id, &db).await.unwrap();
    //may be dobule check source data
    let myset_data=&UserModel{
        nickname:"change to 4".to_string(),
        gender:2,
        ..source.clone()
    };
    let update=Update::model(myset_data, Some(&source));
    let update=update.execute_by_scalar_pk(update_id,&db).await.unwrap();
    assert_eq!(update.rows_affected(),1);


     //test find and change
     let select=Select::type_new::<UserModel>();
     let source=select.fetch_one_by_scalar_pk::<UserModel,_,_>(update_id, &db).await.unwrap();
     //may be dobule check source data
     let nike_name="change to 5".to_string();
     let userchange=sqlx_model::model_option_set!(UserModelRef,{
        nickname:nike_name,
     });
     let update=Update::<sqlx::MySql,UserModel,_>::new(userchange);
     let update=update.execute_by_pk(&source,&db).await.unwrap();
     assert_eq!(update.rows_affected(),1);
 
 
    
    //test example
    let nike_name="change to 6".to_string();
    let userchange=sqlx_model::model_option_set!(UserModelRef,{
        nickname:nike_name,
    });
    let update=Update::<sqlx::MySql,UserModel,_>::new(userchange);
    let sql={
        let table = UserModel::table_name();
        let values = update.sql_values_sets();
        let sql = format!(
            "UPDATE {} SET {} WHERE {}",
            table.full_name(),
            values,
            sql_format!("id={}",update_id)
        );
        sql
    };
    let update =sqlx::query(sql.as_str()).execute(&db).await.unwrap();
    assert_eq!(update.rows_affected(),1);


    //test example
    let nike_name="change to 7".to_string();
    let userchange=sqlx_model::model_option_set!(UserModelRef,{
        nickname:nike_name,
    });
    let update=Update::<sqlx::MySql,UserModel,_>::new(userchange);
    let sql={
        let table = UserModel::table_name();
        let values = update.sql_sets();
        let sql = format!(
            "UPDATE {} SET {} WHERE {}",
            table.full_name(),
            values,
            sql_format!("id={}",update_id)
        );
        sql
    };
    let mut res=sqlx::query(sql.as_str());
    res = update.bind_values(res);
    let update =res.execute(&db).await.unwrap();
    assert_eq!(update.rows_affected(),1);

}