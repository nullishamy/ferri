use server::launch;
extern crate rocket;

use sqlx::sqlite::SqlitePool;
use std::env;

use clap::Parser;
use main::config;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    init: bool,

    #[arg(short, long)]
    config: PathBuf,
}

pub fn read_config(path: impl AsRef<Path>) -> config::Config {
    let content = fs::read_to_string(path).unwrap();
    toml::from_str(&content).unwrap()
}

#[rocket::main]
async fn main() {
    let cli = Cli::parse();
    let config = read_config(cli.config);

    if cli.init {
        // Seed DB
        let pool = SqlitePool::connect(&env::var("DATABASE_URL").unwrap())
            .await
            .unwrap();
        let mut conn = pool.acquire().await.unwrap();

        sqlx::query!(
            r#"
          INSERT INTO actor (id, inbox, outbox)
          VALUES (?1, ?2, ?3)
        "#,
            "https://ferri.amy.mov/users/9b9d497b-2731-435f-a929-e609ca69dac9",
            "https://ferri.amy.mov/users/9b9d497b-2731-435f-a929-e609ca69dac9/inbox",
            "https://ferri.amy.mov/users/9b9d497b-2731-435f-a929-e609ca69dac9/outbox"
        )
        .execute(&mut *conn)
        .await
            .unwrap();

        let ts = main::ap::new_ts();

        sqlx::query!(
            r#"
          INSERT INTO user (
            id, acct, url, remote, username,
            actor_id, display_name, created_at
          )
          VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        "#,
            "9b9d497b-2731-435f-a929-e609ca69dac9",
            "amy",
            "https://ferri.amy.mov/@amy",
            false,
            "amy",
            "https://ferri.amy.mov/users/9b9d497b-2731-435f-a929-e609ca69dac9",
            "amy",
            ts
        )
        .execute(&mut *conn)
        .await
        .unwrap();
    } else {
        let _ = launch(config).launch().await;
    }
}
