use serde::Serialize;
use sqlx::{FromRow, SqlitePool};
use sqlx::Error;

#[derive(Debug, FromRow, Serialize)]
pub struct Chat {
    pub id: Option<i64>,
    pub name: String,
    pub owner_id: Option<i64>,
}


impl Chat {
    // Функція для створення нового чату
    pub async fn create(pool: &SqlitePool, name: &str, owner_id: i64) -> Result<Self, Error> {
        let chat = sqlx::query_as!(
            Chat,
            r#"
            INSERT INTO chats (name, owner_id)
            VALUES (?, ?)
            RETURNING id, name, owner_id
            "#,
            name,
            owner_id
        )
            .fetch_one(pool)
            .await?;

        Ok(chat)
    }

    // Функція для отримання всіх чатів користувача
    pub async fn get_chats_for_user(pool: &SqlitePool, user_id: i64) -> Result<Vec<Self>, Error> {
        let chats = sqlx::query_as!(
            Chat,
            r#"
            SELECT chats.id, chats.name, chats.owner_id
            FROM chats
            JOIN chat_users ON chats.id = chat_users.chat_id
            WHERE chat_users.user_id = ?
            "#,
            user_id
        )
            .fetch_all(pool)
            .await?;

        Ok(chats)
    }

    pub async fn is_user_in_chat(pool: &SqlitePool, user_id: i64, chat_id: i64) -> Result<bool, Error> {
        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*)
            FROM chat_users
            WHERE user_id = ? AND chat_id = ?
            "#,
            user_id,
            chat_id
        )
            .fetch_one(pool)
            .await?;

        Ok(count > 0)
    }

    pub async fn add_user_to_chat(pool: &SqlitePool, user_id: i64, chat_id: i64) -> Result<(), Error> {
        sqlx::query!(
            r#"
            INSERT INTO chat_users (user_id, chat_id)
            VALUES (?, ?)
            ON CONFLICT DO NOTHING
            "#,
            user_id,
            chat_id
        )
            .execute(pool)
            .await?;

        Ok(())
    }
}
