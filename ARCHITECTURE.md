---
title: "GoreeCloud Advanced Download Manager — Architecture"
document_type: "Architecture"
version: "v0.3"
product_version: "0.1.0"
release_lifecycle: "Development"
status: "Active Development"
classification: "Public"
last_updated: "2026-09-15"
---

# GoreeCloud Advanced Download Manager — Architecture

## Status

The product is in **Development** with a bounded Rust shared-core source foundation. The component boundaries below remain the target architecture; only the Rust core-domain, HTTP resume-safety contract, and minimal CLI-shell portions are currently implemented.

## Initial technology decision

The shared Download Engine foundation uses Rust, with the repository toolchain pinned to Rust 1.98.1 and workspace minimum `rust-version` 1.98. The initial workspace separates portable domain logic, protocol-specific safety contracts, and the CLI shell:

- `crates/download-core` — shared core-domain contracts intended to remain portable across Linux, Windows, and Android integration layers;
- `crates/download-http` — dependency-free HTTP request/response safety contracts for full transfers, safe restart, validator-bound resume, and range-response disposition; it performs no network I/O;
- `crates/gcdm` — minimal Development-stage command-line shell for version/status visibility while transfer commands remain unavailable.

This decision establishes the common core language/runtime direction. It does not pre-decide the persistence backend, HTTP/HTTP3 client libraries, IPC mechanism, desktop UI framework, Android UI/binding layer, package formats, or credential-storage adapters. Those remain separately governed implementation choices.

## Component model

### GoreeCloud Download Engine

Owns protocol transfer execution, segmentation, resume semantics, retry behavior, integrity verification, queue scheduling, and bandwidth policy. The current Rust source implements bounded job/state/resume-validator contracts plus HTTP resume-safety planning and response disposition; actual protocol execution remains pending.

### GoreeCloud Download Service

Owns durable jobs and process-lifetime independence. It will expose controlled local interfaces to graphical clients, CLI, browser connectors, and approved GoreeCloud callers. A UI closing must not automatically destroy an enabled background transfer service. The durable service is not implemented yet.

### GoreeCloud Download Database

Stores jobs, queue/rule definitions, history where enabled, transfer checkpoints, validators, and processing state. Persistence must be transaction-safe and schema migrations recoverable. No persistence backend has been selected or implemented yet.

### Platform UI clients

Linux, Windows, and Android clients adapt platform-specific notifications, credential storage, filesystem/storage rules, lifecycle/background behavior, and Glaze UI presentation without forking core transfer semantics unnecessarily. No graphical client is implemented yet.

### Browser Connector

Provides a narrow authenticated bridge for browser handoff. Browser extensions must not receive unrestricted engine, filesystem, credential, or history authority. This connector remains planned.

### Mesh Connector

Provides explicit trusted-device discovery and cross-device control when enabled. Local downloading must not depend on Mesh. This connector remains planned.

### GoreeCloud Integration Layer

Connects applicable capabilities to GoreeCloud Manager, Privacy Shield, Wardveil Security, Everkeep, Glaze UI, GoreeCloud Mesh, and GoreeCloud Identity while preserving each system's authority boundary. GoreeCloud Sync remains separately governed. No application-specific Platform-System integration is accepted yet.

## HTTP resume-safety boundary

Before a real HTTP client is introduced, the source foundation defines transport-independent decisions that a later adapter must honor:

- a transfer with no persisted bytes uses a full request;
- a partial transfer may issue a range request only when a safe `If-Range` validator is available;
- strong ETag is preferred, with Last-Modified as fallback; a weak ETag without a usable Last-Modified validator forces full restart;
- a resumed response may append only when the server returns `206 Partial Content` and the parsed `Content-Range` begins exactly at the requested byte offset;
- `200 OK` in response to a resume attempt is treated as a full-body replacement, never as appendable data;
- `412 Precondition Failed` and `416 Range Not Satisfiable` require retry from the beginning;
- malformed, offset-mismatched, or otherwise unexpected partial responses are rejected rather than combined with existing data.

These are source-level safety contracts. They do not establish network execution, parser correctness, durable checkpoint storage, or crash-safe file handling.

## Boundary rules

- Authorization must travel with remote/API operations rather than relying only on ambient identity.
- URLs, credentials, referrers, cookies, and signed parameters are sensitive data.
- Swarm owns BitTorrent specialization; the Download Manager may delegate and present unified status.
- Browser interception, remote access, synchronization, and plugin/provider execution are opt-in or explicitly authorized surfaces, not prerequisites for the local engine.
- Temporary/partial files must not be presented as successful final downloads.
- Protocol and parser additions require explicit security, recovery, and compatibility behavior.
- Shared core code forbids Rust `unsafe` code unless a later separately governed exception is explicitly justified and reviewed.

## Near-term architecture work

The next architecture layer remains persistent local job state and single-stream HTTP/HTTPS execution. Before that implementation is treated as durable, the project must define transaction boundaries, partial-file naming/promotion, crash recovery, schema/version migration, validator persistence, retry/error contracts, and test fixtures for interrupted or changed remote objects. The current HTTP contract layer is intended to make later network execution fail closed around resume semantics rather than inventing those rules inside an adapter.
