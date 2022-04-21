use sqlx::Database;
use sqlx::Executor;
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


    test1(Some(&mut ta));
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



async fn test1<'c>(mut ta:Option<&mut Transaction<'c,sqlx::MySql>>){
    let db=db_mysql().await;
    let ba=None;
    let tta=match ta {
        Some(bb)=>bb,
        None=>{
            &mut db.begin().await.unwrap()
        }
    };
    
    //let tta=ta.unwrap();
    
    
    //---
    let str="bbbb".to_string();
    let userinsert=sqlx_model::model_option_set!(UserModelRef,{
        nickname:str,
        gender:11,
    });
    let i1=Insert::<sqlx::MySql,UserModel,_>::new(userinsert).execute( tta).await.unwrap();
    assert!(i1.last_insert_id()>0);

    // //---
    // let nike_name="new vec tran".to_string();
    // let gender=1;
    // let userinsert=vec![sqlx_model::model_option_set!(UserModelRef,{
    //     nickname:nike_name,
    //     gender:gender,
    // })];
    // let i2=Insert::<sqlx::MySql,UserModel,_>::new_vec(userinsert).execute(tta).await.unwrap();
    // assert!(i2.last_insert_id()>0);
    //ta.rollback().await.unwrap(); 
    // if ta.is_none() {
    //     tta.commit().await.unwrap();
    // }
}

