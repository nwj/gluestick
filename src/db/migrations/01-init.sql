CREATE TABLE pastes (
  -- id is a UUIDv7
  id BLOB PRIMARY KEY CHECK(length(id) = 16),
  description TEXT NOT NULL CHECK(length(description) > 0),
  body TEXT NOT NULL CHECK(length(body) > 0)
) STRICT;
