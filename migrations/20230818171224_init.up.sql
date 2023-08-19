-- Add up migration script here

CREATE TABLE
    IF NOT EXISTS users (
        id serial PRIMARY KEY UNIQUE,
        user_id VARCHAR(255) NOT NULL UNIQUE,
        username VARCHAR(255) NOT NULL UNIQUE,
        created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP ,
        updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
    );