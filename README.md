# untis_telegram_bot

A service Telegram bot that periodically fetches schedules from Untis and posts status/notifications to a target chat. The workspace is split into three crates for clean layering:

- Root crate (src/) — orchestration: observers, worker lifecycle, message formatting ([src/main.rs](src/main.rs)).
- db/ — PostgreSQL access with Diesel using an Active Record style API ([db/src/models.rs](db/src/models.rs)).
- untis/ — Untis JSON‑RPC + REST client ([untis/src/client.rs](untis/src/client.rs), [untis/src/jsonrpc.rs](untis/src/jsonrpc.rs)).

## Techniques used

- Active Record pattern on Diesel models: instance methods like `insert()`, `update()`, `reload()`, `delete()` in [db/src/models.rs](db/src/models.rs) simplify data lifecycle and keep call sites clean.
- Optimistic concurrency: `update_optimistic(expected_updated_at, changeset)` gates updates on a version check (`updated_at`), reducing contention while preserving correctness.
- Lazy, global connection pool via `OnceCell` + `r2d2` in [db/src/diesel_impl.rs](db/src/diesel_impl.rs), providing a single thread‑safe pool for the process.
- Hybrid transport: JSON‑RPC for most methods, but direct REST for homeworks in [untis/src/client.rs](untis/src/client.rs), extending beyond the public RPC surface.
- Structured async logging with environment‑driven filters (`tracing` + `tracing-subscriber`) and different production/debug formatters in [src/main.rs](src/main.rs).
- Worker orchestration with Tokio `JoinSet`: observer loop starts/stops workers based on DB state; crashed workers are restarted in the next tick.
- Periodic data hygiene: a daily cleanup task removes outdated lessons to keep tables lean.
- JSON encoding/decoding via `serde` with hand‑rolled JSON‑RPC envelopes in [untis/src/jsonrpc.rs](untis/src/jsonrpc.rs). For general JSON concepts see [MDN JSON docs](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/JSON).
- Cookie handling and DNS options with `reqwest` (cookies, hickory‑dns). MDN reference for HTTP cookies: [MDN HTTP cookies](https://developer.mozilla.org/docs/Web/HTTP/Cookies). For date/time concepts in client platforms, see [MDN Intl.DateTimeFormat](https://developer.mozilla.org/docs/Web/JavaScript/Reference/Global_Objects/Intl/DateTimeFormat).

## Notable libraries and tech

- Diesel (ORM) — [diesel.rs](https://diesel.rs/)
- r2d2 (pool) — [crates.io/crates/r2d2](https://crates.io/crates/r2d2)
- Teloxide (Telegram bot framework) — [github.com/teloxide/teloxide](https://github.com/teloxide/teloxide)
- reqwest (HTTP client) — [crates.io/crates/reqwest](https://crates.io/crates/reqwest)
- tracing — [crates.io/crates/tracing](https://crates.io/crates/tracing)
- serde / serde_json — [serde.rs](https://serde.rs/)
- strum (derive for enums) — [crates.io/crates/strum](https://crates.io/crates/strum)
- impl-tools (macro utilities) — [crates.io/crates/impl-tools](https://crates.io/crates/impl-tools)
- restructed (derive models/views/patches) — [crates.io/crates/restructed](https://crates.io/crates/restructed)

## Source links referenced

- Observer & workers: [src/main.rs](src/main.rs)
- Models and Active Record API: [db/src/models.rs](db/src/models.rs)
- Connection pool and migrations bootstrap: [db/src/diesel_impl.rs](db/src/diesel_impl.rs)
- Untis high‑level client: [untis/src/client.rs](untis/src/client.rs)
- JSON‑RPC low‑level transport: [untis/src/jsonrpc.rs](untis/src/jsonrpc.rs)
