CREATE SCHEMA IF NOT EXISTS auth;

CREATE TABLE auth.user
(
    id            VARCHAR(26) PRIMARY KEY, -- ulids
    email         VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255)        NOT NULL,
    language    VARCHAR(6) NOT NULL      DEFAULT 'en-US',
    name          VARCHAR(50) UNIQUE  NOT NULL,
    is_admin    BOOLEAN                  DEFAULT false,
    is_active   BOOLEAN                  DEFAULT true,
    is_verified BOOLEAN                  DEFAULT false,
    last_login    TIMESTAMP WITH TIME ZONE,
    created_at  TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at  TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE auth.session
(
    id      BIGSERIAL PRIMARY KEY,
    user_id VARCHAR(26) REFERENCES auth.user (id) ON DELETE CASCADE,
    token   TEXT NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);