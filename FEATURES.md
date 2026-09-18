---
title: "GoreeCloud Advanced Download Manager — Features"
document_type: "Feature State"
version: "v0.6"
product_version: "0.1.0"
release_lifecycle: "Development"
status: "Active"
classification: "Public"
last_updated: "2026-09-18"
---

# GoreeCloud Advanced Download Manager — Features

## Current verified source functionality

The repository contains an initial Rust source foundation. The implemented bounded core currently provides:

- validated job identifiers;
- HTTP/HTTPS-only source URL admission for the current core boundary;
- redacted `Debug` output for sensitive source URLs;
- explicit queued, downloading, paused, waiting, verifying, processing, completed, failed, and cancelled job states;
- bounded state transitions for start, pause, wait, resume, retry, verification, processing, completion, failure, and cancellation;
- monotonic progress checks and expected-size upper-bound checks;
- a requirement that known-length transfers be complete before verification;
- a requirement that a final file be safely promoted before a job can become completed;
- ETag/Last-Modified resume-decision logic that does not silently approve resume when validators are unavailable;
- dependency-free HTTP resume-safety contracts that choose full, restart-full, or validator-bound range requests from persisted progress state;
- strong-ETag preference with Last-Modified fallback for planned `If-Range` requests, while weak or unavailable validators force full restart rather than unsafe append;
- HTTP response-body disposition rules that only permit append after a valid `206 Partial Content` response begins at the requested byte offset, treat `200 OK` as a full-body replacement, and require restart or rejection for incompatible range responses;
- an explicit durable-state schema contract with current schema version `1`, upgrade-required handling for older state, and fail-closed rejection of unsupported future schema state;
- deterministic per-job partial-file paths that keep incomplete bytes separate from the final destination;
- versioned job checkpoint contracts carrying job identity, sensitive source URL, final/staging paths, state, progress, expected length, and remote validators while retaining sensitive-URL debug redaction;
- recovery dispositions that route active jobs through validator-safe HTTP full/restart/resume planning, re-enter verification or processing after interruption, and require completed-artifact revalidation after restart;
- partial-file reconciliation rules that accept matching durable length, truncate an uncommitted tail beyond the checkpoint, and require restart when the checkpoint is ahead of the actual partial artifact;
- generation-based mutation-batch contracts for atomic persistence adapters, including stale-writer detection boundaries and duplicate-job mutation rejection;
- a pinned `crates/download-store-sqlite` durable job-state adapter using bundled SQLite through `rusqlite 0.40.2`, with explicit schema/version validation, optimistic generation checking, atomic mutation batches, integrity checks, lossless native-path persistence, sensitive-URL handling, and reopen/round-trip tests;
- a `crates/download-fs` staging-file adapter that reconciles checkpoint and actual partial-file lengths, truncates uncommitted tails, restarts safely when persisted progress is ahead of the partial artifact, synchronizes append/truncate operations before reporting durable progress, refuses silent replacement of an existing final artifact, and promotes a synchronized staging file by rename with parent-directory synchronization on Unix;
- a minimal `gcdm` Development-stage status/version shell that deliberately does not implement download commands.

The same application repository also contains the application-owned Firefox client under `clients/firefox/`. Firefox client 0.2.12 separately retains accepted Stable evidence for Mozilla unlisted/self-distribution, browser-managed downloads, optional native segmented transfers, queueing/pause/resume/retry behavior, same-job native recovery, deterministic packaging, and the accepted Linux native helper 0.2.11 / protocol 2. That version-specific Firefox evidence does not establish completion or Stable status for the Rust application engine.

The Rust application foundation remains Development-stage functionality. Durable job metadata persistence and bounded staging-file durability primitives are implemented independently, but shared-engine network execution, database-to-filesystem checkpoint ordering and orchestration, end-to-end application-engine restart recovery, multipart downloading, browser-to-shared-engine integration, Linux/Windows/Android graphical clients, and Platform-System runtime integration are not implemented or accepted yet. The repository does contain a separately accepted Firefox 0.2.12 client release; that narrower platform release must not be generalized into a Stable application-core claim.

## Planned feature families

The canonical requirements are maintained in `SPECIFICATIONS.md`; the delivery sequence is maintained in `FEATURE-ROADMAP.md`. Planned families include durable HTTP/HTTPS transfers, validator-safe resume, intelligent multipart acceleration, retries and mirrors, queues and scheduling, bandwidth/storage policy, rules and post-processing, checksum verification, browser acquisition workflows, non-DRM media-manifest downloads, secure authentication, Linux/Windows/Android clients, CLI/headless service operation, a shared local download API, privacy and security controls, and authorized cross-device GoreeCloud integration.

Torrent and magnet workflows remain planned as GoreeCloud Swarm delegation rather than an independent duplicate BitTorrent stack.

## Status rule

A planned capability moves into the implemented section only after authoritative source and the applicable tests, builds, integration evidence, and runtime validation support that claim. Source-level contracts are not represented as complete runtime features.
