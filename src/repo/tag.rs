use sqlx::{Acquire, PgConnection};
use uuid::Uuid;

use crate::model::{Tag, UpdateTag};

#[derive(Clone)]
pub struct TagRepo {}

impl TagRepo {
    pub async fn list_by_user_id(user_id: Uuid, db: &mut PgConnection) -> sqlx::Result<Vec<Tag>> {
        sqlx::query_as!(
            Tag,
            r#"SELECT tag.* FROM tag RIGHT JOIN auth.user_to_tag ON tag.id = auth.user_to_tag.tag_id WHERE auth.user_to_tag.user_id = $1 and deleted_at is null"#,
            user_id
        )
        .fetch_all(db)
        .await
    }

    pub async fn list_all(db: &mut PgConnection) -> sqlx::Result<Vec<Tag>> {
        sqlx::query_as!(Tag, r#"SELECT * from tag WHERE deleted_at is null"#)
            .fetch_all(db)
            .await
    }

    pub async fn create_missing(
        tags: Vec<UpdateTag>,
        current_user_id: Uuid,
        db: &mut PgConnection,
    ) -> sqlx::Result<Vec<Tag>> {
        let mut tx = db.begin().await.unwrap();
        let mut all_tags = vec![];
        for tag in tags {
            match tag {
                UpdateTag::Existing { id } => {
                    let existing = sqlx::query_as!(Tag, r#"SELECT * FROM tag WHERE id = $1"#, id)
                        .fetch_one(&mut *tx)
                        .await?;
                    all_tags.push(existing);
                }
                UpdateTag::New { label } => {
                    let created = sqlx::query_as!(
                        Tag,
                        r#"INSERT INTO tag (title, created_by, updated_by) VALUES ($1, $2, $3) RETURNING *"#,
                        label,
                        current_user_id,
                        current_user_id
                    )
                    .fetch_one(&mut *tx)
                    .await?;
                    all_tags.push(created);
                }
            }
        }

        tx.commit().await.unwrap();
        Ok(all_tags)
    }
}
