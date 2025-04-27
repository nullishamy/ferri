CREATE TABLE IF NOT EXISTS follow
(
	-- Activity ID
	id TEXT PRIMARY KEY NOT NULL,
	follower_id TEXT NOT NULL,
	followed_id TEXT NOT NULL,
	
	FOREIGN KEY(follower_id) REFERENCES actor(id),
	FOREIGN KEY(followed_id) REFERENCES actor(id)
);
