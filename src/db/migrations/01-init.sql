CREATE TABLE pastes (
  -- id is a UUIDv7
  id BLOB PRIMARY KEY CHECK(length(id) = 16),
  text TEXT NOT NULL CHECK(length(text) > 0)
) STRICT;
