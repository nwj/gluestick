CREATE TABLE users (
  -- id is a UUIDv7
  id BLOB PRIMARY KEY CHECK(length(id) = 16),
  username TEXT NOT NULL UNIQUE CHECK(length(username) > 2 and username = lower(username)),
  email TEXT NOT NULL UNIQUE CHECK(email LIKE '%_@_%._%' AND email = lower(email)),
  -- password is a password hash, salted and hashed via Argon2id
  password TEXT NOT NULL CHECK(length(password) > 0)
) STRICT;

CREATE TABLE pastes (
  -- id is a UUIDv7
  id BLOB PRIMARY KEY CHECK(length(id) = 16),
  title TEXT NOT NULL CHECK(length(title) > 0),
  description TEXT NOT NULL CHECK(length(description) > 0),
  body TEXT NOT NULL CHECK(length(body) > 0)
) STRICT;
