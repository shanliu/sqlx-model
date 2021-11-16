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


#### 引入

> 使用 default-features = false 禁用默认使用的tokio,自行选择运行时跟数据库类型

```toml
[dependencies]
sqlx-model = { version = "~0.0.1-beta.15", path = "../../",default-features = false,features = ["sqlx-mysql"] }
sqlx = {version = "~0.5",features = [ "mysql","offline","runtime-async-std-native-tls"] }
async-std={version = "1.10.0", features = [ "attributes" ]}
```

##### 常用增删改查示例

使用前准备，结构体增加derive宏
> 使用 sqlx_model::SqlxModel 宏 自动增加辅助方法
> 同时会创建 UserModelRef 的结构，用于辅助增删改查操作

> 可以通过 `https://github.com/shanliu/db-to-code` 生成以下文件，具体生成方式参考 db-to-code 说明

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
    let user=select.fetch_one_by_where::<UserModel>(Some(format!("id=1")), &db).await.unwrap();
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
    let update=Update::<sqlx::MySql,UserModel,_>::new(userchange);
    let update=update.execute_by_scalar_pk(1,&db).await.unwrap();
    assert_eq!(update.rows_affected(),1);
```

4. 查询：

> 更多使用方法参考 tests 目录

```rust
    let select=Select::type_new::<UserModel>();
    let user=select.fetch_one_by_scalar_pk::<UserModel,_>(iid, &db).await.unwrap();
    assert_eq!(user.id as u64,iid);
```


##### 辅助SQL生成操作

> 绑定SQL查询可以用到SQLX内部查询SQL缓存,会快少许

1. 自动转义SQL生成

```rust
     let data=["dd","b'bb"];
     let password_id=Some(1);
     let sql=sql_format!(
            "select * from yaf_users where id>{id} {in_grade} and password_id {password_where} ",
            id=1,
            in_grade=sql_array_str!("and grade in ({})",data),
            password_where=sql_option_str!("= {}","is {}",password_id)
        );
     println!("{}",sql);//select * from yaf_users where id>1 and grade in ('dd','b\'bb') and password_id = 1
    //会转义'防止sql注入
```

2. 绑定SQL方式SQL生成

```rust
    let (sql,bind_res)=sqlx_model::sql_bind!(sqlx::MySql,"id>{id}");
    let _= Select::type_new::<UserModel>().fetch_one_by_where_call::<UserModel,_>(sql,|mut query_res,_|{
        sqlx_model::sql_bind_vars!(bind_res,query_res,{
            "id":1
        })
    }, &pool).await.unwrap();
```
