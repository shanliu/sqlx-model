mod insert;
mod update;

mod select;

mod delete;


mod transaction;


#[test]
fn test_model_enum_status(){
    #[derive(sqlx_model::SqlxModelStatus)]
    #[sqlx_model_status(type="u8")]
    enum UserModelStatus {
        Statu1=1,
        Statu2=2,
    }
    assert_eq!(UserModelStatus::Statu1.eq(1),true);
    assert_eq!(UserModelStatus::Statu1.eq(2),false);
    assert_eq!(UserModelStatus::Statu2.eq(2),true);
}