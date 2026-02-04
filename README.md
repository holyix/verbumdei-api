# Verbumdei-api

Backend skeleton for the Verbum Dei project.

## Layout

- `src/main.rs`: Entrypoint; sets up tracing and delegates to the API runner.
- `src/routes/api.rs`: API router + middleware + fallback.
- `src/db.rs`: MongoDB connection/init.
- `src/config.rs`: Environment-driven configuration (host, port, Mongo URI/db).
- `src/resources/health`: Basic health endpoints (`/health`, `/health/db`).
- `src/resources/questions`: Question model, handlers, queries.
- `src/resources/ui`: Static UI catalogs (locales, levels) served from `/v1/ui/*`.

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
- `GET /eras`, `GET /eras/:eraId`, `GET /eras/:eraId/episodes`, `GET /eras/:eraId/episodes/:episodeId`
- `GET /episodes?book=Genesis`
- `GET /v1/ui/locales`, `GET /v1/ui/levels` (frontend pulls locales/levels from here)

## Data utilities

Make targets wrap the scripts in `scripts/`:

- `make load-questions` — load `data/questions.json` into Mongo `questions`
- `make load-eras` — load `data/eras.json` into Mongo `eras`
- `make load-data` — load all collection data (`questions`, `eras`)
- `make dump-questions` — dump Mongo `questions` into `data/questions.json`
- `make dump-eras` — dump Mongo `eras` into `data/eras.json`
- `make dump-data` — dump all collection data (`questions`, `eras`)
- `make validate` — start the API in the background and run the workflow script
- `make fmt` — format Rust code
- `make fmt-check` — check Rust formatting
- `make check` — run lint + format checks
- `make test|lint|build|all`
