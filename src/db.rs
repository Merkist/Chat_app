// Це буде залежати від того, яку БД ви використовуєте (наприклад, SQLite, PostgreSQL).
// Ось приклад для SQLite:

use sqlx::{SqlitePool, Error};
use dotenv::dotenv;
use std::env;


pub async fn establish_connection() -> Result<SqlitePool, Error> {
    // Завантажуємо змінні середовища з файлу .env
    dotenv().ok();

    // Отримуємо значення DATABASE_URL
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env file");

    // Підключаємося до бази даних
    let pool = SqlitePool::connect(&database_url).await?;
    println!("Connected to SQLite database!");

    // Створення нового користувача
    //let user = User::create(&pool, "test_user", "secure_password").await?;
    //println!("User created: {:?}", user);

    Ok(pool)
}
