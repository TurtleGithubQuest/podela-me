CREATE SCHEMA IF NOT EXISTS auth;
CREATE SCHEMA IF NOT EXISTS subject;

CREATE TYPE subject.legal_form AS ENUM (
    'Sro',
    'As',
    'Vos',
    'Spolek',
    'Nadace',
    'Druzstvo'
);

CREATE TYPE karma AS (
    amount SMALLINT,
    reviews SMALLINT,
    age SMALLINT,
    popularity SMALLINT
);

CREATE DOMAIN ulid AS VARCHAR(26) CHECK (VALUE ~ '^[0-9A-Z]{26}$');

    ------------------------- AUTH -------------------------
CREATE TABLE auth.user
(
    id            ulid PRIMARY KEY,
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

CREATE TABLE auth.session --maybe move this to redis...someday
(
    id ulid PRIMARY KEY,
    user_id ulid REFERENCES auth.user (id) ON DELETE CASCADE,
    ip VARCHAR(45),
    enforce_ip BOOLEAN DEFAULT FALSE,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

    ------------------------- SUBJECT -------------------------
CREATE TABLE subject.organization (
    id ulid PRIMARY KEY,
    form subject.legal_form NOT NULL,
    user_id ulid REFERENCES auth.user(id) ON DELETE SET NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at  TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE subject.website (
    id ulid PRIMARY KEY,
    organization_id ulid REFERENCES subject.organization(id) ON DELETE SET NULL,
    karma karma,
    description VARCHAR(512),
    name VARCHAR(255) NOT NULL,
    domain_name VARCHAR(255) UNIQUE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at  TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);