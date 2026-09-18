---
title: "GoreeCloud Advanced Download Manager — Changelog"
document_type: "Changelog"
version: "v0.6"
product_version: "0.1.0"
release_lifecycle: "Development"
status: "Active"
classification: "Public"
last_updated: "2026-09-17"
---

# GoreeCloud Advanced Download Manager — Changelog

All notable governed repository changes should be recorded here. Source milestones do not by themselves establish a supported release.

## Unreleased — Development 0.1.0

### Source foundation

- Selected Rust as the shared-core implementation direction and pinned the current repository toolchain to Rust 1.98.1.
- Added the `crates/download-core` shared core-domain crate and `crates/gcdm` Development CLI shell.
- Added bounded job identifiers, sensitive URL handling/redaction, explicit transfer-state transitions, progress invariants, final-file-promotion gating, and remote-validator resume decisions.
- Added the dependency-free `crates/download-http` contract crate for safe HTTP full/restart/resume request planning and response-body disposition before network execution is introduced.
- Added strong-ETag/Last-Modified `If-Range` planning, weak-validator fallback behavior, exact resume-offset checks for `206 Partial Content`, full-body replacement behavior for `200 OK`, and explicit restart behavior for `412 Precondition Failed` / `416 Range Not Satisfiable`.
- Added the dependency-free `crates/download-state` contract crate for schema compatibility, deterministic per-job partial-file paths, versioned job checkpoints, crash-recovery dispositions, partial-file reconciliation, final-artifact revalidation, and generation-based atomic mutation-batch boundaries for a future persistence adapter.
- Added ADR-0002 to define durable-state transaction, migration/rollback, checkpoint ordering, staging/promotion, and restart-recovery expectations before a persistence backend is selected.
- Added `crates/download-store-sqlite`, the first transaction-safe durable job-state backend, using pinned `rusqlite 0.40.2` with bundled SQLite and bounded unsigned metadata conversion.
- Added explicit SQLite schema/version validation, generation-checked atomic mutation batches, integrity checking, lossless native-path persistence, sensitive-URL handling, reopen/round-trip tests, and ADR-0003 documenting the backend and durability policy.
- Extended Android CI with NDK C-toolchain configuration and compilation of the SQLite adapter for the Android shared-core target.
- Expanded Android shared-core CI coverage to compile the portable core, HTTP safety, and durable-state contract crates for the Android target.
- Added repository-policy validation and CI definitions for formatting, Clippy, tests, native workspace checking, and Android shared-core compilation checking.
- No network transfer execution, filesystem durability/runtime transfer orchestration, graphical client, installable release, Platform-System acceptance, or production qualification is included in this milestone.

### Documentation and governance

- Established the canonical planned `SPECIFICATIONS.md`.
- Established the canonical phased `FEATURE-ROADMAP.md` and synchronized GoreeCloud Drive roadmap representation.
- Established the repository governance/documentation baseline, product internal version 0.1.0, architecture boundary, current feature state, user manual, privacy/security guidance, branding, benefits, competitive objectives, notes, editor configuration, ignore rules, rights notice, and Platform Contract declaration.
- Advanced the software release lifecycle from Concept to Development only after source implementation began; planned feature requirements remain planned unless separately verified.
- Reconciled the roadmap and current-state documentation with verified Phase 1 source-contract progress while preserving the distinction between source contracts and runtime durability.
- Migrated the repository Platform Contract declaration and policy validator from the superseded seven-system model to all nine Integral Platform Systems by adding GoreeCloud Policy and GoreeCloud Observability as blocked/unaccepted integrations.
- Updated the required Stable Glaze UI consumer target from 1.4.1 to 1.5.1 while preserving the truthful absence of a runtime UI or application-specific Glaze acceptance.
- Reconciled README current-state wording with the verified SQLite durable-state implementation without implying network-transfer, filesystem-durability, restart-recovery, client, release, or production completion.
