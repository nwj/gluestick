CREATE TABLE syntax_highlight_cache (
  -- paste_id is a UUIDv7
  paste_id BLOB PRIMARY KEY CHECK(length(paste_id) = 16),
  -- html is a paste body formatted to a syntax highlighted html string
  html TEXT NOT NULL,
  FOREIGN KEY(paste_id) REFERENCES pastes(id) ON DELETE CASCADE
) STRICT;
