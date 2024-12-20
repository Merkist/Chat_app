use actix_web_actors::ws;
use actix_web::{web, HttpRequest, HttpResponse};
use actix_session::Session;
use crate::message::Message;
use sqlx::SqlitePool;
use actix::prelude::*;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

// Місце для зберігання чатів
pub type ChatRooms = Arc<Mutex<HashMap<i64, Addr<ChatRoom>>>>;

pub struct ChatSession {
    pub chat_id: i64,
    pub user_id: i64,
    pub addr: Addr<ChatRoom>,
    pub pool: Arc<SqlitePool>,
}

impl ChatSession {
    pub fn new(chat_id: i64, user_id: i64, addr: Addr<ChatRoom>, pool: Arc<SqlitePool>) -> Self {
        ChatSession { chat_id, user_id, addr, pool }
    }
}

impl Handler<NewMessage> for ChatSession {
    type Result = ();

    fn handle(&mut self, msg: NewMessage, ctx: &mut Self::Context) {
        println!("Sending message to client: {}", msg.content);
        // Відправка отриманого повідомлення у вебсокет
        ctx.text(format!("User {}: {}", msg.user_id, msg.content));
    }
}


impl Actor for ChatSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.addr.do_send(UserJoined {
            user_id: self.user_id,
            addr: ctx.address(),
        });
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        self.addr.do_send(UserLeft {
            user_id: self.user_id,
        });
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ChatSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        if let Ok(ws::Message::Text(text)) = msg {
            println!("Received message: {}", text);
            let text_clone = text.clone(); // Клонуємо значення
            let pool = self.pool.clone();
            let user_id = self.user_id;
            let chat_id = self.chat_id;

            // Зберігаємо повідомлення в базу даних
            actix_rt::spawn(async move {
                let _ = Message::create(&pool, chat_id, user_id, Some(text_clone.to_string()), None).await;
            });

            // Надсилаємо повідомлення в ChatRoom
            self.addr.do_send(NewMessage {
                content: text.to_string(),
                user_id,
            });
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct NewMessage {
    pub content: String,
    pub user_id: i64,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct UserJoined {
    pub user_id: i64,
    pub addr: Addr<ChatSession>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct UserLeft {
    pub user_id: i64,
}

pub struct ChatRoom {
    pub users: Vec<Addr<ChatSession>>,
}

impl ChatRoom {
    pub fn new() -> Self {
        ChatRoom { users: Vec::new() }
    }
}

impl Actor for ChatRoom {
    type Context = Context<Self>;
}

impl Handler<NewMessage> for ChatRoom {
    type Result = ();

    fn handle(&mut self, msg: NewMessage, _: &mut Self::Context) {
        println!("Broadcasting message to {} users: {}", self.users.len(), msg.content);
        for user in &self.users {
            user.do_send(NewMessage {
                content: msg.content.clone(),
                user_id: msg.user_id,
            });
        }
    }
}

impl Handler<UserJoined> for ChatRoom {
    type Result = ();

    fn handle(&mut self, msg: UserJoined, _: &mut Self::Context) {
        println!("User joined: {}", msg.user_id);
        self.users.push(msg.addr);
        println!("Users count: {}", self.users.len());
    }
}

impl Handler<UserLeft> for ChatRoom {
    type Result = ();

    fn handle(&mut self, msg: UserLeft, _: &mut Self::Context) {
        self.users.retain(|user| user.connected());
    }
}

pub async fn websocket_handler(
    req: HttpRequest,
    stream: web::Payload,
    pool: web::Data<SqlitePool>,
    path: web::Path<i64>,
    session: Session,
    chat_rooms: web::Data<ChatRooms>, // Приймаємо централізоване сховище чатів
) -> Result<HttpResponse, actix_web::Error> {
    println!("WebSocket connection initiated");

    let chat_id = path.into_inner();

    // Перевірка наявності user_id в сесії
    if let Some(user_id) = session.get::<String>("user_id").unwrap() {
        println!("authorized WebSocket connection");

        let user_id: i64 = user_id.parse().unwrap();

        // Перевіряємо, чи є вже чат з таким chat_id
        let chat_addr = {
            let chat_rooms = chat_rooms.lock().unwrap();
            chat_rooms.get(&chat_id).cloned()
        };

        let addr = if let Some(addr) = chat_addr {
            // Якщо чат існує, використовуємо вже існуючий
            addr
        } else {
            // Якщо чат не існує, створюємо новий
            let new_chat = ChatRoom::new().start();
            let mut chat_rooms = chat_rooms.lock().unwrap();
            chat_rooms.insert(chat_id, new_chat.clone());
            new_chat
        };

        // Створюємо ChatSession
        let pool_arc = Arc::new(pool.get_ref().clone());
        let session = ChatSession::new(chat_id, user_id, addr.clone(), pool_arc);

        ws::start(session, &req, stream)
    } else {
        println!("Unauthorized WebSocket connection");
        Ok(HttpResponse::Unauthorized().body("User is not logged in"))
    }
}
