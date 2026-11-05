ALTER TABLE devices ADD COLUMN app_version text, ADD COLUMN online_until timestamptz;

ALTER TABLE uploads ADD COLUMN source text CHECK (source IN ('web', 'desktop'));

CREATE INDEX matches_created_at_idx ON matches (created_at);

CREATE TABLE analytics_hourly (
    hour timestamptz PRIMARY KEY,
    web_peak integer NOT NULL,
    app_peak integer NOT NULL,
    web_distinct integer NOT NULL,
    app_distinct integer NOT NULL,
    matches bigint NOT NULL,
    ranked_matches bigint NOT NULL,
    users bigint NOT NULL,
    linked_accounts bigint NOT NULL,
    public_decks bigint NOT NULL,
    devices bigint NOT NULL
);
