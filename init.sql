-- Script to initialize the database.

PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS xwords(
    date DATE NOT NULL PRIMARY KEY,
    solved BOOLEAN NOT NULL,
    duration INTEGER
);

CREATE TABLE IF NOT EXISTS misc(
    k TEXT NOT NULL PRIMARY KEY,
    v TEXT
);