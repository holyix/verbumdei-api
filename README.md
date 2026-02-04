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

- Health: `GET /health`, `GET /health/db`
- Questions: `GET /v1/questions`, `GET /v1/questions/:id`, `POST /v1/questions`, `DELETE /v1/questions/:id`
- UI catalogs: `GET /v1/ui/locales`, `GET /v1/ui/levels` (frontend pulls locales/levels from here)
- Eras + episodes (both unversioned and `/v1/*` aliases are available):
  - `GET /v1/eras` (`/eras`)
  - `GET /v1/eras/:eraId` (`/eras/:eraId`)
  - `GET /v1/eras/:eraId/episodes` (`/eras/:eraId/episodes`)
  - `GET /v1/eras/:eraId/episodes/:episodeId` (`/eras/:eraId/episodes/:episodeId`)
  - `GET /v1/episodes?book=Genesis` (`/episodes?book=Genesis`)

## Eras API localization

Eras endpoints support localized content with two inputs:

- Query param: `?lang=<code>`
- Header: `Accept-Language: <value>`

Supported languages: `en`, `es`, `pt`, `sv`.

Resolution order:

1. `lang` query parameter (highest priority)
2. First supported language from `Accept-Language`
3. Fallback to `en`

Notes:

- Regional tags are normalized to base language (`es-MX` -> `es`, `sv-SE` -> `sv`).
- Unsupported languages (for example `de`) fall back to English.

Examples:

```sh
curl "http://localhost:3000/v1/eras?lang=es"
curl -H "Accept-Language: fr-FR, sv-SE;q=0.9, en;q=0.8" \
  "http://localhost:3000/v1/eras/exodus/episodes/sinai"
curl "http://localhost:3000/v1/episodes?book=Genesis&lang=pt"
```

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
