use super::models::Comment;
use chrono::NaiveDateTime;
use rorm::Model;
use rorm::{and, query, Database};

const PAGE_SIZE: u64 = 20;

/// Query the data to render a thread's posts:
/// - Actual post message
/// - Time of post
/// - Author's username
/// - Whether it was an admin (to highlight name)
pub async fn get_posts(
    db: &Database,
    thread: i32,
    page: usize,
) -> Vec<(String, NaiveDateTime, String, bool)> {
    query!(
        db,
        (
            Comment::F.message,
            Comment::F.created,
            Comment::F.user.fields().username,
            Comment::F.user.fields().admin
        )
    )
    .condition(Comment::F.thread.equals(thread))
    // TODO order by newest posts
    .limit(PAGE_SIZE)
    .offset(page as u64 * PAGE_SIZE)
    .all()
    .await
    .unwrap()
}

/// Get all comments an admin posted in the "Support" thread
pub async fn get_support_messages(db: &Database) -> Vec<Comment> {
    query!(db, Comment)
        .condition(and!(
            Comment::F.thread.fields().name.equals("Support"),
            Comment::F.user.fields().admin.equals(true)
        ))
        .all()
        .await
        .unwrap()
}