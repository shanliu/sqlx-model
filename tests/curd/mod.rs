mod insert;
mod update;

mod select;

mod delete;

mod transaction;

#[test]
fn test_model_enum_status() {
    #[derive(sqlx_model::SqlxModelStatus, PartialEq, Eq)]
    #[sqlx_model_status(type = "u8")]
    enum UserModelStatus {
        Statu1 = 1,
        Statu2 = 2,
    }
    assert!(UserModelStatus::Statu1.eq(1));
    assert!(!UserModelStatus::Statu1.eq(2));
    assert!(UserModelStatus::Statu2.eq(2));
    let status: UserModelStatus = 2.try_into().unwrap();
    assert!(status == UserModelStatus::Statu2);
    let status: Result<UserModelStatus, _> = 3.try_into();
    assert!(status.is_err());
}
