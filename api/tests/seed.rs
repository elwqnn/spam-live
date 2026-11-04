#![cfg(feature = "seed")]

mod common;

use sqlx::PgPool;

#[test]
fn seed_refuses_remote_hosts() {
    assert!(api::seed::check_host("postgres://spam:spam@example.com:5432/spam").is_err());
    assert!(api::seed::check_host("postgres://spam:spam@127.0.0.1:55432/spam?host=example.com").is_err());
    assert!(api::seed::check_host("postgres://spam:spam@127.0.0.1:55432/spam").is_ok());
    assert!(api::seed::check_host("postgres://spam:spam@localhost:55432/spam").is_ok());
}

#[sqlx::test]
async fn seed_is_repeatable_and_covers_every_card(pool: PgPool) {
    let catalog = common::real_catalog();
    api::seed::run(&pool, &catalog).await.unwrap();
    let first = api::seed::counts(&pool).await.unwrap();
    api::seed::run(&pool, &catalog).await.unwrap();
    assert_eq!(api::seed::counts(&pool).await.unwrap(), first);
    assert!(first.iter().all(|(_, n)| *n > 0), "{first:?}");

    let (week, before, old): (i64, i64, i64) = sqlx::query_as(
        "SELECT min(a), min(b), min(c) FROM (
             SELECT count(*) FILTER (WHERE r.app_version = $2 AND r.end_utc >= now() - interval '7 days') AS a,
                 count(*) FILTER (WHERE r.app_version = $2 AND r.end_utc >= now() - interval '14 days'
                     AND r.end_utc < now() - interval '7 days') AS b,
                 count(*) FILTER (WHERE r.app_version <> $2) AS c
             FROM unnest($1::uuid[]) AS k (card) LEFT JOIN deck_results r ON k.card = ANY (r.deck_key)
             GROUP BY k.card
         ) t",
    )
    .bind(catalog.card_ids())
    .bind(catalog.latest())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(week >= 40 && before >= 40 && old >= 40, "{week} {before} {old}");

    let keys: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM (SELECT deck_key FROM deck_results WHERE app_version = $1
             GROUP BY deck_key HAVING count(*) >= 20) t",
    )
    .bind(catalog.latest())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(keys >= 15, "{keys}");
}
