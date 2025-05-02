use crate::{
    groups::UserGroups,
    schema::{files, users},
};
use bcrypt::{DEFAULT_COST, hash};
use diesel::{Insertable, Queryable, prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Queryable, Serialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub group_name: String,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub username: String,
    pub password: String,
    pub group_name: String,
}

#[derive(Queryable, Serialize)]
pub struct FileRecord {
    pub id: i32,
    pub filename: String,
    pub filepath: String,
    pub uploader_id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = files)]
pub struct NewFile {
    pub filename: String,
    pub filepath: String,
    pub uploader_id: i32,
}

pub fn init_admin_user(conn: &mut SqliteConnection) {
    use crate::schema::users::dsl::*;
    if diesel::select(diesel::dsl::exists(users.filter(username.eq("admin"))))
        .get_result(conn)
        .unwrap_or(false)
    {
        return;
    }

    let hashed = hash("admin123", DEFAULT_COST).unwrap();
    let admin = NewUser {
        username: "admin".into(),
        password: hashed,
        group_name: UserGroups::Admin.to_string(),
    };
    diesel::insert_into(users)
        .values(admin)
        .execute(conn)
        .unwrap();
}
