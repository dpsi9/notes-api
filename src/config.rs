use std::env;

pub struct Config {
    pub database_url: String,
    pub server_addr: String,
}

pub fn get_config() -> Config {
    Config {
        database_url: env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://notes_user:notes_pass@127.0.0.1:5432/notes_db".into()),
        server_addr: env::var("127.0.0.1:8080").unwrap_or_else(|_| "127.0.0.1:8080".into()),
    }
}
