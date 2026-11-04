set dotenv-load

db_port := "55432"
container := "spam-live-db"
image := "ghcr.io/elwqnn/spam-live"
game_data := home_directory() / ".local/share/Steam/steamapps/common/SPAM/SPAM_Data"

export DATABASE_URL := env("DATABASE_URL", "postgres://spam:spam@127.0.0.1:" + db_port + "/spam")
export PUBLIC_URL := env("PUBLIC_URL", "http://localhost:5173")
export CATALOG_DIR := env("CATALOG_DIR", justfile_directory() / "catalog")

[private]
default:
    @just --list --unsorted

[group('dev')]
[doc('Start the database, the API and the website. Ctrl+C stops all.')]
dev: db
    #!/usr/bin/env bash
    set -m
    trap 'kill -- $(jobs -p | sed "s/^/-/") 2>/dev/null' EXIT
    trap exit INT TERM
    (cd api && CARGO_TERM_COLOR=always cargo run 2>&1 | awk '{ print "api | " $0; fflush() }') &
    (cd web && FORCE_COLOR=1 bun run dev 2>&1 | awk '{ print "web | " $0; fflush() }') &
    wait

[group('dev')]
[doc('Start the local Postgres container.')]
db:
    @docker start {{container}} >/dev/null 2>&1 || docker run -d --name {{container}} -p 127.0.0.1:{{db_port}}:5432 -e POSTGRES_USER=spam -e POSTGRES_PASSWORD=spam -e POSTGRES_DB=spam -v spam-live-pgdata:/var/lib/postgresql postgres:18 >/dev/null
    @until docker exec {{container}} pg_isready -U spam -d spam >/dev/null 2>&1; do sleep 0.5; done

[group('dev')]
[doc('Stop the local Postgres container.')]
db-stop:
    docker stop {{container}}

[group('dev')]
[doc('Run the API on port 8080.')]
api: db
    cd api && cargo run

[group('dev')]
[doc('Run the website on port 5173.')]
web:
    cd web && bun run dev

[group('dev')]
[doc('Export the card art of the game to web/static/art/. Never commit it.')]
art data=game_data:
    uv run datamine/art.py "{{data}}" web/static/art/

[group('dev')]
[doc('Run the desktop app against the local website.')]
desktop:
    cd desktop && SPAM_SITE_URL="${SPAM_SITE_URL:-http://localhost:5173}" bun tauri dev

[group('dev')]
[doc('Create a local login and print its session cookie. Pass "admin" for an admin login.')]
session admin="": db
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -n "{{admin}}" ] && [ "{{admin}}" != "admin" ]; then echo "usage: just session [admin]" >&2; exit 1; fi
    token=$(head -c 32 /dev/urandom | od -An -tx1 | tr -d ' \n')
    docker exec -i {{container}} psql -q -v ON_ERROR_STOP=1 -U spam -d spam >/dev/null <<SQL
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

[group('dev')]
[doc('Regenerate web/src/lib/api.d.ts from the API.')]
api-types:
    cd api && cargo run -q -- --openapi | (cd ../web && bunx openapi-typescript --empty-objects-unknown -o src/lib/api.d.ts)

[group('check')]
[doc('Run the linters and type checks of all apps.')]
check:
    cd api && cargo clippy --all-targets -- -D warnings
    cd web && bun run check && bun run lint
    cd desktop/src-tauri && cargo clippy --all-targets -- -D warnings
    cd desktop && bun run check && bun run lint

[group('check')]
[doc('Run the tests of all apps. Arguments go to the API tests.')]
test *args: db
    cd api && cargo test {{args}}
    cd web && bun run test
    cd desktop/src-tauri && cargo test

[group('release')]
[doc('Build the API and web images.')]
images:
    @test -d web/static/art/card -a -d web/static/art/avatar -a -f web/static/art/ui/card_mask.webp || { echo "web/static/art/ is missing or incomplete: run just art first" >&2; exit 1; }
    docker build -f api/Dockerfile -t {{image}}-api:latest .
    docker build -t {{image}}-web:latest web

[group('release')]
[doc('Build and push the images with the tags latest and the commit SHA.')]
push: images
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -n "$(git status --porcelain)" ]; then echo "commit all changes before you push" >&2; exit 1; fi
    sha=$(git rev-parse --short HEAD)
    for app in api web; do
        docker tag {{image}}-$app:latest {{image}}-$app:$sha
        docker push {{image}}-$app:latest
        docker push {{image}}-$app:$sha
    done

[group('release')]
[doc('Pull and restart the stack. Run it on the VPS.')]
deploy:
    git pull --ff-only
    docker compose -f deploy/compose.yaml pull
    docker compose -f deploy/compose.yaml up -d
