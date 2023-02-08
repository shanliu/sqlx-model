#[macro_export]
/// 从SQL中查找指定标记,返回用于查询的SQL跟待绑定的字段列表
/// 示例请查看单元测试
macro_rules! sql_bind {
    ($db:ty,$sql:expr) => {{
        let mut prev = None;
        let mut push_start = false;
        let mut key = vec![];
        let mut keys = vec![];
        let mut new_sql = vec![];
        for mstr in $sql.chars() {
            let mut is_r = false;
            if let Some(_prev) = prev {
                if _prev != '\\' {
                    is_r = true;
                }
            } else {
                is_r = true;
            }
            prev = Some(mstr);
            if is_r {
                if mstr == '{' {
                    push_start = true;
                    key = vec![];
                    continue;
                } else if mstr == '}' && push_start {
                    push_start = false;
                    keys.push(key.iter().collect::<String>());
                    key.clear();
                    new_sql.push('?');
                    continue;
                }
            }
            if push_start {
                key.push(mstr);
            } else {
                new_sql.push(mstr);
            }
        }
        (new_sql.into_iter().collect::<String>(), keys)
    }};
}
#[macro_export]
/// 简化sql_bind返回的待绑定列表的绑定变量操作
/// 可以自行实现此过程
macro_rules! sql_bind_vars {
    ($bind_res:expr,$res:expr,{$($key:literal:$bind:expr),+$(,)?})=>{
        {
            for key in $bind_res.iter() {
                match key.as_str(){
                    $(
                        $key=>{$res=$res.bind($bind)},
                    )+
                    _=>{},
                }
            }
            $res
        }
    };
}

#[test]
fn test_sql_bind_macro() {
    //!!!!!!注意:这是运行时的字符串替换,会有一定的性能损耗!!!!!
    //以下一般用于标识变量名的SQL,统一各种不同数据库SQL差异
    //因为如MYSQL的变量绑定为?符号,数量多了出问题难查找,指定名称方便识别

    let (sql, bind_res) = crate::sql_bind!(
        sqlx::MySql,
        r#"
            select * from (SELECT {nickname} as nickname,{gender} as gender,1 as gender_group,{nickname} as nickname1,{gender} as gender1 ) as t where gender in (1) and gender_group in ({gender_group})
        "#
    );
    assert_eq!(
        "select * from (SELECT ? as nickname,? as gender,1 as gender_group,? as nickname1,? as gender1 ) as t where gender in (1) and gender_group in (?)",
        sql.as_str().trim()
    );
    //上面查询出5个变量位置,3个不同绑定值
    assert_eq!(bind_res.len(), 5);
    //Res 是模拟SQLX的资源用于测试
    struct Res {}
    impl Res {
        fn bind<T>(self, _: T) -> Self {
            self
        }
    }
    let mut res = Res {};
    //将上面查询出的变量位置进行变量绑定
    crate::sql_bind_vars!(bind_res,res,{
        "nickname":1,
        "gender":"ddd",
        "gender_group":3
    });
}
