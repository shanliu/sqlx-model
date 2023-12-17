use sqlx::Pool;
use sqlx::{Database, Transaction};
use std::borrow::BorrowMut;

pub trait ExecutorOptionTransaction {
    fn as_copy(&mut self) -> &mut Self;
}

impl<'t, DB> ExecutorOptionTransaction for Transaction<'t, DB>
where
    DB: Database,
{
    fn as_copy(&mut self) -> &mut Self {
        self.borrow_mut()
    }
}

pub trait ExecutorOptionPool {
    fn as_copy(&self) -> &Self;
}

impl<DB> ExecutorOptionPool for Pool<DB>
where
    DB: Database,
{
    fn as_copy(&self) -> &Self {
        self
    }
}

#[macro_export]
/// 对包含块代码中的链接变量选择事物或连接池
/// @param $block 执行sql代码 里面可用 $execute 变量 多次使用 $execute.as_copy()
/// @param $transaction Option 当存在时$block中 $execute 变量用此值
/// @param $poll Option 不存在时 $execute 变量用此值
/// @param $execute  $block块中用到的连接变量名
macro_rules! executor_option {
    ($block:block,$transaction:expr,$poll:expr,$execute:tt) => {
        match $transaction {
            Some($execute) => {
                #[allow(unused_imports)]
                use $crate::ExecutorOptionTransaction;
                $block
            }
            None => {
                #[allow(unused_imports)]
                use $crate::ExecutorOptionPool;
                let $execute = $poll;
                $block
            }
        }
    };
}

#[test]
fn test_executor_option() {
    let va: Option<i32> = None;
    let vb = 1;
    let a = executor_option!({ aa }, va, vb, aa);
    assert!(a == 1);

    let va: Option<i32> = Some(2);
    let vb = 1;
    let a = executor_option!({ aa }, va, vb, aa);
    assert!(a == 2);
}
