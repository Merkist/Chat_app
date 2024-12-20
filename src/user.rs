use sqlx::SqlitePool;

pub struct User {
    pub id: Option<i64>,
    pub username: String,
    pub password: String,
}

impl User {
    pub async fn create(pool: &SqlitePool, username: &str, password: &str) -> Result<Self, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            "INSERT INTO users (username, password) VALUES (?, ?) RETURNING id, username, password",
            username,
            password
        )
            .fetch_one(pool)
            .await?;

        Ok(user)
    }

    pub async fn get_by_username(pool: &SqlitePool, username: &str) -> Result<Option<Self>, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            "SELECT id, username, password FROM users WHERE username = ?",
            username
        )
            .fetch_optional(pool)
            .await?;

        Ok(user)
    }

    pub async fn get_by_id(pool: &SqlitePool, id: i64) -> Result<Option<Self>, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            "SELECT id, username, password FROM users WHERE id = ?",
            id
        )
            .fetch_optional(pool)
            .await?;

        Ok(user)
    }
}
