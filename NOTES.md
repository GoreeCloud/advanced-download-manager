---
title: "GoreeCloud Advanced Download Manager — Repository Notes"
document_type: "Repository Notes"
version: "v0.3"
product_version: "0.1.0"
release_lifecycle: "Development"
status: "Active"
classification: "Internal"
last_updated: "2026-09-18"
---

# GoreeCloud Advanced Download Manager — Repository Notes

## Verified current state

The repository is in **Development** with a bounded Rust shared-core source foundation and an application-owned Firefox client under `clients/firefox/`. `SPECIFICATIONS.md` and `FEATURE-ROADMAP.md` remain authoritative for planned scope and delivery sequencing. The current source includes portable core job/state contracts, HTTP resume-safety contracts, backend-neutral durable-state/recovery contracts, a minimal CLI shell, repository validation, and cross-platform CI definitions.

No completed shared Rust network transfer engine, Linux/Windows/Android graphical application client, supported standalone application release, production deployment, or representative application-core runtime acceptance is verified yet. Separately, the Firefox client retains accepted Stable 0.2.12 signing/restart/native-recovery evidence; that platform-specific evidence does not promote the application core.

Product internal version remains **0.1.0**. Development lifecycle reflects active source implementation and does not imply Release Candidate, Stable, or production readiness.

## Current decisions

- The common shared-core implementation direction is Rust, pinned to toolchain 1.98.1 for the current foundation.
- Core operation is local-first and must not depend on an account.
- `crates/download-core` owns portable core-domain contracts.
- `crates/download-http` owns transport-independent HTTP full/restart/resume safety decisions and performs no network I/O.
- `crates/download-state` owns backend-neutral schema, checkpoint, staging-path, atomic mutation-batch, and restart-recovery contracts and performs no persistence or filesystem I/O.
- `crates/gcdm` is currently only a narrow Development CLI shell.
- Durable single-stream HTTP/HTTPS downloading precedes multipart acceleration.
- Specialized BitTorrent/magnet work is delegated to GoreeCloud Swarm.
- Remote, synchronization, and broader ecosystem integrations remain later layers and must not become dependencies of the basic local transfer engine. Firefox is now an application-owned client source under `clients/firefox/`; convergence from its existing browser/native implementation to the future shared Download Service remains a separate architecture task.
- All seven Integral Platform Systems remain evaluated as applicable but blocked/unaccepted for this application.

## Open architecture work

The next bounded implementation decision is the actual transaction-safe persistence backend and serialization/migration adapter that will satisfy ADR-0002. Filesystem checkpoint ordering, synchronization, locking, corruption handling, and restart fixtures must be validated before runtime durability is claimed. Actual HTTP/HTTPS client technology and the first single-stream transport path remain pending after or alongside that durable-state adapter.

IPC/API technology, Linux/Windows/Android platform UI frameworks, Android bindings, shared-engine credential storage, application packaging/signing, broader dependency/security scanning, and production recovery mechanics remain open. Firefox packaging/signing is separately implemented for the version-specific client release.

## Repository hygiene

Merged documentation and implementation branches are cleanup candidates after their work is fully merged and verified. Branch existence is not implementation, release, or acceptance evidence. The current GitHub connector does not expose branch deletion, so branch cleanup remains an explicit follow-up rather than being represented as completed.

The live `main` branch is currently reported by GitHub as unprotected. GoreeCloud repository policy expects protection for the default authoritative branch; configuration remains a repository-governance follow-up because the currently available connector does not expose branch-protection mutation.
