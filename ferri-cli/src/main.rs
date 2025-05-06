use main::types::{db, make, ObjectUri, ObjectUuid};
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

fn s(st: &'static str) -> String {
    st.to_string()
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

        let actor = db::Actor {
            id: ObjectUri(s("https://ferri.amy.mov/users/9b9d497b-2731-435f-a929-e609ca69dac9")),
            inbox: s("https://ferri.amy.mov/users/9b9d497b-2731-435f-a929-e609ca69dac9/inbox"),
            outbox: s("https://ferri.amy.mov/users/9b9d497b-2731-435f-a929-e609ca69dac9/outbox")
        };

        make::new_actor(actor.clone(), &mut *conn).await.unwrap();

        let user = db::User {
            id: ObjectUuid(s("9b9d497b-2731-435f-a929-e609ca69dac9")),
            actor,
            username: s("amy"),
            display_name: s("amy (display)"),
            acct: s("amy"),
            remote: false,
            url: s("https://ferri.amy.mov/@amy"),
            created_at: main::now(),
            icon_url: s("https://ferri.amy.mov/assets/pfp.png"),
            posts: db::UserPosts {
                last_post_at: None
            },
            key_id: s("https://ferri.amy.mov/users/9b9d497b-2731-435f-a929-e609ca69dac9#main-key")

        };

        make::new_user(user, &mut *conn).await.unwrap();
    } else {
        let _ = launch(config).launch().await;
    }
}
