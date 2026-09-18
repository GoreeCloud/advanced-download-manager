---
title: "GoreeCloud Advanced Download Manager — Architecture"
document_type: "Architecture"
version: "v0.6"
product_version: "0.1.0"
release_lifecycle: "Development"
status: "Active Development"
classification: "Public"
last_updated: "2026-09-17"
---

# GoreeCloud Advanced Download Manager — Architecture

## Status

The product is in **Development** with a bounded Rust source foundation. The component boundaries below remain the target architecture; the Rust core-domain contracts, HTTP resume-safety contracts, backend-neutral durable-state/recovery contracts, SQLite durable job-state adapter, staging-file durability adapter, native single-stream runtime/orchestrator, and minimal CLI shell are currently implemented.

## Initial technology decision

The shared Download Engine foundation uses Rust, with the repository toolchain pinned to Rust 1.98.1 and workspace minimum `rust-version` 1.98. The current workspace separates portable domain logic, protocol-specific safety contracts, durable-state/recovery contracts, durable metadata persistence, bounded staging-file operations, native transport/orchestration, and the CLI shell:

- `crates/download-core` — shared core-domain contracts intended to remain portable across Linux, Windows, and Android integration layers;
- `crates/download-http` — dependency-free HTTP request/response safety contracts for full transfers, safe restart, validator-bound resume, and range-response disposition; it performs no network I/O;
- `crates/download-state` — dependency-free durable-state and recovery contracts for schema compatibility, checkpoints, staging paths, atomic mutation-batch generations, crash recovery, and final-artifact revalidation; it performs no persistence or filesystem I/O;
- `crates/download-store-sqlite` — transaction-safe SQLite durable job-state adapter with schema/version validation, generation-checked atomic mutation batches, integrity checks, and lossless native-path storage;
- `crates/download-fs` — bounded staging-file adapter that applies checkpoint-length reconciliation, synchronized append/truncate operations, safe restart-on-short-partial behavior, and non-overwriting final-file promotion; it performs no network I/O;
- `crates/download-runtime` — native single-stream HTTP/HTTPS transport/orchestration layer using pinned reqwest 0.13.5 with Rustls; it applies the HTTP safety contracts, synchronizes staging bytes before advancing SQLite checkpoints, and performs bounded recovery/restart orchestration;
- `crates/gcdm` — minimal Development-stage command-line shell for version/status visibility while transfer commands remain unavailable.

This establishes the common core language/runtime direction, SQLite as the initial durable metadata backend, and reqwest 0.13.5 with Rustls as the first native single-stream HTTP/HTTPS transport. HTTP/3, broader protocol stacks, IPC mechanism, desktop UI framework, Android transport/binding strategy, package formats, credential-storage adapters, and production migration/recovery mechanisms remain separately governed implementation choices.

## Component model

### GoreeCloud Download Engine

Owns protocol transfer execution, segmentation, resume semantics, retry behavior, integrity verification, queue scheduling, and bandwidth policy. The current Rust source implements bounded job/state/resume-validator contracts, HTTP resume-safety planning, durable metadata persistence, staging-file operations, and the first native single-stream HTTP/HTTPS execution path. Multipart execution, generalized retry policy, additional protocols, and user-facing service control remain pending.

### GoreeCloud Download Service

Owns durable jobs and process-lifetime independence. It will expose controlled local interfaces to graphical clients, CLI, browser connectors, and approved GoreeCloud callers. A UI closing must not automatically destroy an enabled background transfer service. The durable service is not implemented yet.

### GoreeCloud Download Database

Stores jobs, queue/rule definitions, history where enabled, transfer checkpoints, validators, and processing state. Persistence must be transaction-safe and schema migrations recoverable. The repository now defines backend-neutral schema, generation, checkpoint, mutation-batch, crash-recovery, and partial-artifact contracts in `crates/download-state` and implements the initial transaction-safe SQLite adapter in `crates/download-store-sqlite`. End-to-end service wiring, production migration/rollback execution, broader locking/corruption qualification, and restart recovery remain separate evidence gates.

### Platform UI clients

Linux, Windows, and Android clients adapt platform-specific notifications, credential storage, filesystem/storage rules, lifecycle/background behavior, and Glaze UI presentation without forking core transfer semantics unnecessarily. No graphical client is implemented yet.

### Browser Connector

Provides a narrow authenticated bridge for browser handoff. Browser extensions must not receive unrestricted engine, filesystem, credential, or history authority. This connector remains planned.

### Mesh Connector

Provides explicit trusted-device discovery and cross-device control when enabled. Local downloading must not depend on Mesh. This connector remains planned.

### GoreeCloud Integration Layer

Connects applicable capabilities to GoreeCloud Manager, Privacy Shield, Wardveil Security, Everkeep, Glaze UI, GoreeCloud Mesh, GoreeCloud Identity, GoreeCloud Policy, and GoreeCloud Observability while preserving each system's authority boundary. GoreeCloud Sync remains separately governed and is not a tenth Integral Platform System. No application-specific Platform-System integration is accepted yet.

## HTTP resume-safety boundary

The transport-independent HTTP safety contracts are consumed by the native runtime and continue to govern every request/response transition:

- a transfer with no persisted bytes uses a full request;
- a partial transfer may issue a range request only when a safe `If-Range` validator is available;
- strong ETag is preferred, with Last-Modified as fallback; a weak ETag without a usable Last-Modified validator forces full restart;
- a resumed response may append only when the server returns `206 Partial Content` and the parsed `Content-Range` begins exactly at the requested byte offset;
- `200 OK` in response to a resume attempt is treated as a full-body replacement, never as appendable data;
- `412 Precondition Failed` and `416 Range Not Satisfiable` require retry from the beginning;
- malformed, offset-mismatched, or otherwise unexpected partial responses are rejected rather than combined with existing data.

The native runtime now applies these contracts to real reqwest responses and uses the shared `Content-Range` parser. Loopback tests validate real HTTP full and validator-bound range requests on Ubuntu and Windows; this does not establish representative Internet-server compatibility or live HTTPS/TLS acceptance.

## Native single-stream runtime boundary

`crates/download-runtime` is the first concrete native execution/orchestration layer. ADR-0004 records the transport choice and initial security boundaries.

The current bounded runtime:

- uses pinned reqwest `0.13.5` with default features disabled and only the blocking and Rustls feature set enabled;
- disables automatic redirects rather than forwarding sensitive source URLs or credentials to an unapproved destination;
- disables reqwest's automatic system-proxy discovery, so ambient `HTTP_PROXY`/`HTTPS_PROXY` state cannot silently change this milestone's network authority;
- requests `Accept-Encoding: identity` so persisted byte offsets continue to refer to the representation used for range requests;
- derives full/restart/range requests from `download-http` and validates status, `Content-Range`, expected length, and persisted validators before appending;
- synchronizes each staging chunk before committing the corresponding downloaded-byte checkpoint to SQLite;
- truncates/synchronizes the staging file and commits a zero-byte durable checkpoint before a forced full restart;
- commits a Verifying checkpoint before final-file promotion and a Completed checkpoint only after promotion succeeds;
- can reopen SQLite after an interrupted response body, reconcile the staging artifact, and continue with a validator-bound range request;
- can resume a persisted Paused checkpoint through its explicit `resume` entry point;
- can reconcile the crash window where the final artifact was already promoted while durable state still reports Verifying/Processing.

The runtime is currently a library boundary, not a supported service or user-facing client. Active in-flight pause/cancel signaling, configured redirects, authentication/cookies, proxy support, generalized retry/backoff, storage-failure injection, Android transport qualification, and representative live HTTPS/TLS tests remain open.

## Durable state and recovery boundary

`crates/download-state` establishes recovery semantics consumed by the durable-state and staging-file adapters:

- durable schema version `1` is explicit; older versions require an explicit migration and unsupported future versions fail closed;
- job mutations are modeled as all-or-nothing batches with an expected store generation and a one-step generation advance, giving a future backend an optimistic-concurrency boundary;
- incomplete bytes use a deterministic sibling staging path `<final-file-name>.gcdm-part.<job-id>` rather than the final destination;
- persisted checkpoints include the sensitive source URL, job state, progress, expected length, validators, final path, and deterministic staging path while inheriting redacted debug handling from `SensitiveUrl`;
- restart recovery compares actual staging length to the committed checkpoint: matching lengths are consistent, uncommitted tails are truncated to the checkpoint, and a staging file shorter than the committed checkpoint requires safe restart rather than skipping a missing byte range;
- queued/downloading/paused/waiting jobs recover through the HTTP full/restart/resume planner; verifying and processing jobs re-enter their respective phases;
- completed jobs require final-artifact presence and known-length consistency before completed state is trusted after restart.

ADR-0002 defines the corresponding migration and rollback expectations. `crates/download-state` itself does not write a database or mutate files; `crates/download-store-sqlite`, `crates/download-fs`, and `crates/download-runtime` now implement and test the bounded metadata/filesystem ordering path, including restart after SQLite reopen and promotion-window recovery. Power-loss qualification, corruption/locking behavior, storage-failure injection, and service-lifecycle recovery remain separate evidence gates.

## Boundary rules

- Authorization must travel with remote/API operations rather than relying only on ambient identity.
- URLs, credentials, referrers, cookies, and signed parameters are sensitive data.
- Swarm owns BitTorrent specialization; the Download Manager may delegate and present unified status.
- Browser interception, remote access, synchronization, and plugin/provider execution are opt-in or explicitly authorized surfaces, not prerequisites for the local engine.
- Temporary/partial files must not be presented as successful final downloads.
- Protocol and parser additions require explicit security, recovery, and compatibility behavior.
- Shared core code forbids Rust `unsafe` code unless a later separately governed exception is explicitly justified and reviewed.

## Near-term architecture work

The next architecture layer is to harden the bounded runtime around storage failures, missing/conflicting artifacts, database locking/corruption, active pause/cancel lifecycle, and representative live HTTPS/TLS behavior, then expose the stable engine through the first service/CLI control boundary. Redirect, authentication/cookie, and proxy policy must be explicitly designed before those capabilities are enabled rather than inheriting ambient client behavior.
