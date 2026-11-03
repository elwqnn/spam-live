CREATE TABLE users (
    id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    steam_id text NOT NULL UNIQUE CHECK (steam_id ~ '^[0-9]{17}$'),
    display_name text NOT NULL,
    avatar_url text,
    is_admin boolean NOT NULL DEFAULT false,
    banned_at timestamptz,
    spam_account_id uuid UNIQUE,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE sessions (
    token_hash bytea PRIMARY KEY,
    user_id bigint NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    created_at timestamptz NOT NULL DEFAULT now(),
    expires_at timestamptz NOT NULL
);
CREATE INDEX sessions_user_id_idx ON sessions (user_id);

CREATE TABLE devices (
    id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    user_id bigint NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    token_hash bytea NOT NULL UNIQUE,
    created_at timestamptz NOT NULL DEFAULT now(),
    last_used_at timestamptz
);
CREATE INDEX devices_user_id_idx ON devices (user_id);

CREATE TABLE device_codes (
    device_code_hash bytea PRIMARY KEY,
    user_code_hash bytea NOT NULL UNIQUE,
    user_id bigint REFERENCES users (id) ON DELETE CASCADE,
    expires_at timestamptz NOT NULL
);

CREATE TABLE matches (
    game_uid text PRIMARY KEY CHECK (game_uid ~ '^match-[0-9]+$'),
    app_version text NOT NULL,
    game_mode integer NOT NULL,
    is_ai boolean NOT NULL,
    start_utc timestamptz NOT NULL,
    end_utc timestamptz NOT NULL,
    duration_ms bigint NOT NULL,
    turn_count integer NOT NULL,
    seat0_cards uuid[] NOT NULL CHECK (cardinality(seat0_cards) = 4),
    seat1_cards uuid[] NOT NULL CHECK (cardinality(seat1_cards) = 4),
    winner_seat smallint CHECK (winner_seat IN (0, 1)),
    is_draw boolean NOT NULL,
    disputed boolean NOT NULL DEFAULT false,
    seat0_stars integer,
    seat1_stars integer,
    created_at timestamptz NOT NULL DEFAULT now(),
    CHECK (is_draw = (winner_seat IS NULL))
);

CREATE TABLE uploads (
    game_uid text NOT NULL REFERENCES matches (game_uid) ON DELETE CASCADE,
    user_id bigint NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    seat smallint NOT NULL CHECK (seat IN (0, 1)),
    uploaded_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (game_uid, user_id)
);
CREATE INDEX uploads_user_id_idx ON uploads (user_id);

CREATE TABLE replays (
    game_uid text NOT NULL,
    user_id bigint NOT NULL,
    data bytea NOT NULL,
    uploaded_at timestamptz NOT NULL DEFAULT now(),
    expires_at timestamptz NOT NULL,
    PRIMARY KEY (game_uid, user_id),
    FOREIGN KEY (game_uid, user_id) REFERENCES uploads (game_uid, user_id) ON DELETE CASCADE
);
CREATE INDEX replays_expires_at_idx ON replays (expires_at);
