<div align="center">
<h3>基于 sqlx 轻量级orm实现</h3>
</div>

<div align="center">
  <a href="https://crates.io/crates/sqlx-model">
    <img src="https://img.shields.io/crates/v/sqlx-model.svg?style=flat-square"
    alt="Crates.io version" />
  </a>
  <a href="https://docs.rs/sqlx-model">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
      alt="docs.rs docs" />
  </a>
</div>

0.1.*版本升级0.2.* 升级提醒
```
1. 部分函数的String类型改为传入&str,为了提示效率 [调用方需加个&]
2. 为了支持当不存在where条件且需要排序等操作,更换了WHERE入参的类型,涉及以下函数:
   [Select]fetch_all_by_where [Update] execute_by_where [Delete] execute_by_where
   由 Option 改为了 WhereOption [调用方需修改调用],如下例:
   select.fetch_one_by_where::<UserModel>(Some(format!("id=1")), &db).await.unwrap();
   修改为:select.fetch_one_by_where::<UserModel>(WhereOption::Where(format!("id=1")), &db).await.unwrap();
```

##### 引入

> 使用 default-features = false 禁用默认使用的tokio,自行选择运行时跟数据库类型

```toml
[dependencies]
sqlx-model = { version = "~0.2.0", path = "../../",default-features = false,features = ["sqlx-mysql"] }
sqlx = {version = "~0.6",features = [ "mysql","offline","runtime-async-std-native-tls"] }
async-std={version = "1.10.0", features = [ "attributes" ]}
```

##### 常用增删改查示例

使用前准备，结构体增加derive宏
> 使用 sqlx_model::SqlxModel 宏 自动增加辅助方法
> 同时会创建 UserModelRef 的结构，用于辅助增删改查操作


##### 如果已有表结构,可以通过表结构生成对应的`rs`model文件,以下工具可用:

> https://crates.io/crates/sqlx-model-tools 具体使用方式参考[该create文档](./sqlx-model-tools)


```rust
#[derive(sqlx::FromRow,sqlx_model::SqlxModel,Clone,Debug)]
//#[sqlx(rename_all="lowercase")] //按规则自定义字段名
#[sqlx_model(table_pk="id")]//自定义表主键，不指定默认第一个字段
#[sqlx_model(table_name="users")]//自定义关联表名，不指定为去除Model后的user
pub struct UserModel {
    #[sqlx(default)]
    pub id: u32,
    #[sqlx(default)]
    pub nickname: String,
    #[sqlx(default)]
    pub gender: u8,
    #[sqlx(default)]
    pub headimg: Option<String>,
    #[sqlx(default)]
    #[sqlx(rename="password_id")]//自定义字段名
    pub password_id: u32,
}
```

1. 新增：

> 更多使用方法参考 tests 目录

```rust
    let nike_name="new insert".to_string();
    let gender=1;
    let userinsert=sqlx_model::model_option_set!(UserModelRef,{
        nickname:nike_name,
        gender:gender,
        //不需要全部字段赋值，没赋值生成SQL会少对应字段，等于用表中默认值
    });
    let i1=Insert::<sqlx::MySql,UserModel,_>::new(userinsert).execute(&db).await.unwrap();
    assert!(i1.last_insert_id()>0);
```

2. 删除：

> 更多使用方法参考 tests 目录

```rust
    let select=Select::type_new::<UserModel>();
    let user=select.fetch_one_by_where::<UserModel>(&WhereOption::Where(format!("id=1")), &db).await.unwrap();
    let detete=Delete::<sqlx::MySql>::new(UserModel::table_name())
        .execute_by_pk(&user, &db)
        .await.unwrap();
    assert_eq!(detete.rows_affected(),1);
```

3. 修改：

> 更多使用方法参考 tests 目录

```rust
    let nike_name="change to 1".to_string();
    let userchange=sqlx_model::model_option_set!(UserModelRef,{
    nickname:nike_name,
    });
    let update=Update::<sqlx::MySql,UserModel,_,_>::new(userchange);
    let update=update.execute_by_scalar_pk(1,&db).await.unwrap();
    assert_eq!(update.rows_affected(),1);
```

4. 查询：

> 更多使用方法参考 tests 目录

```rust
    let iid=1;
    let select=Select::type_new::<UserModel>();
    let user=select.fetch_one_by_scalar_pk::<UserModel,_,_>(iid, &db).await.unwrap();
    assert_eq!(user.id as u64,iid);
```

5. 事务:

> 更多使用方法参考 tests 目录

```rust
    let mut ta=db.begin().await.unwrap();
    let nike_name="new tran".to_string();
    let userinsert=sqlx_model::model_option_set!(UserModelRef,{
        nickname:nike_name,
        gender:11,
    });
    Insert::<sqlx::MySql,UserModel,_>::new(userinsert).execute(&mut ta).await.unwrap();
    //其他 查删改操作...
    ta.commit().await.unwrap();
```

6. 事务跟Poll选择执行

```rust 
fn my_exec(transaction:Option<&mut Transaction<'t,sqlx::MySql>>){
    let pool=get_db_pool();
    let res=executor_option!({
        //  transaction 为 None 用 &pool 代替 db 如果 [因为execute为泛型且为&mut,多次时需要手动调用as_copy]
        //  否则为 transaction 里的值代替 db
        Insert::<sqlx::MySql, UserEmailModel, _>::new(idata).execute(db.as_copy()).await?
    },transaction,&pool,db);
}
```

7. 日期及其他自定义字段类型支持示例

```rust
use chrono::{DateTime, Datelike, TimeZone, Timelike, Utc};
use sqlx::FromRow;
use sqlx_model::{SqlQuote, SqlxModel};
use std::ops::Deref;
#[derive(sqlx::Type, Clone, Debug, PartialEq, Eq)]
#[sqlx(transparent)]
pub struct MyTime<Tz: TimeZone>(DateTime<Tz>);
impl<Tz: TimeZone> Deref for MyTime<Tz> {
    type Target = DateTime<Tz>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
 //其他自定义结构需实现 SqlQuote<T> 
 //其中 T 为sqlx支持类型[如String,i32,i64...等]
impl<Tz: TimeZone> SqlQuote<String> for MyTime<Tz> {
    fn sql_quote(&self) -> String {
        format!(
            "{}-{}-{} {}:{}:{}",
            self.0.year(),
            self.0.month(),
            self.0.day(),
            self.0.hour(),
            self.0.minute(),
            self.0.second()
        )
    }
}

#[derive(FromRow, Clone, Debug, SqlxModel)]
pub struct UserModel {
    #[sqlx(default)]
    pub id: u64,
    pub id1: MyTime<Utc>,
}
```

##### 辅助SQL生成操作

```rust
     let data=["dd","b'bb"];
     let password_id=Some(1);
     let sql=sql_format!(
            "select * from yaf_users where id>{id} {in_grade} and password_id {password_where} ",
            id=1,
            in_grade=sql_array_str!("and grade in ({})",data),
            password_where=sql_option_str!("= {}","is {}",password_id)
        );
     println!("{sql}");//select * from yaf_users where id>1 and grade in ('dd','b\'bb') and password_id = 1
    //会转义'防止sql注入
```
