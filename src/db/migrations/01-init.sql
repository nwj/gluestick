CREATE TABLE users (
  -- id is a UUIDv7
  id BLOB PRIMARY KEY CHECK(length(id) = 16),
  username TEXT NOT NULL UNIQUE CHECK(username = lower(username) AND length(username) BETWEEN 1 AND 32),
  email TEXT NOT NULL UNIQUE CHECK(email LIKE '%_@_%' AND email = lower(email)),
  -- password is a password hash, salted and hashed via Argon2id
  password TEXT NOT NULL CHECK(length(password) > 0),
  -- created_at and updated_at are both unix timestamps, with millisecond precision
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  CHECK(created_at <= updated_at)
) STRICT;

CREATE TABLE sessions (
  -- session_token is a randomly generated u128, formatted as hex, hashed via SHA-256
  session_token BLOB PRIMARY KEY CHECK(length(session_token) = 32),
  -- user_id is a UUIDv7
  user_id BLOB NOT NULL CHECK(length(user_id) = 16),
  -- created_at and updated_at are both unix timestamps, with millisecond precision
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  CHECK(created_at <= updated_at),
  FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
) STRICT;

CREATE TABLE api_sessions (
  -- api_key is a randomly generated u128, formatted as hex, hashed via SHA-256
  api_key BLOB PRIMARY KEY CHECK(length(api_key) = 32),
  -- user_id is a UUIDv7
  user_id BLOB NOT NULL CHECK(length(user_id) = 16),
  -- created_at and updated_at are both unix timestamps, with millisecond precision
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  CHECK(created_at <= updated_at),
  FOREIGN KEY(user_id) REFERENCES users(id)
) STRICT;

CREATE TABLE invite_codes (
  code TEXT PRIMARY KEY
) STRICT;

CREATE TABLE pastes (
  -- id is a UUIDv7
  id BLOB PRIMARY KEY CHECK(length(id) = 16),
  -- user_id is a UUIDv7
  user_id BLOB NOT NULL CHECK(length(user_id) = 16),
  filename TEXT NOT NULL CHECK(length(filename) BETWEEN 1 AND 256),
  description TEXT NOT NULL CHECK(length(description) <= 256),
  body TEXT NOT NULL CHECK(length(body) > 0),
  visibility TEXT NOT NULL CHECK(visibility IN ('public', 'secret')),
  -- created_at and updated_at are both unix timestamps, with millisecond precision
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  CHECK(created_at <= updated_at),
  FOREIGN KEY(user_id) REFERENCES users(id)
) STRICT;

CREATE TABLE syntax_highlight_cache (
  -- paste_id is a UUIDv7
  paste_id BLOB PRIMARY KEY CHECK(length(paste_id) = 16),
  -- html is a paste body formatted to a syntax highlighted html string
  html TEXT NOT NULL,
  FOREIGN KEY(paste_id) REFERENCES pastes(id) ON DELETE CASCADE
) STRICT;
