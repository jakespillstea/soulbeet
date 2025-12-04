CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY NOT NULL,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS folders (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    path TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_folders_user_id ON folders(user_id);

-- Default user: admin / admin
-- Password hash for 'admin' using Argon2
INSERT OR IGNORE INTO users (id, username, password_hash)
VALUES (
    '00000000-0000-0000-0000-000000000000',
    'admin',
    '$argon2id$v=19$m=19456,t=2,p=1$llsT7N68SnCXwaqcvFP08g$W+5l4cDaOfsY9nK2jFs7JGwkxtVtmN+VLIWC7ZOM9/E'
);