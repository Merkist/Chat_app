use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::{FromRow, SqlitePool};
use sqlx::Error;

#[derive(Debug, FromRow, Serialize)]
pub struct MessageWithUsername {
    pub id: i64,
    pub chat_id: i64,
    pub user_id: i64,
    pub content: Option<String>,
    pub file_path: Option<String>,  // Шлях до файлу
    pub sent_at: Option<chrono::NaiveDateTime>,
    pub username: String,
}


#[derive(Debug, FromRow, Serialize)]
pub struct Message {
    pub id: Option<i64>,
    pub chat_id: Option<i64>,
    pub user_id: Option<i64>,
    pub content: Option<String>,  // Текст повідомлення
    pub file_path: Option<String>,  // Шлях до файлу
    pub sent_at: Option<NaiveDateTime>,  // Дата та час надсилання
}

impl Message {
    // Функція для створення нового повідомлення
    pub async fn create(
        pool: &SqlitePool,
        chat_id: i64,
        user_id: i64,
        content: Option<String>,
        file_path: Option<String>,
    ) -> Result<Self, Error> {
        let message = sqlx::query_as!(
            Message,
            r#"
            INSERT INTO messages (chat_id, user_id, content, file_path)
            VALUES (?, ?, ?, ?)
            RETURNING id, chat_id, user_id, content, file_path, sent_at
            "#,
            chat_id,
            user_id,
            content,
            file_path
        )
            .fetch_one(pool)
            .await?;

        Ok(message)
    }


    // Функція для отримання всіх повідомлень у чаті
    pub async fn get_messages_for_chat(pool: &SqlitePool, chat_id: i64) -> Result<Vec<MessageWithUsername>, Error> {
        let messages = sqlx::query_as!(
            MessageWithUsername,
            r#"
        SELECT messages.id, messages.chat_id, messages.user_id, messages.content,
        messages.file_path, messages.sent_at, users.username
        FROM messages
        JOIN users ON messages.user_id = users.id
        WHERE messages.chat_id = ?
        ORDER BY messages.sent_at ASC
        "#,
            chat_id
        )
            .fetch_all(pool)
            .await?;

        Ok(messages)
    }
}
