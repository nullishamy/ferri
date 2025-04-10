use server::launch;
extern crate rocket;

use sqlx::sqlite::SqlitePool;
use std::env;

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
	#[arg(short, long)]
    init: bool
}

#[rocket::main]
async fn main() {
	let cli = Cli::parse();
	if cli.init {
		// Seed DB
		let pool = SqlitePool::connect(&env::var("DATABASE_URL").unwrap()).await.unwrap();
		let mut conn = pool.acquire().await.unwrap();
		
		sqlx::query!(r#"
          INSERT INTO actor (id, inbox, outbox)
          VALUES (?1, ?2, ?3)
        "#, "https://ferri.amy.mov/users/amy", "https://ferri.amy.mov/users/amy/inbox", "https://ferri.amy.mov/users/amy/outbox")
          .execute(&mut *conn)
		  .await.unwrap();

		sqlx::query!(r#"
          INSERT INTO user (id, actor_id, display_name)
          VALUES (?1, ?2, ?3)
        "#, "amy", "https://ferri.amy.mov/users/amy", "amy")
           .execute(&mut *conn)
           .await.unwrap();
	} else {
		let _ = launch().launch().await;
	}
}

