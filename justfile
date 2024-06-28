set dotenv-load := true

dev:
    cargo watch -x run

create-migration name:
    sqlx migrate add {{name}}

migrate:
    sqlx migrate run

reset-db:
    sqlx database drop
    sqlx database create
    sqlx migrate run

prepare:
    cargo sqlx prepare

