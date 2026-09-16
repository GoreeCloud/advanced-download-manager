---
title: "GoreeCloud Advanced Download Manager — Changelog"
document_type: "Changelog"
version: "v0.2"
product_version: "0.1.0"
release_lifecycle: "Development"
status: "Active"
classification: "Public"
last_updated: "2026-09-15"
---

# GoreeCloud Advanced Download Manager — Changelog

All notable governed repository changes should be recorded here. Source milestones do not by themselves establish a supported release.

## Unreleased — Development 0.1.0

### Source foundation

- Selected Rust as the shared-core implementation direction and pinned the current repository toolchain to Rust 1.98.1.
- Added the `crates/download-core` shared core-domain crate and `crates/gcdm` Development CLI shell.
- Added bounded job identifiers, sensitive URL handling/redaction, explicit transfer-state transitions, progress invariants, final-file-promotion gating, and remote-validator resume decisions.
- Added repository-policy validation and CI definitions for formatting, Clippy, tests, native workspace checking, and Android shared-core compilation checking.
- No network transfer execution, durable persistence, graphical client, installable release, Platform-System acceptance, or production qualification is included in this milestone.

### Documentation and governance

- Established the canonical planned `SPECIFICATIONS.md`.
- Established the canonical phased `FEATURE-ROADMAP.md` and synchronized GoreeCloud Drive roadmap representation.
- Established the repository governance/documentation baseline, product internal version 0.1.0, architecture boundary, current feature state, user manual, privacy/security guidance, branding, benefits, competitive objectives, notes, editor configuration, ignore rules, rights notice, and Platform Contract declaration.
- Advanced the software release lifecycle from Concept to Development only after source implementation began; planned feature requirements remain planned unless separately verified.
