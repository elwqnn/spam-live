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

check:
    cd api && cargo clippy --all-targets -- -D warnings

test *args:
    cd api && DATABASE_URL="${DATABASE_URL:-$(just db-url)}" cargo test {{args}}
