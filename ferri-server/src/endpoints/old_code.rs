fn handle_delete_activity(activity: ap::DeleteActivity) {
    warn!(?activity, "unimplemented delete activity");
}

async fn create_actor(
    user: &ap::Person,
    actor: &str,
    conn: impl sqlx::Executor<'_, Database = Sqlite>,
) {
    sqlx::query!(
        r#"
            INSERT INTO actor (id, inbox, outbox)
            VALUES ( ?1, ?2, ?3 )
            ON CONFLICT(id) DO NOTHING;
        "#,
        actor,
        user.inbox,
        user.outbox
    )
    .execute(conn)
    .await
    .unwrap();
}

async fn create_user(
    user: &ap::Person,
    actor: &str,
    conn: impl sqlx::Executor<'_, Database = Sqlite>,
) {
    // HACK: Allow us to formulate a `user@host` username by assuming the actor is on the same host as the user
    let url = Url::parse(actor).unwrap();
    let host = url.host_str().unwrap();
    info!(
        "creating user '{}'@'{}' ({:#?})",
        user.preferred_username, host, user
    );

    let (acct, remote) = if host != "ferri.amy.mov" {
        (format!("{}@{}", user.preferred_username, host), true)
    } else {
        (user.preferred_username.clone(), false)
    };

    let url = format!("https://ferri.amy.mov/{}", acct);
    let icon_url = user.icon.as_ref().map(|ic| ic.url.clone()).unwrap_or(
        "https://ferri.amy.mov/assets/pfp.png".to_string()
    );

    let uuid = Uuid::new_v4().to_string();
    // FIXME: Pull from user
    let ts = main::ap::new_ts();
    sqlx::query!(
        r#"
          INSERT INTO user (
            id, acct, url, remote, username,
            actor_id, display_name, created_at, icon_url
          )
          VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
          ON CONFLICT(actor_id) DO NOTHING;
        "#,
        uuid,
        acct,
        url,
        remote,
        user.preferred_username,
        actor,
        user.name,
        ts,
        icon_url
    )
    .execute(conn)
    .await
    .unwrap();
}

async fn create_follow(
    activity: &ap::FollowActivity,
    conn: impl sqlx::Executor<'_, Database = Sqlite>,
) {
    sqlx::query!(
        r#"
            INSERT INTO follow (id, follower_id, followed_id)
            VALUES ( ?1, ?2, ?3 )
            ON CONFLICT(id) DO NOTHING;
        "#,
        activity.obj.id.0,
        activity.actor,
        activity.object
    )
    .execute(conn)
    .await
    .unwrap();
}

struct RemoteInfo {
    acct: String,
    web_url: String,
    is_remote: bool,
}

fn get_remote_info(actor_url: &str, person: &ap::Person) -> RemoteInfo {
    let url = Url::parse(actor_url).unwrap();
    let host = url.host_str().unwrap();

    let (acct, remote) = if host != "ferri.amy.mov" {
        (format!("{}@{}", person.preferred_username, host), true)
    } else {
        (person.preferred_username.clone(), false)
    };

    let url = format!("https://ferri.amy.mov/{}", acct);

    RemoteInfo {
        acct: acct.to_string(),
        web_url: url,
        is_remote: remote,
    }
}

async fn resolve_actor<'a>(
    actor_url: &str,
    http: &HttpWrapper<'a>,
    conn: &mut SqliteConnection,
) -> Result<db::User, DbError> {
    let person = {
        let res = http.get_person(actor_url).await;
        if let Err(e) = res {
            error!("could not load user {}: {}", actor_url, e.to_string());
            return Err(DbError::FetchError(format!(
                "could not load user {}: {}",
                actor_url, e
            )));
        }

        res.unwrap()
    };

    let user_id = ObjectUuid::new();
    let remote_info = get_remote_info(actor_url, &person);

    let actor = db::Actor {
        id: ObjectUri(actor_url.to_string()),
        inbox: person.inbox.clone(),
        outbox: person.outbox.clone(),
    };

    info!("creating actor {}", actor_url);

    let actor = make::new_actor(actor.clone(), conn).await.unwrap_or(actor);

    info!("creating user {} ({:#?})", remote_info.acct, person);

    let user = db::User {
        id: user_id,
        actor,
        username: person.name,
        display_name: person.preferred_username,
        acct: remote_info.acct,
        remote: remote_info.is_remote,
        url: remote_info.web_url,
        created_at: main::ap::now(),
        icon_url: person.icon.map(|ic| ic.url).unwrap_or(
            "https://ferri.amy.mov/assets/pfp.png".to_string()
        ),

        posts: db::UserPosts { last_post_at: None },
    };

    Ok(make::new_user(user.clone(), conn).await.unwrap_or(user))
}

async fn handle_follow_activity<'a>(
    followed_account: &str,
    activity: ap::FollowActivity,
    http: HttpWrapper<'a>,
    mut db: Connection<Db>,
) {
    let actor = resolve_actor(&activity.actor, &http, &mut db)
        .await
        .unwrap();

    info!("{:?} follows {}", actor, followed_account);

    create_follow(&activity, &mut **db).await;

    let follower = main::ap::User::from_actor_id(&activity.actor, &mut **db).await;
    let followed = main::ap::User::from_id(followed_account, &mut **db)
        .await
        .unwrap();
    let outbox = main::ap::Outbox::for_user(followed.clone(), http.client());

    let activity = main::ap::Activity {
        id: format!("https://ferri.amy.mov/activities/{}", Uuid::new_v4()),
        ty: main::ap::ActivityType::Accept,
        object: activity.obj.id.0,
        ..Default::default()
    };

    let req = main::ap::OutgoingActivity {
        signed_by: format!(
            "https://ferri.amy.mov/users/{}#main-key",
            followed.username()
        ),
        req: activity,
        to: follower.actor().clone(),
    };

    req.save(&mut **db).await;
    outbox.post(req).await;
}

async fn handle_like_activity(activity: ap::LikeActivity, mut db: Connection<Db>) {
    warn!(?activity, "unimplemented like activity");

    let target_post = sqlx::query!("SELECT * FROM post WHERE uri = ?1", activity.object)
        .fetch_one(&mut **db)
        .await;

    if let Ok(post) = target_post {
        warn!(?post, "tried to like post");
    } else {
        warn!(post = ?activity.object, "could not find post");
    }
}

async fn handle_boost_activity<'a>(
    activity: ap::BoostActivity,
    http: HttpWrapper<'a>,
    mut db: Connection<Db>,
) {
    let key_id = "https://ferri.amy.mov/users/9b9d497b-2731-435f-a929-e609ca69dac9#main-key";
    dbg!(&activity);
    let post = http
        .client()
        .get(&activity.object)
        .activity()
        .sign(key_id)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    info!("{}", post);

    let post = serde_json::from_str::<ap::Post>(&post);
    if let Err(e) = post {
        error!(?e, "when decoding post");
        return;
    }

    let post = post.unwrap();

    info!("{:#?}", post);
    let attribution = post.attributed_to.unwrap();

    let post_user = http.get_person(&attribution).await;
    if let Err(e) = post_user {
        error!(
            "could not load post_user {}: {}",
            attribution,
            e.to_string()
        );
        return;
    }
    let post_user = post_user.unwrap();

    let user = http.get_person(&activity.actor).await;
    if let Err(e) = user {
        error!("could not load actor {}: {}", activity.actor, e.to_string());
        return;
    }
    let user = user.unwrap();

    debug!("creating actor {}", activity.actor);
    create_actor(&user, &activity.actor, &mut **db).await;

    debug!("creating user {}", activity.actor);
    create_user(&user, &activity.actor, &mut **db).await;

    debug!("creating actor {}", attribution);
    create_actor(&post_user, &attribution, &mut **db).await;

    debug!("creating user {}", attribution);
    create_user(&post_user, &attribution, &mut **db).await;

    let attributed_user = main::ap::User::from_actor_id(&attribution, &mut **db).await;
    let actor_user = main::ap::User::from_actor_id(&activity.actor, &mut **db).await;

    let base_id = main::ap::new_id();
    let now = main::ap::new_ts();

    let reblog_id = main::ap::new_id();

    let attr_id = attributed_user.id();
    // HACK: ON CONFLICT is to avoid duplicate remote posts coming in
    //       check this better in future
    sqlx::query!(
        "
       INSERT INTO post (id, uri, user_id, content, created_at)
       VALUES (?1, ?2, ?3, ?4, ?5)
       ON CONFLICT(uri) DO NOTHING
    ",
        reblog_id,
        post.obj.id.0,
        attr_id,
        post.content,
        post.ts
    )
    .execute(&mut **db)
    .await
    .unwrap();

    let uri = format!(
        "https://ferri.amy.mov/users/{}/posts/{}",
        actor_user.id(),
        base_id
    );
    let user_id = actor_user.id();

    info!("inserting post with id {} uri {}", base_id, uri);

    sqlx::query!(
        "
       INSERT INTO post (id, uri, user_id, content, created_at, boosted_post_id)
       VALUES (?1, ?2, ?3, ?4, ?5, ?6)
    ",
        base_id,
        uri,
        user_id,
        "",
        now,
        reblog_id
    )
    .execute(&mut **db)
    .await
    .unwrap();
}

async fn handle_create_activity<'a>(
    activity: ap::CreateActivity,
    http: HttpWrapper<'a>,
    mut db: Connection<Db>,
) {
    assert!(activity.object.ty == ap::ActivityType::Note);
    debug!("resolving user {}", activity.actor);

    let user = http.get_person(&activity.actor).await;
    if let Err(e) = user {
        error!("could not load user {}: {}", activity.actor, e.to_string());
        return;
    }

    let user = user.unwrap();

    debug!("creating actor {}", activity.actor);
    create_actor(&user, &activity.actor, &mut **db).await;

    debug!("creating user {}", activity.actor);
    create_user(&user, &activity.actor, &mut **db).await;

    let user = main::ap::User::from_actor_id(&activity.actor, &mut **db).await;
    debug!("user created {:?}", user);

    let user_id = user.id();
    let now = Local::now().to_rfc3339();
    let content = activity.object.content.clone();
    let post_id = Uuid::new_v4().to_string();
    let uri = activity.obj.id.0;

    info!(post_id, "creating post");

    sqlx::query!(
        r#"
        INSERT INTO post (id, uri, user_id, content, created_at)
        VALUES (?1, ?2, ?3, ?4, ?5)
    "#,
        post_id,
        uri,
        user_id,
        content,
        now
    )
    .execute(&mut **db)
    .await
    .unwrap();
}
