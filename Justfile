db_container := "spam-tcg-db"

db:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! docker container inspect {{db_container}} >/dev/null 2>&1; then
        docker run -d --name {{db_container}} -p 127.0.0.1::5432 \
            -e POSTGRES_USER=spam -e POSTGRES_PASSWORD=spam -e POSTGRES_DB=spam \
            -v spam-tcg-pgdata:/var/lib/postgresql postgres:18 >/dev/null
    else
        docker start {{db_container}} >/dev/null
    fi
    until docker exec {{db_container}} pg_isready -U spam -d spam >/dev/null 2>&1; do sleep 0.5; done
    echo "DATABASE_URL=$(just db-url)"

db-stop:
    docker stop {{db_container}}

[private]
db-url:
    @echo "postgres://spam:spam@127.0.0.1:$(docker port {{db_container}} 5432/tcp | head -n1 | rev | cut -d: -f1 | rev)/spam"

dev-api:
    #!/usr/bin/env bash
    set -euo pipefail
    cd api
    if [ -f .env ]; then set -a; . ./.env; set +a; fi
    export DATABASE_URL="${DATABASE_URL:-$(just db-url)}"
    export PUBLIC_URL="${PUBLIC_URL:-http://localhost:5173}"
    export CATALOG_DIR="${CATALOG_DIR:-../catalog}"
    cargo run

dev-web:
    cd web && bun run dev

dev-session admin="":
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -n "{{admin}}" ] && [ "{{admin}}" != "admin" ]; then echo "usage: just dev-session [admin]" >&2; exit 1; fi
    token=$(head -c 32 /dev/urandom | od -An -tx1 | tr -d ' \n')
    docker exec -i {{db_container}} psql -q -v ON_ERROR_STOP=1 -U spam -d spam >/dev/null <<SQL
    WITH u AS (
        INSERT INTO users (steam_id, display_name, is_admin)
        VALUES ('76561190000000001', 'Dev user', {{ if admin == "admin" { "true" } else { "false" } }})
        ON CONFLICT (steam_id) DO UPDATE SET is_admin = EXCLUDED.is_admin
        RETURNING id
    )
    INSERT INTO sessions (token_hash, user_id, expires_at)
    SELECT sha256(convert_to('$token', 'UTF8')), id, now() + interval '30 days' FROM u;
    SQL
    echo "session=$token"

api-types:
    cd api && cargo run -q -- --openapi | (cd ../web && bunx openapi-typescript --empty-objects-unknown -o src/lib/api.d.ts)

check:
    cd api && cargo clippy --all-targets -- -D warnings
    cd web && bun run check && bun run lint

test *args: && test-web
    cd api && DATABASE_URL="${DATABASE_URL:-$(just db-url)}" cargo test {{args}}

test-web:
    cd web && bun run test
