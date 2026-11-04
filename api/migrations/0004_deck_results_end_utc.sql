CREATE OR REPLACE VIEW deck_results AS
SELECT m.game_uid, m.app_version, r.deck_key, r.win, m.end_utc
FROM matches m
CROSS JOIN LATERAL (VALUES
    (array_sort(m.seat0_cards), m.winner_seat IS NOT DISTINCT FROM 0),
    (array_sort(m.seat1_cards), m.winner_seat IS NOT DISTINCT FROM 1)
) AS r (deck_key, win)
WHERE m.game_mode = 10 AND NOT m.is_ai AND NOT m.disputed
    AND array_sort(m.seat0_cards) <> array_sort(m.seat1_cards);
