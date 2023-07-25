CREATE TABLE IF NOT EXISTS todos
(
    id                INTEGER PRIMARY KEY NOT NULL,
    value             TEXT                NOT NULL,
    is_completed      BOOLEAN             NOT NULL DEFAULT 0
);
