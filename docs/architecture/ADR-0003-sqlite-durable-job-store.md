---
title: "ADR-0003 — SQLite Durable Job Store"
document_type: "Architecture Decision Record"
version: "v0.2"
product_version: "0.1.0"
release_lifecycle: "Development"
status: "Accepted"
classification: "Public"
last_updated: "2026-09-16"
---

# ADR-0003 — SQLite Durable Job Store

## Decision

Use **SQLite** as the first durable local job-state backend for GoreeCloud Advanced Download Manager, accessed from Rust through **rusqlite 0.40.2** with its `bundled` and `fallible_uint` features and default features disabled.

The SQLite adapter will live in a separate `crates/download-store-sqlite` crate and must implement the backend-neutral durability contract already defined by `crates/download-state` and ADR-0002. The shared domain and recovery crates remain independent of SQLite.

The initial database policy is intentionally conservative:

- local embedded database; no external database service or mandatory account;
- SQLite rollback-journal mode (`DELETE`) rather than WAL for the initial single-service writer model;
- `synchronous=EXTRA` for the durable job-state database;
- parameterized SQL for all data values;
- explicit schema versioning and fail-closed handling of unsupported future schemas;
- optimistic generation checking for durable mutation batches;
- explicit typed columns rather than opaque whole-object serialization;
- no application logs containing persisted source URLs or other sensitive values;
- no claim that SQLite persistence alone makes partial-file bytes durable—the filesystem checkpoint-ordering rules in ADR-0002 remain independently required.

## Verified dependency baseline

Dependency selection was reviewed on September 15–16, 2026 against current upstream primary sources and applicable GoreeCloud dependency/open-source governance.

### rusqlite

Upstream release `v0.40.2` was published August 8, 2026. Its package metadata declares:

- package version `0.40.2`;
- MIT license;
- Rust 2021 edition;
- `bundled` support for compiling SQLite sources rather than relying on a system SQLite installation;
- `fallible_uint` support for checked conversion of SQLite integer values into unsigned Rust integer types;
- `libsqlite3-sys` dependency version `0.38.2`.

Upstream source:

- https://github.com/rusqlite/rusqlite/releases/tag/v0.40.2
- https://github.com/rusqlite/rusqlite/blob/v0.40.2/Cargo.toml

The `v0.40.2` release lowered rusqlite's minimum supported Rust version to 1.88.0, which is compatible with this repository's pinned Rust 1.98.1 toolchain.

### bundled SQLite

`libsqlite3-sys 0.38.2` is MIT-licensed. Its bundled feature compiles the included SQLite source through the Rust `cc` build dependency. The SQLite header shipped with the verified rusqlite `v0.40.2` tag identifies bundled SQLite **3.53.2**, source date June 3, 2026.

The upstream build script explicitly detects Android targets and sets `SQLITE_TEMP_STORE=3` for bundled SQLite on Android. Android compilation still requires the applicable NDK/C compiler environment and remains subject to GoreeCloud CI validation; upstream support is not a substitute for a successful GoreeCloud target build.

Upstream source:

- https://github.com/rusqlite/rusqlite/blob/v0.40.2/libsqlite3-sys/Cargo.toml
- https://github.com/rusqlite/rusqlite/blob/v0.40.2/libsqlite3-sys/build.rs
- https://github.com/rusqlite/rusqlite/blob/v0.40.2/libsqlite3-sys/sqlite3/sqlite3.h

### SQLite licensing and durability

SQLite's official project documentation states that SQLite deliverable code and documentation are dedicated to the public domain. This satisfies the GoreeCloud open-source control objective without introducing a proprietary database service.

SQLite's official durability documentation distinguishes journal and synchronous modes. WAL with `synchronous=NORMAL` can lose the most recently committed transaction after operating-system crash or power loss even while remaining consistent. Because this database represents durable transfer checkpoints, the initial adapter deliberately avoids optimizing for lower-sync write throughput and instead uses the simpler rollback-journal model with `synchronous=EXTRA`.

Official SQLite references:

- https://www.sqlite.org/copyright.html
- https://www.sqlite.org/pragma.html#pragma_synchronous

## Why SQLite

SQLite fits the current GoreeCloud Advanced Download Manager boundary because the durable job database is local application state owned by one download service rather than a shared network database.

Compared with an external database service, SQLite removes additional service discovery, authentication, network, deployment, backup, and availability dependencies from the basic local downloader. It remains self-contained, portable, transactional, widely supported, and compatible with headless operation.

Using rusqlite directly avoids introducing an async database framework, executor/runtime integration, query-macro toolchain, or multi-database abstraction before those capabilities are justified. The Download Service may later isolate blocking database operations behind its own worker/thread boundary without changing the persistence contract.

## Why bundled SQLite

The `bundled` feature is selected to make the SQLite implementation version controlled by the repository dependency graph rather than silently varying with the host operating system.

This improves reproducibility across Linux, Windows, and Android and avoids making a system SQLite package a hidden mandatory dependency. It also increases the source/build dependency surface, so the exact rusqlite/libsqlite3-sys/SQLite baseline must remain visible in `Cargo.lock`, dependency review, vulnerability scanning, and future update records.

Default rusqlite features are disabled because the first adapter does not need the optional statement-cache or wasm-oriented default feature set. The adapter enables only `bundled` plus `fallible_uint`. `fallible_uint` is used for bounded SQLite metadata count reads such as table-existence checks; it does not change the durable representation of download byte counters, which remain fixed-width 8-byte blobs so the full `u64` range is preserved.

## Journal and synchronous policy

The first production-shaped store uses one owning service process and does not require unrelated clients to open the database directly. Therefore the concurrency advantages of WAL do not currently justify its additional persistent sidecar/checkpoint/backup semantics.

The adapter configures:

- `PRAGMA journal_mode=DELETE`;
- `PRAGMA synchronous=EXTRA`;
- `PRAGMA foreign_keys=ON`;
- a bounded busy timeout.

WAL may be reconsidered if verified service architecture or measured workload shows that database reader/writer concurrency is a real bottleneck. Any switch must include backup, checkpoint, power-loss, and recovery validation rather than being treated as a performance-only toggle.

## Schema and data representation

The first schema stores:

- schema version and store generation;
- job identifier;
- source URL required for transfer recovery;
- exact final and staging paths;
- job state and state detail where applicable;
- downloaded and expected byte counts;
- ETag and Last-Modified validators.

Unsigned 64-bit counters are encoded as fixed-width 8-byte big-endian blobs so values are not silently constrained to SQLite's signed 64-bit integer range.

Filesystem paths are stored losslessly using an explicit platform encoding rather than assuming every native path is UTF-8. Unix-family targets store native path bytes; Windows stores UTF-16 code units in little-endian form. The encoding identifier is persisted and a store from an incompatible platform encoding fails closed instead of corrupting a path.

The database is machine-local operational state. Cross-operating-system migration of active partial download jobs is not an initial requirement.

## Sensitive URL handling

Download URLs can contain signed parameters, tokens, private identifiers, or other sensitive information. Durable resume requires the current source URL to be recoverable, so the initial schema stores it as sensitive local application state.

The adapter must never include URL values in `Debug`, error messages, SQL text, or routine diagnostics. The database file itself must therefore be treated as sensitive application data and covered by later platform storage-permission, Privacy Shield, backup, and at-rest protection decisions.

This ADR does **not** claim application-layer encryption of the SQLite database. Stable/release acceptance must not imply encrypted-at-rest protection unless the applicable platform implementation and evidence establish it.

## Transaction model

Every `CommitBatchV1` is applied within one SQLite transaction.

The adapter must:

1. acquire a write transaction;
2. read the authoritative store generation;
3. reject the batch if the generation differs from `expected_generation`;
4. apply every upsert/remove mutation;
5. advance generation exactly to `next_generation`;
6. commit atomically.

Any error before commit rolls back the entire batch. Generation conflict is an expected concurrency result, not permission to overwrite newer state.

## Migration and rollback

Schema initialization creates version `1` only for a new empty store.

A store with a newer unsupported schema fails closed. A recognized older schema must enter an explicit migration path; the adapter must not silently reinterpret it as current.

Future schema migrations must execute transactionally where SQLite permits and must preserve a recoverable pre-migration state as required by ADR-0002. A later migration that also transforms external partial-file state requires an explicit recovery design rather than assuming the database transaction covers filesystem changes.

## Alternatives considered

### Custom file or journal format

Rejected for the initial production-shaped backend. Reimplementing transaction logging, concurrency control, crash recovery, integrity behavior, migrations, and inspection tooling would add substantial correctness risk without a demonstrated product benefit.

### SQLx + SQLite

Not selected for the first local store. SQLx can support SQLite and async workflows, but the current download-state persistence boundary does not require multi-database portability or async query APIs. Adding that abstraction now would increase dependency/runtime complexity before a concrete need exists.

### External PostgreSQL or another database service

Rejected for the local engine. It would violate the local-first objective by adding a network/service dependency to basic downloading and would materially increase deployment, authentication, recovery, and administration burden.

### SQLite WAL + `synchronous=NORMAL`

Not selected for the initial durability baseline. It is attractive for throughput/concurrency, but the current store has a single owning service and durable checkpoint semantics matter more than avoiding a WAL sync on each commit. Official SQLite documentation also explicitly notes that WAL/NORMAL may lose recent committed transactions after system crash or power loss.

## Consequences

This decision introduces the first non-workspace third-party Rust dependency chain into the repository. Exact versions must be locked, reviewed, scanned, and updated deliberately under GoreeCloud security/dependency governance.

The adapter may now implement actual durable job metadata persistence and reopen/recovery tests while preserving the source-level separation between job semantics, HTTP safety, backend-neutral recovery, filesystem durability, and network execution.

Successful database tests will establish only the tested persistence boundary. They will not by themselves prove partial-file durability, end-to-end restart recovery, supported clients, release readiness, Platform-System acceptance, or production qualification.

## Revisit conditions

Revisit this ADR when verified evidence shows one of the following:

- SQLite cannot satisfy required durability, migration, corruption-recovery, platform, or performance behavior;
- direct multi-client database access becomes a justified requirement;
- database write contention materially affects transfer performance;
- another backend materially improves security or recoverability without adding disproportionate dependency burden;
- platform requirements make the bundled SQLite build unsuitable;
- a security or licensing change invalidates the verified dependency baseline.
