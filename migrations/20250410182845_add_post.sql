CREATE TABLE IF NOT EXISTS post
(
	id TEXT PRIMARY KEY NOT NULL,
	uri TEXT NOT NULL UNIQUE,
	user_id TEXT NOT NULL,
	content TEXT NOT NULL,
	created_at TEXT NOT NULL,
	boosted_post_id TEXT,

	FOREIGN KEY(user_id) REFERENCES user(id),
	FOREIGN KEY(boosted_post_id) REFERENCES post(id)
);
