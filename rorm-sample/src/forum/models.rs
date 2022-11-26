use chrono::{NaiveDate, NaiveDateTime};
use rorm::{DbEnum, ForeignModel, Model};

#[derive(Model)]
#[rorm(rename = "account")]
pub struct User {
    #[rorm(id)]
    pub id: i32,

    #[rorm(unique, index)]
    #[rorm(max_length = 255)]
    pub username: String,

    #[rorm(default = false)]
    #[rorm(rename = "superuser")]
    pub admin: bool,

    #[rorm(auto_create_time)]
    pub member_since: NaiveDateTime,

    pub gender: Option<Gender>,

    pub birthday: Option<NaiveDate>,
}

#[derive(DbEnum, Copy, Clone)]
pub enum Gender {
    Male,
    Female,
    Other,
}

#[derive(Model)]
pub struct Comment {
    #[rorm(id)]
    pub id: i32,

    #[rorm(max_length = 255)]
    pub message: String,

    #[rorm(auto_create_time)]
    pub created: NaiveDateTime,

    #[rorm(on_delete = "Cascade")]
    pub user: ForeignModel<User>,

    #[rorm(on_delete = "Cascade")]
    pub thread: ForeignModel<Thread>,
}

#[derive(Model)]
pub struct Thread {
    #[rorm(id)]
    pub id: i32,

    #[rorm(max_length = 255)]
    pub name: String,

    pub creator: ForeignModel<User>,

    #[rorm(ignore)]
    pub fred: Option<std::thread::Thread>,

    #[rorm(field = "Comment::F.thread")]
    pub comments: rorm::internal::field::back_ref::BackRef<Comment>,
}
