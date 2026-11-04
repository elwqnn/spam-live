CREATE OR REPLACE FUNCTION win_rate(games bigint, wins bigint) RETURNS float8
    LANGUAGE sql IMMUTABLE
    RETURN CASE WHEN games > 0 THEN wins::float8 / games END;
