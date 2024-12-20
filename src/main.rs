use actix_web::{web, App, HttpServer};
use actix_files::Files;
use actix_web::middleware::Logger;
use tera::Tera;
use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use actix_web::cookie::Key;
use crate::pages::{logout_user, register_user, show_register_form, show_create_chat_page,
                   login_user, show_login_form, show_home_page, create_chat, join_chat,
                   show_chat_page, search_chats};

use crate::websocket::websocket_handler;
use crate::websocket::ChatRooms;

mod pages;
mod websocket;
mod db;
mod user;
mod chat;
mod message;



#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = db::establish_connection().await.unwrap();
    let tera = Tera::new("static/**/*").unwrap();
    let secret_key = Key::generate(); // Генерація секретного ключа для сесій
    let chat_rooms = web::Data::new(ChatRooms::default()); // Створюємо сховище для чатів


    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(tera.clone()))
            .app_data(web::Data::new(pool.clone()))
            .app_data(chat_rooms.clone()) // Додаємо сховище до даних застосунку
            .wrap(SessionMiddleware::new(CookieSessionStore::default(), secret_key.clone()))
            .wrap(Logger::default())
            .route("/", web::get().to(show_home_page))  // Головна сторінка
            .route("/register", web::get().to(show_register_form))
            .route("/register", web::post().to(register_user))
            .route("/login", web::get().to(show_login_form))
            .route("/login", web::post().to(login_user))
            .route("/logout", web::get().to(logout_user))
            .route("/create_chat", web::get().to(show_create_chat_page))
            .route("/create_chat", web::post().to(create_chat))
            .route("/chat/{chat_id}", web::get().to(show_chat_page))
            .route("/chat/{chat_id}/join", web::post().to(join_chat))
            .route("/search_chats", web::get().to(search_chats))
            .route("/ws/chat/{chat_id}", web::get().to(websocket_handler)) // Вебсокети
            .service(Files::new("/static", "./static").show_files_listing())
    })
        .bind("127.0.0.1:3030")?
        .run()
        .await
}
