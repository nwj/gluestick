CREATE TABLE users (
  -- id is a UUIDv7
  id BLOB PRIMARY KEY CHECK(length(id) = 16),
  username TEXT NOT NULL UNIQUE CHECK(length(username) > 2 and username = lower(username)),
  email TEXT NOT NULL UNIQUE CHECK(email LIKE '%_@_%._%' AND email = lower(email)),
  -- password is a password hash, salted and hashed via Argon2id
  password TEXT NOT NULL CHECK(length(password) > 0)
) STRICT;

CREATE TABLE sessions (
  -- session_token is hashed via SHA-256
  session_token BLOB PRIMARY KEY CHECK(length(session_token) = 32),
  -- user_id is a UUIDv7
  user_id BLOB NOT NULL CHECK(length(user_id) = 16)
) STRICT;

CREATE TABLE pastes (
  -- id is a UUIDv7
  id BLOB PRIMARY KEY CHECK(length(id) = 16),
  title TEXT NOT NULL CHECK(length(title) > 0),
  description TEXT NOT NULL CHECK(length(description) > 0),
  body TEXT NOT NULL CHECK(length(body) > 0),
  -- created_at and updated_at are both unix timestamps, with seconds precision
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  CHECK(created_at <= updated_at)
) STRICT;
