CREATE TABLE IF NOT EXISTS auth
(
	token TEXT PRIMARY KEY NOT NULL,
	user_id TEXT NOT NULL,
    
    FOREIGN KEY(user_id) REFERENCES user(id)
);

CREATE TABLE IF NOT EXISTS app
(
    client_id TEXT PRIMARY KEY NOT NULL,
    client_secret TEXT NOT NULL,
    scopes TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS oauth
(
    id_token TEXT PRIMARY KEY NOT NULL,
    client_id TEXT NOT NULL,
    expires_in INTEGER NOT NULL,
    scope TEXT NOT NULL,
    access_token TEXT NOT NULL,

    FOREIGN KEY(access_token) REFERENCES auth(token),
    FOREIGN KEY(client_id) REFERENCES app(client_id)
);
