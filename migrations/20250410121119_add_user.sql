CREATE TABLE IF NOT EXISTS user
(
	-- UUID
	id TEXT PRIMARY KEY NOT NULL,
	username TEXT NOT NULL,
	actor_id TEXT NOT NULL,
	display_name TEXT NOT NULL,

	FOREIGN KEY(actor_id) REFERENCES actor(id)
);
