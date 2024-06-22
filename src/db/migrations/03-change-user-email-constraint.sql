CREATE TABLE users_tmp (
  -- id is a UUIDv7
  id BLOB PRIMARY KEY CHECK(length(id) = 16),
  username TEXT NOT NULL UNIQUE CHECK(length(username) BETWEEN 3 AND 32),
  email TEXT NOT NULL UNIQUE CHECK(email LIKE '%_@_%' AND email = lower(email)),
  -- password is a password hash, salted and hashed via Argon2id
  password TEXT NOT NULL CHECK(length(password) > 0)
) STRICT;

INSERT INTO users_tmp SELECT * FROM users;
DROP TABLE users;

CREATE TABLE users (
  -- id is a UUIDv7
  id BLOB PRIMARY KEY CHECK(length(id) = 16),
  username TEXT NOT NULL UNIQUE CHECK(length(username) BETWEEN 3 AND 32),
  email TEXT NOT NULL UNIQUE CHECK(email LIKE '%_@_%' AND email = lower(email)),
  -- password is a password hash, salted and hashed via Argon2id
  password TEXT NOT NULL CHECK(length(password) > 0)
) STRICT;

INSERT INTO users SELECT * FROM users_tmp;
DROP TABLE users_tmp;
