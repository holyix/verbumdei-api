# Verbumdei-api

Backend skeleton for the Verbum Dei project.

## Layout

- `src/main.rs`: Axum entrypoint and router setup.
- `src/config.rs`: Environment-driven configuration (host, port, Mongo URI/db).
- `src/routes/health.rs`: Basic health endpoint at `/health`.

## Running (once Rust is installed)

```sh
cp .env.example .env
cargo run
```
