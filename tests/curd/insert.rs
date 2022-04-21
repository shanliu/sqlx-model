use sqlx_model::{Insert,Update,ModelTableName};
use crate::common::db_mysql;
use crate::common::UserModelRef;
use crate::common::UserModel;
#[tokio::test]
async fn curd_insert(){
    
    let db=db_mysql().await;
    //---
    let nike_name="new insert".to_string();
    let gender=1;
    let userinsert=sqlx_model::model_option_set!(UserModelRef,{
        nickname:nike_name,
        gender:gender,
    });
    let i1=Insert::<sqlx::MySql,UserModel,_>::new(userinsert).execute(&db).await.unwrap();
    assert!(i1.last_insert_id()>0);

    //---
    let nike_name="new vec insert".to_string();
    let gender=1;
    let userinsert=vec![sqlx_model::model_option_set!(UserModelRef,{
        nickname:nike_name,
        gender:gender,
    })];
    let i2=Insert::<sqlx::MySql,UserModel,_>::new_vec(userinsert).execute(&db).await.unwrap();
    assert!(i2.last_insert_id()>0);

    //---
    let nickname="model insert".to_string();
    let i3=Insert::<sqlx::MySql,UserModel,_>::model(&UserModel{
        id:i2.last_insert_id() as u32+100,
        nickname:nickname.clone(),
        gender:1,
        headimg:Some("ddd".to_string()),
        password_id:1,
    }).execute(&db).await.unwrap();
    assert_eq!(i3.rows_affected(),1);

    //---
    let vec=vec![
        UserModel{
            id:i3.last_insert_id() as u32+101,
            nickname:"model vec".to_string(),
            gender:1,
            headimg:Some("ddd".to_string()),
            password_id:1,
        }
    ];
    let i4=Insert::<sqlx::MySql,UserModel,_>::model_vec(&vec).execute(&db).await.unwrap();
    assert_eq!(i4.rows_affected(),1);


    //---
    let umodel=UserModel{
            id:i3.last_insert_id() as u32+101,
            nickname:"model vec ".to_string(),
            gender:1,
            headimg:Some("ddd".to_string()),
            password_id:1,
        };
    let nike_name="model insert change".to_string();
    let userchange=sqlx_model::model_option_set!(UserModelRef,{
        nickname:nike_name,
    });
    let update=Update::<sqlx::MySql,UserModel,_>::new(userchange);
    let i5=Insert::<sqlx::MySql,UserModel,_>::model(&umodel).execute_update(update, &db).await.unwrap();
    assert_eq!(i5.rows_affected(),2);



     //---
     let umodel=UserModel{
        id:i3.last_insert_id() as u32+104,
        nickname:"model vec ".to_string(),
        gender:1,
        headimg:Some("ddd".to_string()),
        password_id:1,
    };
    let i6=Insert::<sqlx::MySql,UserModel,_>::model(&umodel).execute_by_sql(|me|{
        let table = UserModel::table_name();
        let vals = me.sql_values();
        let sql=format!(
            "INSERT INTO {} ({})VALUES {}",
            table.full_name(),
            me.fields.to_vec().join(","),
            vals.join(",")
        );
        sql
    }, &db).await.unwrap();
    assert_eq!(i6.rows_affected(),1);


    let nike_name="new insert".to_string();
    let gender=1;
    let userinsert=sqlx_model::model_option_set!(UserModelRef,{
        nickname:nike_name,
        gender:gender,
    });
    let i7=Insert::<sqlx::MySql,UserModel,_>::new(userinsert).execute_by_sql(|me|{
        let table = UserModel::table_name();
        let vals = me.sql_values();
        let sql=format!(
            "INSERT INTO {} ({})VALUES {}",
            table.full_name(),
            me.fields.to_vec().join(","),
            vals.join(",")
        );
        sql
    }, &db).await.unwrap();
    assert_eq!(i7.rows_affected(),1);

    let nike_name="new insert 8".to_string();
    let gender=1;
    let userinsert=sqlx_model::model_option_set!(UserModelRef,{
        nickname:nike_name,
        gender:gender,
    });
    let i8=Insert::<sqlx::MySql,UserModel,_>::new(userinsert).execute_by_sql_call(|me|{
        let table = UserModel::table_name();
        let vals = me.sql_param();
        let sql=format!(
            "INSERT INTO {} ({})VALUES {}",
            table.full_name(),
            me.fields.to_vec().join(","),
            vals.join(",")
        );
        sql
    },|mut res,insert|{
        res = insert.bind_values(res);
        res
    }, &db).await.unwrap();
    assert_eq!(i8.rows_affected(),1);


}