CREATE TABLE password_reset_tokens (
    email VARCHAR NOT NULL PRIMARY KEY,
    token VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);