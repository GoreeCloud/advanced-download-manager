---
title: "ADR-0004 — Native Single-Stream HTTP/HTTPS Runtime"
document_type: "Architecture Decision Record"
version: "v0.1"
product_version: "0.1.0"
release_lifecycle: "Development"
status: "Accepted"
classification: "Public"
last_updated: "2026-09-17"
---

# ADR-0004 — Native Single-Stream HTTP/HTTPS Runtime

## Decision

Use a dedicated Rust `crates/download-runtime` layer as the first concrete local transfer orchestrator and use **reqwest 0.13.5** as the initial native HTTP/HTTPS client with:

- exact dependency pin `=0.13.5`;
- default reqwest features disabled;
- only the `blocking` and `rustls` features enabled for this milestone;
- explicit Rustls TLS backend selection;
- automatic redirect following disabled;
- automatic system-proxy discovery disabled;
- `Accept-Encoding: identity` on transfer requests so persisted byte offsets remain meaningful for range resume;
- request planning and response disposition delegated to `crates/download-http` rather than duplicated in the transport layer.

The first runtime is deliberately synchronous and single-stream. It is a native engine/library boundary, not a supported UI, service, CLI download command, Android transport, or release.

## Verified dependency baseline

The selected reqwest release is `0.13.5`. Its package documentation identifies Rust `1.85.0` as the minimum supported Rust version, which is below this repository's pinned Rust `1.98.1` toolchain.

The reqwest client enables system proxy discovery by default; this milestone explicitly calls `ClientBuilder::no_proxy()` so environment variables such as `HTTP_PROXY`, `HTTPS_PROXY`, or `ALL_PROXY` cannot silently change the network path before GoreeCloud has designed and authorized proxy policy.

Reqwest follows redirects by default; this milestone explicitly selects `redirect::Policy::none()` so a source URL containing signed parameters, credentials, or private identifiers is not automatically forwarded to a second destination.

Upstream references:

- https://docs.rs/reqwest/0.13.5/reqwest/
- https://docs.rs/reqwest/0.13.5/reqwest/blocking/struct.ClientBuilder.html
- https://docs.rs/reqwest/0.13.5/reqwest/redirect/struct.Policy.html

The generated `Cargo.lock` is authoritative for the complete resolved transitive dependency graph.

## Runtime ordering contract

The runtime consumes the established domain, HTTP, durable-state, SQLite, and staging-file layers rather than weakening their boundaries.

For each successfully read body chunk:

1. append the bytes to the staging artifact;
2. flush and synchronize those staging bytes;
3. derive the new downloaded-byte checkpoint;
4. commit that checkpoint through the generation-checked SQLite mutation batch.

The database therefore must not claim a byte offset that has not first reached the staging-file synchronization boundary.

If resume is unsafe or the server requires restart from the beginning:

1. truncate the staging file to zero;
2. synchronize the truncation;
3. persist a zero-byte Downloading checkpoint with stale validators/expected length cleared where applicable;
4. only then issue the replacement full request.

This ordering intentionally allows a crash to leave synchronized staging bytes ahead of the committed database checkpoint; recovery truncates that uncommitted tail. It does not allow the database checkpoint to be intentionally advanced ahead of synchronized staging bytes.

## Request and response safety

The runtime delegates full/restart/range planning to `download-http`.

A validator-bound resume:

- uses a strong ETag when available, otherwise Last-Modified;
- sends `Range: bytes=<offset>-` plus `If-Range`;
- appends only after a valid `206 Partial Content` begins at the exact persisted offset;
- requires a complete object length for the current bounded resume path;
- requires the returned range to reach the end of that object;
- checks Content-Length against the range length when Content-Length is present;
- rejects a changed expected object length;
- rejects conflicting returned validators.

A `200 OK` returned for a resume attempt is treated as a full replacement, never as append data. `412` and `416` trigger one bounded restart from zero through the durable reset ordering above. Other incompatible results fail closed rather than mixing representations.

## Final promotion and crash recovery

When body transfer reaches the known expected length:

1. persist state as Verifying;
2. synchronize and promote the staging artifact through `download-fs`;
3. persist state as Completed only after successful promotion.

If the process stops after promotion but before the Completed commit, a later runtime invocation sees Verifying/Processing plus the final artifact, revalidates its length, and can reconcile the durable state to Completed without downloading again.

An interrupted response body leaves only chunks that were synchronized and checkpointed as committed progress. Tests reopen the SQLite store after interruption and successfully continue with a validator-bound range request from the persisted offset.

A persisted Paused checkpoint can also be resumed through the runtime's explicit `resume` entry point. This does not yet implement an active in-flight pause signal or service-level pause/cancel lifecycle.

## Validation boundary

The runtime is compiled and tested as part of the native Rust workspace on Ubuntu and Windows.

Current tests include:

- scripted full download and validator-bound resume;
- forced restart when no safe validator exists;
- `412` restart from zero;
- body interruption with checkpoint never advancing beyond synchronized staging bytes;
- SQLite store close/reopen followed by validator-bound resume;
- persisted Paused-state resume;
- changed-validator rejection;
- incomplete resume-range rejection;
- crash-window recovery after final-file promotion;
- real loopback HTTP full request through `ReqwestTransport`;
- real loopback HTTP range request proving `Range`, `If-Range`, and identity encoding.

The current Android CI gate compiles the portable/shared-core crates and does **not** compile or run `download-runtime`; therefore no Android transport acceptance is claimed.

The source is configured for HTTPS through Rustls, but current integration tests use loopback HTTP. Representative live HTTPS/TLS server compatibility remains an explicit later gate.

## Security and privacy boundaries

This milestone intentionally does not enable:

- redirect following;
- system or configured proxies;
- cookie storage;
- HTTP authentication or bearer-token injection APIs;
- browser session import;
- credential storage;
- remote control;
- telemetry.

Source URLs remain sensitive durable state as defined by ADR-0003. Transport/runtime errors use bounded categories and must not embed the source URL.

Any future redirect, authentication, cookie, proxy, browser-session, or remote-control support must have explicit authority, sensitive-data, and destination-change rules before it is enabled.

## Alternatives considered

### Asynchronous reqwest runtime

Not selected for the first bounded transfer. The current goal is correctness of single-stream durability and restart ordering, not high concurrency. Adding an async executor and task lifecycle now would expand process/service behavior before the durable local path is stable.

### System curl or platform-native HTTP stacks

Not selected for the shared first native runtime. They would create different dependency, configuration, proxy, certificate, and behavior surfaces across Linux and Windows before cross-platform semantics are established.

### A smaller synchronous HTTP client

Not selected because reqwest provides a maintained HTTP client, Rustls integration, explicit redirect/proxy controls, and a path to later async/concurrent work without changing the domain and durability contracts.

## Consequences

The repository now contains real native network execution rather than only transport-independent contracts, and the third-party dependency graph is correspondingly larger. Exact versions must remain locked, reviewed, scanned, and updated deliberately.

The runtime can be exercised by tests and future service/CLI layers, but its presence does not establish a supported application build, Android runtime support, Internet-server compatibility matrix, live HTTPS acceptance, UI behavior, packaging, Platform-System integration, deployment, or production qualification.

## Revisit conditions

Revisit this decision when:

- active multi-transfer concurrency requires an async runtime or another execution model;
- HTTP/2 or HTTP/3 qualification requires transport changes;
- Android or another platform requires a different networking adapter;
- authorized proxy, redirect, authentication, cookie, or browser-session behavior is introduced;
- security, licensing, vulnerability, or maintenance evidence changes the dependency decision;
- representative performance or compatibility tests show the selected client cannot meet product requirements.
