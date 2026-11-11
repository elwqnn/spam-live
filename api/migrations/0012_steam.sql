CREATE TABLE steam_players (
    at timestamptz PRIMARY KEY,
    players integer NOT NULL CHECK (players >= 0)
);

CREATE TABLE steam_news (
    gid text PRIMARY KEY,
    title text NOT NULL,
    url text NOT NULL,
    published_at timestamptz NOT NULL,
    feed text NOT NULL
);
