use actix_web::{web, HttpResponse, Responder};
use sqlx::SqlitePool;
use crate::user::User;
use crate::chat::Chat;
use crate::message::Message;
use tera::{Tera, Context};
use actix_session::Session;


pub async fn show_home_page(pool: web::Data<SqlitePool>, tera: web::Data<Tera>, session: Session) -> impl Responder {
    // Перевіряємо, чи є користувач в сесії
    if let Some(user_id) = session.get::<String>("user_id").unwrap() {
        // Якщо користувач є в сесії, отримуємо його ім'я

        let pool = pool.get_ref(); // Отримуємо референс на SqlitePool
        let user = User::get_by_id(pool, user_id.parse::<i64>().unwrap()).await;

        //let user = User::get_by_username(&req.app_data::<SqlitePool>().unwrap(), &user_id).await;

        match user {
            Ok(Some(user)) => {

                let chats = Chat::get_chats_for_user(pool, user_id.parse::<i64>().unwrap()).await.unwrap_or_default();;

                let mut context = Context::new();
                context.insert("username", &user.username);
                context.insert("chats", &chats);
                println!("Context before rendering: {:?}", context);

                let rendered = tera.render("index.html", &context).unwrap();

                HttpResponse::Ok().content_type("text/html").body(rendered)
            }
            _ => {
                // Якщо користувача не знайдено, відправляємо відповідь про помилку
                HttpResponse::Unauthorized().body("User not found.")
            }
        }
    } else {
        // Якщо користувача немає в сесії, показуємо сторінку без імені
        let rendered = tera.render("index.html", &Context::new()).unwrap();
        HttpResponse::Ok().content_type("text/html").body(rendered)
    }
}

#[derive(serde::Deserialize)]
pub struct RegisterForm {
    pub username: String,
    pub password: String,
}

pub async fn register_user(
    pool: web::Data<SqlitePool>,
    form: web::Form<RegisterForm>,
) -> impl Responder {
    let pool = pool.get_ref();

    // Перевіряємо, чи вже є користувач з таким ім'ям
    match User::get_by_username(pool, &form.username).await {
        Ok(Some(_)) => {
            // Якщо такий користувач є, повертаємо помилку
            HttpResponse::BadRequest().body("Username is already taken.")
        }
        Ok(None) => {
            // Якщо користувача немає, створюємо нового
            match User::create(pool, &form.username, &form.password).await {
                Ok(_) => HttpResponse::Ok().body("Registration successful!"),
                Err(_) => HttpResponse::InternalServerError().body("An error occurred during registration."),
            }
        }
        Err(_) => HttpResponse::InternalServerError().body("Database error."),
    }
}

pub async fn show_register_form() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(include_str!("../static/register.html"))
}


#[derive(serde::Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

pub async fn login_user(
    pool: web::Data<SqlitePool>,
    form: web::Form<LoginForm>,
    session: Session,
    tera: web::Data<Tera>,
) -> impl Responder {
    let pool = pool.get_ref();

    // Перевіряємо, чи є користувач з таким ім'ям
    match User::get_by_username(pool, &form.username).await {
        Ok(Some(user)) => match user.password == form.password {
            true => {
                // Якщо паролі співпадають, зберігаємо user_id в сесії
                session.insert("user_id", user.id.unwrap().to_string()).unwrap();

                HttpResponse::Found()
                    .header("Location", "/")  // Перенаправлення на головну сторінку
                    .finish()
            }
            false => {
                let mut context = Context::new();
                context.insert("message", "Invalid password.");
                let rendered = tera.render("login.html", &context).unwrap();
                HttpResponse::Unauthorized().content_type("text/html").body(rendered)
            }
        },
        Ok(None) => {
            let mut context = Context::new();
            context.insert("message", "User not found.");
            let rendered = tera.render("login.html", &context).unwrap();
            HttpResponse::NotFound().content_type("text/html").body(rendered)
        }
        Err(_) => HttpResponse::InternalServerError().body("Database error."),
    }
}

pub async fn show_login_form(tera: web::Data<Tera>) -> impl Responder {
    let mut context = Context::new();
    context.insert("message", "Please log in.");
    let rendered = tera.render("login.html", &context).unwrap();

    HttpResponse::Ok().content_type("text/html").body(rendered)
}

pub async fn logout_user(session: Session, tera: web::Data<Tera>) -> impl Responder {
    // Видаляємо user_id з сесії
    session.remove("user_id");

    HttpResponse::Found()
        .header("Location", "/")  // Перенаправлення на головну сторінку
        .finish()
}


pub async fn show_chat_page(
    tera: web::Data<Tera>,
    session: Session,
    pool: web::Data<SqlitePool>,
    chat_id: web::Path<i64>,
) -> impl Responder {
    if let Some(user_id) = session.get::<String>("user_id").unwrap() {
        let user_id: i64 = user_id.parse().unwrap();
        let pool = pool.get_ref();

        // Перевіряємо, чи користувач приєднаний до чату
        let is_member = Chat::is_user_in_chat(pool, user_id, *chat_id).await.unwrap_or(false);

        // Отримуємо повідомлення з чату
        let messages = Message::get_messages_for_chat(pool, *chat_id).await.unwrap_or_default();


        let mut context = Context::new();
        context.insert("chat_id", &chat_id.to_string());
        context.insert("is_member", &is_member);
        context.insert("messages", &messages);

        let rendered = tera.render("chat.html", &context).unwrap();
        HttpResponse::Ok().content_type("text/html").body(rendered)
    } else {
        HttpResponse::Unauthorized().body("You must log in to access this page.")
    }
}

pub async fn join_chat(
    pool: web::Data<SqlitePool>,
    session: Session,
    chat_id: web::Path<i64>,
) -> impl Responder {
    if let Some(user_id) = session.get::<String>("user_id").unwrap() {
        let user_id: i64 = user_id.parse().unwrap();
        let pool = pool.get_ref();

        match Chat::add_user_to_chat(pool, user_id, *chat_id).await {
            Ok(_) => {
                HttpResponse::Found()
                    .append_header(("Location", format!("/chat/{}", *chat_id)))
                    .finish()
            }
            Err(_) => HttpResponse::InternalServerError().body("Failed to join the chat."),
        }
    } else {
        HttpResponse::Unauthorized().body("You must log in to join this chat.")
    }
}



pub async fn show_create_chat_page(
    tera: web::Data<Tera>,
    session: Session,
) -> impl Responder {
    // Перевіряємо, чи є користувач в сесії
    if let Some(_) = session.get::<String>("user_id").unwrap() {
        let rendered = tera.render("create_chat.html", &Context::new()).unwrap();
        HttpResponse::Ok().content_type("text/html").body(rendered)
    } else {
        HttpResponse::Unauthorized().body("You must log in to create a chat.")
    }
}


#[derive(serde::Deserialize)]
pub struct CreateChatForm {
    pub name: String,
}

pub async fn create_chat(
    pool: web::Data<SqlitePool>,
    session: Session,
    form: web::Form<CreateChatForm>,
) -> impl Responder {
    // Перевіряємо, чи є користувач в сесії
    if let Some(user_id) = session.get::<String>("user_id").unwrap() {
        let pool = pool.get_ref();
        let user_id: i64 = user_id.parse().unwrap();

        // Створюємо новий чат
        match Chat::create(pool, &form.name, user_id).await {
            Ok(chat) => {
                HttpResponse::Found()
                    .header("Location", "/")  // Перенаправлення на головну сторінку
                    .finish()
            }
            Err(err) => {
                eprintln!("Error creating chat: {:?}", err);
                HttpResponse::InternalServerError().body("Failed to create chat.")
            }
        }
    } else {
        HttpResponse::Unauthorized().body("You must log in to create a chat.")
    }
}

#[derive(serde::Deserialize)]
pub struct SearchQuery {
    query: String,
}

pub async fn search_chats(pool: web::Data<SqlitePool>, query: web::Query<SearchQuery>) -> impl Responder {
    let query = &query.query;

    // Виконуємо SQL запит на пошук чатів за назвою
    let chats = sqlx::query_as::<_, Chat>(
        "SELECT id, name, owner_id FROM chats WHERE name LIKE ? ORDER BY name"
    )
        .bind(format!("%{}%", query)) // Використовуємо LIKE для пошуку за частиною назви
        .fetch_all(pool.get_ref())
        .await;

    match chats {
        Ok(chats) => {
            // Повертаємо знайдені чати у форматі JSON
            HttpResponse::Ok().json(serde_json::json!({ "chats": chats }))
        }
        Err(e) => {
            eprintln!("Error searching chats: {}", e);
            HttpResponse::InternalServerError().body("Error searching chats")
        }
    }
}