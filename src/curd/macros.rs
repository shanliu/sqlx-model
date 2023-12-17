macro_rules! scalar_pk_where {
    ($db_type:ty,$table_pk:expr,$len:expr) => {
        match $table_pk.0.get(0) {
            Some(ref pkfield) => {
                let bst = DbType::type_new::<$db_type>().mark($len);

                format!("{}={}", pkfield.name, bst)
            }
            None => "0".to_string(),
        }
    };
}
