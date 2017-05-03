-- Your SQL goes here
CREATE TABLE welcomes (
    id INTEGER PRIMARY KEY NOT NULL,
    user VARCHAR NOT NULL,
    text VARCHAR NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS UniqueWelcomeUser ON welcomes (user);
