use crate::curd_mysql::common::db_mysql;
use crate::curd_mysql::common::UserModel;
use crate::curd_mysql::common::UserModelRef;
use sqlx::Acquire;
use sqlx::Transaction;
use sqlx_model::Insert;
#[tokio::test]
async fn curd_tran() {
    let db = db_mysql().await;

    test1(None).await;

    let mut ta = db.begin().await.unwrap();
    //---
    let str = "bbbb".to_string();
    let userinsert = sqlx_model::model_option_set!(UserModelRef,{
        nickname:str,
        gender:11,
    });
    let i1 = Insert::<sqlx::MySql, UserModel, _>::new(userinsert)
        .execute(&mut ta)
        .await
        .unwrap();
    assert!(i1.last_insert_id() > 0);

    test1(Some(&mut ta)).await;

    //---
    let nike_name = "new vec tran".to_string();
    let gender = 1;
    let userinsert = vec![sqlx_model::model_option_set!(UserModelRef,{
        nickname:nike_name,
        gender:gender,
    })];
    let i2 = Insert::<sqlx::MySql, UserModel, _>::new_vec(userinsert)
        .execute(&mut ta)
        .await
        .unwrap();
    assert!(i2.last_insert_id() > 0);
    //ta.rollback().await.unwrap();
    ta.commit().await.unwrap();
}

async fn test1<'c>(ta: Option<&mut Transaction<'c, sqlx::MySql>>) {
    let db = db_mysql().await;
    let mut k = match ta {
        Some(pb) => pb.begin().await.unwrap(),
        None => db.begin().await.unwrap(),
    };
    let str = "bbbb".to_string();
    let userinsert = sqlx_model::model_option_set!(UserModelRef,{
        nickname:str,
        gender:11,
    });
    let i1 = Insert::<sqlx::MySql, UserModel, _>::new(userinsert)
        .execute(&mut k)
        .await
        .unwrap();
    assert!(i1.last_insert_id() > 0);
    k.commit().await.unwrap();
}
