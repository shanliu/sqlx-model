use sqlx::Transaction;
use sqlx_model::{Insert};
use crate::common::db_mysql;
use crate::common::UserModelRef;
use crate::common::UserModel;
#[tokio::test]
async fn curd_tran(){
    
    let db=db_mysql().await;

    let mut ta=db.begin().await.unwrap();
    //---
    let str="bbbb".to_string();
    let userinsert=sqlx_model::model_option_set!(UserModelRef,{
        nickname:str,
        gender:11,
    });
    let i1=Insert::<sqlx::MySql,UserModel,_>::new(userinsert).execute(&mut ta).await.unwrap();
    assert!(i1.last_insert_id()>0);


    test1(Some(&mut ta)).await;
    //---
    let nike_name="new vec tran".to_string();
    let gender=1;
    let userinsert=vec![sqlx_model::model_option_set!(UserModelRef,{
        nickname:nike_name,
        gender:gender,
    })];
    let i2=Insert::<sqlx::MySql,UserModel,_>::new_vec(userinsert).execute(&mut ta).await.unwrap();
    assert!(i2.last_insert_id()>0);
    //ta.rollback().await.unwrap(); 
    ta.commit().await.unwrap();

}



async fn test1<'c>(ta:Option<&mut Transaction<'c,sqlx::MySql>>){
    let db=db_mysql().await;
    let tta;
    let mut k=None;
    match ta {
        Some(a)=>{
            tta=a;
        }
        None=>{
            k=Some(db.begin().await.unwrap());
            tta=k.as_mut().unwrap();
        }
    };
   
    let str="bbbb".to_string();
    let userinsert=sqlx_model::model_option_set!(UserModelRef,{
        nickname:str,
        gender:11,
    });
    let i1=Insert::<sqlx::MySql,UserModel,_>::new(userinsert).execute( tta).await.unwrap();
    assert!(i1.last_insert_id()>0);

    if let Some(s)=k {
        s.commit().await.unwrap();
    }
}

