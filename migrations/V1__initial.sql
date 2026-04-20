PRAGMA foreign_keys = ON;

CREATE TABLE books (
  id             INTEGER PRIMARY KEY,
  stable_id      TEXT,
  file_hash      TEXT,
  title_norm     TEXT NOT NULL,
  author_norm    TEXT,
  path           TEXT NOT NULL,
  title          TEXT NOT NULL,
  author         TEXT,
  language       TEXT,
  publisher      TEXT,
  published_at   TEXT,
  word_count     INTEGER,
  page_count     INTEGER,
  added_at       TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  finished_at    TEXT,
  rating         INTEGER,
  parse_error    TEXT,
  deleted_at     TEXT
);
CREATE UNIQUE INDEX idx_books_stable_id ON books(stable_id) WHERE stable_id IS NOT NULL;
CREATE INDEX idx_books_file_hash  ON books(file_hash)  WHERE file_hash IS NOT NULL;
CREATE INDEX idx_books_norm_match ON books(title_norm, author_norm);

CREATE TABLE tags (
  id   INTEGER PRIMARY KEY,
  name TEXT NOT NULL UNIQUE
);
CREATE TABLE book_tags (
  book_id INTEGER NOT NULL REFERENCES books(id) ON DELETE CASCADE,
  tag_id  INTEGER NOT NULL REFERENCES tags(id)  ON DELETE CASCADE,
  PRIMARY KEY (book_id, tag_id)
);

CREATE TABLE progress (
  book_id       INTEGER PRIMARY KEY REFERENCES books(id) ON DELETE CASCADE,
  spine_idx     INTEGER NOT NULL,
  char_offset   INTEGER NOT NULL,
  anchor_hash   TEXT NOT NULL,
  percent       REAL NOT NULL,
  last_read_at  TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  time_read_s   INTEGER NOT NULL DEFAULT 0,
  words_read    INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE bookmarks (
  id           INTEGER PRIMARY KEY,
  book_id      INTEGER NOT NULL REFERENCES books(id) ON DELETE RESTRICT,
  mark         TEXT NOT NULL,
  spine_idx    INTEGER NOT NULL,
  char_offset  INTEGER NOT NULL,
  anchor_hash  TEXT NOT NULL,
  created_at   TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE (book_id, mark)
);

CREATE TABLE highlights (
  id                INTEGER PRIMARY KEY,
  book_id           INTEGER NOT NULL REFERENCES books(id) ON DELETE RESTRICT,
  spine_idx         INTEGER NOT NULL,
  chapter_title     TEXT,
  char_offset_start INTEGER NOT NULL,
  char_offset_end   INTEGER NOT NULL,
  text              TEXT NOT NULL,
  context_before    TEXT,
  context_after     TEXT,
  note              TEXT,
  anchor_status     TEXT NOT NULL DEFAULT 'ok',
  created_at        TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at        TEXT
);
CREATE INDEX idx_highlights_book ON highlights(book_id);
