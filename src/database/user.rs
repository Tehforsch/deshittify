pub struct User {
    pub id: i32,
    pub data: UserData,
}

#[derive(Debug)]
pub struct UserData {
    pub user_id: i64,
    pub name: String,
}
