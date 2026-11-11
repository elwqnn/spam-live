ALTER TABLE replays ADD COLUMN decoder_version integer, ADD COLUMN decode_error text;

CREATE TABLE match_event_sources (
    game_uid text NOT NULL REFERENCES matches (game_uid) ON DELETE CASCADE,
    seat smallint NOT NULL CHECK (seat IN (0, 1)),
    user_id bigint REFERENCES users (id) ON DELETE SET NULL,
    own boolean NOT NULL,
    decoder_version integer NOT NULL,
    decoded_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (game_uid, seat)
);

CREATE TABLE shop_buys (
    game_uid text NOT NULL REFERENCES matches (game_uid) ON DELETE CASCADE,
    seat smallint NOT NULL CHECK (seat IN (0, 1)),
    idx integer NOT NULL,
    turn integer NOT NULL,
    card uuid NOT NULL,
    tox uuid NOT NULL,
    decoder_version integer NOT NULL,
    PRIMARY KEY (game_uid, seat, idx)
);

CREATE TABLE talent_picks (
    game_uid text NOT NULL REFERENCES matches (game_uid) ON DELETE CASCADE,
    seat smallint NOT NULL CHECK (seat IN (0, 1)),
    card uuid NOT NULL,
    talent smallint NOT NULL CHECK (talent >= 1),
    decoder_version integer NOT NULL,
    PRIMARY KEY (game_uid, seat, card)
);

CREATE TABLE card_damage (
    game_uid text NOT NULL REFERENCES matches (game_uid) ON DELETE CASCADE,
    seat smallint NOT NULL CHECK (seat IN (0, 1)),
    card uuid NOT NULL,
    done integer NOT NULL,
    taken integer NOT NULL,
    decoder_version integer NOT NULL,
    PRIMARY KEY (game_uid, seat, card)
);

CREATE TABLE hp_turns (
    game_uid text NOT NULL REFERENCES matches (game_uid) ON DELETE CASCADE,
    seat smallint NOT NULL CHECK (seat IN (0, 1)),
    turn integer NOT NULL,
    hp integer NOT NULL,
    decoder_version integer NOT NULL,
    PRIMARY KEY (game_uid, seat, turn)
);

CREATE OR REPLACE VIEW deck_results AS
SELECT m.game_uid, m.app_version, r.deck_key, r.win, m.end_utc, least(m.seat0_stars, m.seat1_stars) AS stars, r.seat
FROM matches m
CROSS JOIN LATERAL (VALUES
    (array_sort(m.seat0_cards), m.winner_seat IS NOT DISTINCT FROM 0, 0::smallint),
    (array_sort(m.seat1_cards), m.winner_seat IS NOT DISTINCT FROM 1, 1::smallint)
) AS r (deck_key, win, seat)
WHERE m.game_mode = 10 AND NOT m.is_ai AND NOT m.disputed
    AND array_sort(m.seat0_cards) <> array_sort(m.seat1_cards);
