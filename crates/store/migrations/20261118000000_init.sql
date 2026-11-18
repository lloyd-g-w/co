-- Users and their credentials.
CREATE TABLE users (
    id            uuid PRIMARY KEY,
    username      text NOT NULL UNIQUE,
    password_hash text NOT NULL,  -- argon2id PHC string
    created_at    timestamptz NOT NULL DEFAULT now()
);

-- Login sessions (bearer tokens; only a hash of the token is stored).
CREATE TABLE auth_tokens (
    token_hash  bytea PRIMARY KEY,
    user_id     uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    created_at  timestamptz NOT NULL DEFAULT now(),
    expires_at  timestamptz NOT NULL
);
CREATE INDEX auth_tokens_user_id ON auth_tokens (user_id);

-- Per-user settings: theme, preferences, defaults.
CREATE TABLE user_settings (
    user_id    uuid PRIMARY KEY REFERENCES users (id) ON DELETE CASCADE,
    settings   jsonb NOT NULL DEFAULT '{}'::jsonb,
    updated_at timestamptz NOT NULL DEFAULT now()
);

-- Tool hosts (machines), known by their persistent host id.
CREATE TABLE hosts (
    id         text PRIMARY KEY,
    user_id    uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    name       text NOT NULL,
    home       text,
    last_seen  timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX hosts_user_id ON hosts (user_id);
