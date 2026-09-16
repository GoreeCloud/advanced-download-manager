# goreecloud-download-store-sqlite

SQLite-backed durable local job-state adapter for GoreeCloud Advanced Download Manager.

The crate implements the backend-neutral contracts in `goreecloud-download-state` using a bundled, pinned SQLite dependency through rusqlite. It owns schema initialization, optimistic store generations, atomic checkpoint mutation batches, lossless native-path persistence, strict state decoding, reopen validation, and database integrity checks.

It does **not** perform network transfers or partial-file I/O. Filesystem durability, checkpoint ordering relative to partial bytes, truncation/restart reconciliation, and final-file promotion remain separate engine/service responsibilities defined by ADR-0002.

The database contains sensitive operational state, including source URLs required for recovery. Do not log, publish, or treat database files as nonsensitive artifacts.
