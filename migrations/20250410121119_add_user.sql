CREATE TABLE IF NOT EXISTS user
(
	-- Username
    id TEXT PRIMARY KEY NOT NULL,
	actor_id TEXT NOT NULL,
	display_name TEXT NOT NULL,

	FOREIGN KEY(actor_id) REFERENCES actor(id)
);
