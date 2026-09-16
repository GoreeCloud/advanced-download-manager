---
title: "GoreeCloud Advanced Download Manager — Features"
document_type: "Feature State"
version: "v0.3"
product_version: "0.1.0"
release_lifecycle: "Development"
status: "Active"
classification: "Public"
last_updated: "2026-09-15"
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
- a minimal `gcdm` Development-stage status/version shell that deliberately does not implement download commands.

This is source-foundation functionality only. Network transfer execution, persistent job storage, restart recovery, multipart downloading, browser integration, graphical clients, and Platform-System runtime integration are not implemented or accepted yet. The HTTP contract crate does not perform network I/O and does not establish a usable downloader.

## Planned feature families

The canonical requirements are maintained in `SPECIFICATIONS.md`; the delivery sequence is maintained in `FEATURE-ROADMAP.md`. Planned families include durable HTTP/HTTPS transfers, validator-safe resume, intelligent multipart acceleration, retries and mirrors, queues and scheduling, bandwidth/storage policy, rules and post-processing, checksum verification, browser acquisition workflows, non-DRM media-manifest downloads, secure authentication, Linux/Windows/Android clients, CLI/headless service operation, a shared local download API, privacy and security controls, and authorized cross-device GoreeCloud integration.

Torrent and magnet workflows remain planned as GoreeCloud Swarm delegation rather than an independent duplicate BitTorrent stack.

## Status rule

A planned capability moves into the implemented section only after authoritative source and the applicable tests, builds, integration evidence, and runtime validation support that claim. Source-level contracts are not represented as complete runtime features.
