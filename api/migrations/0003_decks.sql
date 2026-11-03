CREATE TABLE decks (
    id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    author_id bigint NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    name text NOT NULL CHECK (char_length(name) BETWEEN 1 AND 60),
    cards uuid[] NOT NULL CHECK (cardinality(cards) = 4),
    deck_key uuid[] GENERATED ALWAYS AS (array_sort(cards)) STORED,
    talents smallint[] NOT NULL CHECK (cardinality(talents) = 4 AND 0 <= ALL (talents) AND 2 >= ALL (talents)),
    guide text NOT NULL DEFAULT '' CHECK (char_length(guide) <= 50000),
    status text NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'public')),
    app_version text NOT NULL,
    game_deck_id text CHECK (char_length(game_deck_id) BETWEEN 1 AND 64),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    published_at timestamptz,
    UNIQUE (author_id, game_deck_id),
    CHECK (deck_key[1] < deck_key[2] AND deck_key[2] < deck_key[3] AND deck_key[3] < deck_key[4])
);

CREATE TABLE votes (
    deck_id bigint NOT NULL REFERENCES decks (id) ON DELETE CASCADE,
    user_id bigint NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (deck_id, user_id)
);
CREATE INDEX votes_user_id_idx ON votes (user_id);

CREATE TABLE comments (
    id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    deck_id bigint NOT NULL REFERENCES decks (id) ON DELETE CASCADE,
    author_id bigint NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    body text NOT NULL CHECK (char_length(body) BETWEEN 1 AND 2000),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX comments_deck_id_idx ON comments (deck_id, id);
CREATE INDEX comments_author_id_idx ON comments (author_id);

CREATE TABLE reports (
    id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    reporter_id bigint NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    deck_id bigint REFERENCES decks (id) ON DELETE CASCADE,
    comment_id bigint REFERENCES comments (id) ON DELETE CASCADE,
    reason text NOT NULL CHECK (char_length(reason) BETWEEN 1 AND 500),
    status text NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'dismissed')),
    created_at timestamptz NOT NULL DEFAULT now(),
    resolved_at timestamptz,
    CHECK (num_nonnulls(deck_id, comment_id) = 1)
);
CREATE UNIQUE INDEX reports_open_idx ON reports (reporter_id, deck_id, comment_id) NULLS NOT DISTINCT
    WHERE status = 'open';
CREATE INDEX reports_reporter_id_idx ON reports (reporter_id);
CREATE INDEX reports_deck_id_idx ON reports (deck_id);
CREATE INDEX reports_comment_id_idx ON reports (comment_id);

CREATE TABLE banned_steam_ids (
    steam_id text PRIMARY KEY CHECK (steam_id ~ '^[0-9]{17}$'),
    banned_at timestamptz NOT NULL
);

CREATE FUNCTION win_rate(games bigint, wins bigint) RETURNS float8
    LANGUAGE sql IMMUTABLE
    RETURN CASE WHEN games >= 20 THEN wins::float8 / games END;

CREATE VIEW deck_results AS
SELECT m.game_uid, m.app_version, r.deck_key, r.win
FROM matches m
CROSS JOIN LATERAL (VALUES
    (array_sort(m.seat0_cards), m.winner_seat IS NOT DISTINCT FROM 0),
    (array_sort(m.seat1_cards), m.winner_seat IS NOT DISTINCT FROM 1)
) AS r (deck_key, win)
WHERE m.game_mode = 10 AND NOT m.is_ai AND NOT m.disputed
    AND array_sort(m.seat0_cards) <> array_sort(m.seat1_cards);
