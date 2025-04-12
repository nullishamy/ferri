use server::launch;
extern crate rocket;

use sqlx::sqlite::SqlitePool;
use std::env;

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    init: bool,
}

#[rocket::main]
async fn main() {
    let cli = Cli::parse();
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
            "https://ferri.amy.mov/users/c81db53f-d836-4283-a835-26606c9d14ff",
            "https://ferri.amy.mov/users/c81db53f-d836-4283-a835-26606c9d14ff/inbox",
            "https://ferri.amy.mov/users/c81db53f-d836-4283-a835-26606c9d14ff/outbox"
        )
        .execute(&mut *conn)
        .await
        .unwrap();

        sqlx::query!(
            r#"
          INSERT INTO user (id, username, actor_id, display_name)
          VALUES (?1, ?2, ?3, ?4)
        "#,
            "9b9d497b-2731-435f-a929-e609ca69dac9",
            "amy",
            "https://ferri.amy.mov/users/c81db53f-d836-4283-a835-26606c9d14ff",
            "amy"
        )
        .execute(&mut *conn)
        .await
        .unwrap();
    } else {
        let _ = launch().launch().await;
    }
}
