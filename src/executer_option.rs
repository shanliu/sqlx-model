use std::borrow::BorrowMut;
use std::borrow::Borrow;
use sqlx::Pool;
use sqlx::{Database, Transaction};




pub trait ExecutorOptionTransaction {
    fn as_copy(&mut self)->&mut Self;
}

impl<'t,DB> ExecutorOptionTransaction for Transaction<'t,DB> where
DB: Database, {
    fn as_copy(&mut self)->&mut Self {
        self.borrow_mut()
    }
}

pub trait ExecutorOptionPool {
    fn as_copy(&self)->&Self;
}

impl<DB> ExecutorOptionPool for Pool<DB> where
DB: Database, {
    fn as_copy(&self)->&Self {
        self.borrow()
    }
}



#[macro_export]
macro_rules! executor_option {
    ($block:block,$transaction:expr,$poll:expr,$execute:tt)=>{
        match $transaction {
            Some($execute)=>{
                use $crate::ExecutorOptionTransaction;
                $block
            }
            None=>{
                use $crate::ExecutorOptionPool;
                let $execute=$poll;
                $block
            }
        }
    };
}



#[test]
fn test_executor_option(){
    let va:Option<i32>=None;
    let vb=1;
    let a=executor_option!({
        aa
    },va,vb,aa);
    assert!(a==1);
    
    let va:Option<i32>=Some(2);
    let vb=1;
    let a=executor_option!({
        aa
    },va,vb,aa);
    assert!(a==2);

}