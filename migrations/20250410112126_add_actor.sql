CREATE TABLE IF NOT EXISTS actor
(
	-- URI
	id TEXT PRIMARY KEY NOT NULL,
	inbox TEXT NOT NULL,
	outbox TEXT NOT NULL
);
