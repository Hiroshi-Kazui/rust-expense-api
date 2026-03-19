CREATE TYPE user_role AS ENUM ('admin', 'user');

CREATE TABLE users (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        VARCHAR(255) NOT NULL,
    email       VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    role        user_role NOT NULL DEFAULT 'user',
    created_at  TIMESTAMP NOT NULL DEFAULT NOW()
);
