CREATE TABLE IF NOT EXISTS todo
(
    id                INTEGER PRIMARY KEY NOT NULL,
    value             TEXT                NOT NULL,
    is_completed      BOOLEAN             NOT NULL DEFAULT 0,
    created_at        DATETIME            NOT NULL DEFAULT CURRENT_TIMESTAMP
);
