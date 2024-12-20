CREATE TABLE users (
                       id INTEGER PRIMARY KEY AUTOINCREMENT,
                       username TEXT NOT NULL UNIQUE,
                       password TEXT NOT NULL
);

CREATE TABLE chats (
                       id INTEGER PRIMARY KEY AUTOINCREMENT,
                       name TEXT NOT NULL,
                       owner_id INTEGER NOT NULL REFERENCES users(id)
);

CREATE TABLE messages (
                          id INTEGER PRIMARY KEY AUTOINCREMENT,
                          chat_id INTEGER NOT NULL REFERENCES chats(id),
                          user_id INTEGER NOT NULL REFERENCES users(id),
                          content TEXT NOT NULL,
                          file_path TEXT,
                          sent_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE chat_users (
                            user_id INTEGER NOT NULL REFERENCES users(id),
                            chat_id INTEGER NOT NULL REFERENCES chats(id),
                            PRIMARY KEY (user_id, chat_id)
);

