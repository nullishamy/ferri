CREATE TABLE IF NOT EXISTS post
(
	-- Uri
	id TEXT PRIMARY KEY NOT NULL,
	user_id TEXT NOT NULL,
	content TEXT NOT NULL,
	created_at TEXT NOT NULL,

	FOREIGN KEY(user_id) REFERENCES user(id)
);
