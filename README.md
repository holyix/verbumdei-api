# Verbumdei-api

Backend skeleton for the Verbum Dei project.

## Layout

- `src/main.rs`: Entrypoint; sets up tracing and delegates to the API runner.
- `src/routes/api.rs`: API router + middleware + fallback.
- `src/db.rs`: MongoDB connection/init.
- `src/config.rs`: Environment-driven configuration (host, port, Mongo URI/db).
- `src/resources/health`: Basic health endpoints (`/health`, `/health/db`).
- `src/resources/questions`: Question model, handlers, queries.
- `src/resources/ui`: Static UI catalogs (locales, levels, UI text) served from `/v1/ui/*`.

## Running (once Rust is installed)

```sh
cp .env.example .env
cargo run
```

or with file-watch + debug logging:

```sh
make run-dev
```

## API surface (current)

- `GET /health` and `/health/db`
- `GET /v1/questions`, `GET /v1/questions/:id`, `POST /v1/questions`, `DELETE /v1/questions/:id`
- `GET /v1/ui/locales`, `GET /v1/ui/levels` (frontend pulls locales/levels from here)

## Data utilities

Make targets wrap the scripts in `scripts/`:

- `make load-data` — load `data/questions.json` into Mongo
- `make dump-data` — dump Mongo questions into `data/questions.json`
- `make validate` — start the API in the background and run the workflow script
- `make test|lint|build|all`
