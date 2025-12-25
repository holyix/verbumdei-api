# Verbumdei-api

Backend skeleton for the Verbum Dei project.

## Layout

- `src/main.rs`: Entrypoint; sets up tracing and delegates to the API runner.
- `src/api.rs`: API builder/run loop, router, and fallback.
- `src/db.rs`: MongoDB connection/init.
- `src/config.rs`: Environment-driven configuration (host, port, Mongo URI/db).
- `src/resources/health/handler.rs`: Basic health endpoint at `/health`.
- `src/resources/questions/model.rs`: Question model and DTO mapping.
- `src/resources/questions/handler.rs`: HTTP handlers for questions.
- `src/resources/questions/queries.rs`: `GET /v1/questions/:id` to fetch a question by id.

## Running (once Rust is installed)

```sh
cp .env.example .env
cargo run
```
