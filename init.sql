-- Script to initialize the database.

PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS xwords(
    date DATE NOT NULL,
    solved BOOLEAN NOT NULL,
    duration INTEGER
);

CREATE TABLE IF NOT EXISTS last_solve(
    id INTEGER REFERENCES xwords(rowid)
);