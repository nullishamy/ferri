CREATE TABLE IF NOT EXISTS attachment
(
	-- UUID
	id TEXT PRIMARY KEY NOT NULL,
    post_id TEXT NOT NULL,
    url TEXT NOT NULL,
    media_type TEXT NOT NULL,
    marked_sensitive BOOL NOT NULL,
    alt TEXT,
    
    FOREIGN KEY(post_id) REFERENCES post(id)
);

