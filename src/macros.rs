#[macro_export]
/// 对指定结构体实现名为 $option_struct_name 的可选引用struct
/// @param $struct_name 结构体名
/// @param $option_struct_name 更改值临时存储的结构体名
/// @param {$name:$type} 字段名列表:类型列表
macro_rules! model_table_ref_define {
    ($self_var:ident,$struct_name:ident,$option_struct_name:ident,{$($name:ident[$column_name:literal]:$type:ty),+})=>{
        #[derive(PartialEq,Eq,Debug)]
        pub struct $option_struct_name<'t> {
            $(pub $name:Option<&'t $type>),*
        }
        impl<'t> $option_struct_name<'t> {
            #[allow(dead_code)]
            pub fn none_default()->Self{
                $option_struct_name {
                    $($name:None),*
                }
            }
        }
        impl<'t> $crate::InsertData<'t,sqlx::MySql> for $option_struct_name<'t>
        {
            fn columns(&$self_var) -> Vec<$crate::FieldItem> {
                let mut vec = vec![];
                $(
                    if !$self_var.$name.is_none() {
                        vec.push($crate::FieldItem::new(stringify!($name),$column_name));
                    }
                ) *
                vec
            }
            fn sqlx_bind<'q>(&'q
                $self_var,
                field:&$crate::FieldItem,
                mut res: sqlx::query::Query<'q,sqlx::MySql,<sqlx::MySql as sqlx::database::HasArguments<'q>>::Arguments>,
            ) -> sqlx::query::Query<'q,sqlx::MySql,<sqlx::MySql as sqlx::database::HasArguments<'q>>::Arguments>{
                $crate::model_table_value_bind_define!(value_bind $self_var, res, field, {$($name),+});
            }
            fn sqlx_string(&$self_var,
                field:&$crate::FieldItem
            ) -> Option<String>{
                use $crate::SqlQuote;
                match field.name.as_str() {
                    $(
                        stringify!($name)=> {
                            Some($self_var.$name.map_or("".to_string(),|e|{
                                e.sql_quote().to_string()
                            }))
                        }
                    ) *
                    _=>None
                }
            }
        }
        impl<'t> $crate::ModelInsertData<'t,sqlx::MySql,$option_struct_name<'t>> for $struct_name
        {
            fn insert_data(&'t $self_var) -> $option_struct_name<'t>{
                $option_struct_name {
                    $(
                       $name:Some(&$self_var.$name)
                    ),*
                }
            }
        }
        impl<'t> $crate::UpdateData<'t,sqlx::MySql> for $option_struct_name<'t>
        {
            fn diff_columns(&$self_var) -> Vec<$crate::FieldItem> {
                let mut vec = vec![];
                $(
                    if !$self_var.$name.is_none() {
                        vec.push($crate::FieldItem::new(stringify!($name),$column_name));
                    }
                ) *
                vec
            }
            fn sqlx_bind<'q>(&'q
                $self_var,
                mut res: sqlx::query::Query<'q,sqlx::MySql,<sqlx::MySql as sqlx::database::HasArguments<'q>>::Arguments>,
            ) -> sqlx::query::Query<'q,sqlx::MySql,<sqlx::MySql as sqlx::database::HasArguments<'q>>::Arguments>
            {
                $(
                    if let Some(val) = $self_var.$name {
                        res = res.bind(val.clone());
                    }
                ) *
                res
            }
            fn sqlx_string(&$self_var,field:&$crate::FieldItem) -> Option<String>
            {
                use $crate::SqlQuote;
                match(field.name.as_str()){
                    $(
                        stringify!($name)=>{
                            if let Some(val) = $self_var.$name {
                                return Some(val.sql_quote().to_string())
                            }
                        }
                    ) *
                    _=>{}
                }
                None
            }
        }
        impl<'t> $crate::ModelUpdateData<'t,sqlx::MySql, $option_struct_name<'t>> for $struct_name
        {
            fn diff(&'t $self_var, source_opt: &Option<&Self>) -> $option_struct_name<'t> {

                match source_opt {
                    Some(source) => {
                        $option_struct_name {$(
                            $name: if $self_var.$name != source.$name {
                                Some(&$self_var.$name)
                            } else {
                                None
                            }
                        ),*}
                    }
                    None => $option_struct_name {
                        $(
                           $name:Some(&$self_var.$name)
                        ),*
                    },
                }
            }
        }
    };
    ($struct_name:ident,$option_struct_name:ident,{$($name:ident[$column_name:literal]:$type:ty),+$(,)?})=>{
        $crate::model_table_ref_define!(self,$struct_name,$option_struct_name,{$($name[$column_name]:$type),+});
    };
}

#[macro_export]
/// 对指定结构体 ModelTableName ModelTableField
/// @param $struct_name 结构体名
/// @param $table_name 表名
/// @param {$name} 字段名列表
/// @param {$pk_name} 主键字段名列表
macro_rules! model_table_value_bind_define {
    (value_bind $self_var:ident,$res:expr,$val:expr,{$($name:ident),+})=>{
            match $val.name.as_str() {
                $(
                    stringify!($name)=> {
                        $res=$res.bind(&$self_var.$name);
                    }
                ) *
                _=>{}
            }
            return $res
    };
    ($self_var:ident,$struct_name:ident,$table_name:expr,{$($name:ident[$column_name:literal]),+},{$($pk_name:ident[$pk_column_name:literal]),+})=>{
        impl $crate::ModelTableName for $struct_name {
            fn table_name() -> $crate::TableName {
                $crate::TableName::new($table_name)
            }
        }
        impl $crate::ModelTableField<sqlx::MySql> for $struct_name{
            fn table_pk() -> $crate::TableFields {
                $crate::TableFields::new(vec![
                    $(
                        $crate::FieldItem::new(stringify!($pk_name),$pk_column_name)
                    ),*
                ])
            }
            fn table_column() -> $crate::TableFields {
                $crate::TableFields::new(vec![
                    $(
                        $crate::FieldItem::new(stringify!($name),$column_name)
                    ),*
                ])
            }
            fn query_sqlx_bind<'t>(
                &'t
                $self_var,
                field_val: &$crate::FieldItem,
                mut res: sqlx::query::Query<'t,sqlx::MySql,<sqlx::MySql as sqlx::database::HasArguments<'t>>::Arguments>,
            ) -> sqlx::query::Query<'t,sqlx::MySql,<sqlx::MySql as sqlx::database::HasArguments<'t>>::Arguments>
            {
                $crate::model_table_value_bind_define!(value_bind $self_var, res, field_val, {$($name),+});
            }
            fn query_as_sqlx_bind<'t, M>(
                &'t $self_var,
                field_val: &$crate::FieldItem,
                mut res:  sqlx::query::QueryAs<'t,sqlx::MySql, M,<sqlx::MySql as sqlx::database::HasArguments<'t>>::Arguments>,
            ) -> sqlx::query::QueryAs<'t,sqlx::MySql, M,<sqlx::MySql as sqlx::database::HasArguments<'t>>::Arguments>
            where
                for<'r> M: sqlx::FromRow<'r, sqlx::mysql::MySqlRow> + Send + Unpin,
            {
                $crate::model_table_value_bind_define!(value_bind $self_var, res, field_val,{$($name),+});
            }
        }
    };
    ($struct_name:ident,$table_name:expr,{$($name:ident[$column_name:literal]),+},{$($pk_name:ident[$pk_column_name:literal]),+$(,)?})=>{
        $crate::model_table_value_bind_define!(self ,$struct_name,$table_name,{$($name[$column_name]),+},{$($pk_name[$pk_column_name]),+});
    };
}

#[test]
fn test_model_define_bind_macro() {
    pub struct UserModel {
        pub id: u32,
        pub nickname: String,
        pub gender: u8,
        pub headimg: Option<String>,
        pub password_id: u32,
    }
    crate::model_table_value_bind_define!(UserModel,"user",{
        id["id"],
        nickname["nickname"],
        gender["gender"],
        headimg["headimg"],
        password_id["password_id"]
    },{
        id["id"]
    });
    crate::model_table_ref_define!(UserModel,UserModelRef,{
        id["id"]: u32,
        nickname["nickname"]: String,
        gender["gender"]: u8,
        headimg["headimg"]: Option<String>,
        password_id["password_id"]: u32,
    });
}

#[macro_export]
/// 对实现 none_default 方法的struct 用指定键值对快速创建结构 可由model_table_ref_define实现
/// @param $struct_name 结构体名
/// @param $key 字段名
/// @param $val 数据
macro_rules! model_option_set {
    ($struct_name:ident,{$($key:ident:$val:expr),*$(,)?})=>{
        {
            $struct_name{
                $(
                    $key:Some(&$val),
                )*
                ..$struct_name::none_default()
            }
        }
    };
}

#[macro_export]
/// 对实现 none_default 方法的struct 通过指定变量跟关系快速创建结构 可由model_table_ref_define实现
/// @param $struct_name 结构体名
/// @param $var 数据来源变量
/// @param $key 字段映射关系,例如:0=>fieldname
macro_rules! model_option_map {
    ($struct_name:ident,$var:expr,{$($key:ident),*$(,)?})=>{
        {
            $struct_name{
                $(
                    $key:Some(&$var.$key),
                )*
                ..$struct_name::none_default()
            }
        }
    };
    ($struct_name:ident,$var:expr,{$($from_key:tt=>$to_key:ident),*$(,)?})=>{
        {
            $struct_name{
                $(
                    $to_key:Some(&$var.$from_key),
                )*
                ..$struct_name::none_default()
            }
        }
    };
}

#[test]
fn test_model_option_macro() {
    #[derive(Clone, Debug)]
    #[allow(dead_code)]
    struct UserModel {
        id: u32,
        nickname: String,
        gender: u8,
        headimg: String,
        password_id: u32,
    }
    #[derive(PartialEq, Eq, Debug)]
    struct UserModelOption<'t> {
        id: Option<&'t u32>,
        nickname: Option<&'t String>,
        gender: Option<&'t u8>,
        headimg: Option<&'t String>,
        password_id: Option<&'t u32>,
    }
    impl<'t> UserModelOption<'t> {
        pub fn none_default() -> Self {
            UserModelOption {
                nickname: None,
                gender: None,
                id: None,
                headimg: None,
                password_id: None,
            }
        }
    }

    //test
    let tvar1 = ("option insert".to_string(), 1);
    let tmp = crate::model_option_map!(UserModelOption,tvar1,{0=>nickname,1=>gender});
    assert_eq!(tmp.nickname.unwrap(), &tvar1.0);
    assert_eq!(tmp.gender.unwrap(), &tvar1.1);

    //test
    struct Tvar {
        a: String,
        b: u8,
    }
    let tvar1 = Tvar {
        a: "option insert".to_string(),
        b: 1,
    };
    let tmp = crate::model_option_map!(UserModelOption,tvar1,{a=>nickname,b=>gender});
    assert_eq!(
        tmp,
        UserModelOption {
            nickname: Some(&tvar1.a),
            gender: Some(&tvar1.b),
            id: None,
            headimg: None,
            password_id: None
        }
    );

    //test
    let nike_name = "option insert".to_string();
    let gender = 1;
    let userinsert = crate::model_option_set!(UserModelOption,{
        nickname:nike_name,
        gender:gender,
    });
    assert_eq!(
        userinsert,
        UserModelOption {
            nickname: Some(&tvar1.a),
            gender: Some(&tvar1.b),
            id: None,
            headimg: None,
            password_id: None
        }
    );

    //test
    struct TVAR1 {
        nickname: String,
        gender: u8,
    }
    let tvar1 = TVAR1 {
        nickname: "option insert".to_string(),
        gender: 1,
    };
    let tmp = crate::model_option_map!(UserModelOption,tvar1,{nickname,gender});
    assert_eq!(
        tmp,
        UserModelOption {
            nickname: Some(&tvar1.nickname),
            gender: Some(&tvar1.gender),
            id: None,
            headimg: None,
            password_id: None
        }
    );
}

#[macro_export]
/// 对状态类型的结构提供辅助方法
/// @param $enum_name 状态枚举
/// @param $type 状态的类型
/// @param $item 可选值列表
macro_rules! model_enum_status_define {
    ($self_var:ident,$enum_name:ident,$type:ty,{$($item:expr),*$(,)?})=>{
        impl $enum_name{
            pub fn eq(self,eq:$type)->bool{
                return self.to()==eq;
            }
            pub fn to(self)->$type{
                return self as $type
            }
        }
		impl $crate::SqlQuote<$type> for $enum_name {
			fn sql_quote(&self) -> $type {
				*self as $type
			}
		}
        impl std::convert::TryFrom<$type> for $enum_name {
            type Error=sqlx::Error;
            fn try_from(value:  $type) -> Result<Self, Self::Error> {
                $(
                    if ($item as $type) ==value {
                        return Ok($item);
                    }
                )*
                return Err(sqlx::Error::TypeNotFound { type_name: format!("{}[{}]->{}",stringify!(i8),value,stringify!($enum_name)) })
            }
        }
    };
    ($enum_name:ident,$type:ty,{$($item:expr),*$(,)?})=>{
        $crate::model_enum_status_define!(self ,$enum_name,$type,{$(
            $item,
        )*});
    };
    ($enum_name:ident,$type:ty)=>{
        $crate::model_enum_status_define!(self ,$enum_name,$type,{});
    };
}

#[test]
fn test_model_enum_status() {
    #[derive(PartialEq, Eq, Clone, Copy)]
    enum UserModelStatus {
        Statu1 = 1,
        Statu2 = 2,
    }
    crate::model_enum_status_define!(UserModelStatus,u8,{
        UserModelStatus::Statu1,
        UserModelStatus::Statu2
    });
    assert!(UserModelStatus::Statu1.eq(1));
    assert!(!UserModelStatus::Statu1.eq(2));
    assert!(UserModelStatus::Statu2.eq(2));
    let status: UserModelStatus = 2.try_into().unwrap();
    assert!(status == UserModelStatus::Statu2);
    let status: Result<UserModelStatus, _> = 3.try_into();
    assert!(status.is_err());
}
