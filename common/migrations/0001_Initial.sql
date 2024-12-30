CREATE SCHEMA IF NOT EXISTS auth;

-- AUTH SCHEMA
CREATE TABLE auth.users
(
    id            UUID PRIMARY KEY, -- ulids
    email         VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255)        NOT NULL,
    username      VARCHAR(50) UNIQUE  NOT NULL,
    is_active     BOOLEAN                  DEFAULT true,
    is_verified   BOOLEAN                  DEFAULT false,
    last_login    TIMESTAMP WITH TIME ZONE,
    created_at    TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at    TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE auth.sessions
(
    id         BIGSERIAL PRIMARY KEY,
    user_id    UUID REFERENCES auth.users (id) ON DELETE CASCADE,
    token      TEXT                     NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);