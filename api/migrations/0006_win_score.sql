CREATE OR REPLACE FUNCTION win_score(games bigint, wins bigint) RETURNS float8
    LANGUAGE sql IMMUTABLE
    RETURN CASE WHEN games > 0 THEN
        greatest(0, (wins + 1.9208 - 1.96 * sqrt(wins * (games - wins)::numeric / games + 0.9604)) / (games + 3.8416))::float8
    END;
