CREATE TABLE unknown_cards (
    card_id uuid PRIMARY KEY,
    seen_count bigint NOT NULL,
    first_seen timestamptz NOT NULL,
    last_seen timestamptz NOT NULL
);
