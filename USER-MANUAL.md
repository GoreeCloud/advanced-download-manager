---
title: "GoreeCloud Advanced Download Manager — User Manual"
document_type: "User Manual"
version: "v0.2"
product_version: "0.1.0"
release_lifecycle: "Development"
status: "Current — Development Availability"
classification: "Public"
last_updated: "2026-09-15"
canonical_source: "repository"
central_sync_target: "GoreeCloud/User Manuals/User Manual — GoreeCloud Advanced Download Manager.md"
---

# GoreeCloud Advanced Download Manager — User Manual

## Current availability

GoreeCloud Advanced Download Manager is currently in **Development** lifecycle. The repository contains an initial shared Rust source foundation and a minimal `gcdm` command-line shell, but it does not yet provide a supported download workflow, desktop application, Android application, installer, package, daemon, or production-ready CLI.

There are therefore no supported installation, download, queue, scheduling, browser-integration, remote-control, or recovery procedures for ordinary users yet.

## Development shell

The current `gcdm` source shell exposes only development status/version behavior. It deliberately rejects unimplemented download commands. Its presence should not be treated as a supported end-user interface or release artifact.

## What is being developed

The planned product is a local-first download manager for Linux, Windows, and Android with a shared download engine. Planned capabilities include durable resume, intelligent acceleration, queues, bandwidth and scheduling controls, rules, verification, browser handoff, automation, and optional authorized GoreeCloud integrations. Planned capabilities are not current user instructions.

## Accounts and connectivity

Core downloading is intended to remain usable without a GoreeCloud account. GoreeCloud Identity, Mesh, Manager, remote control, and synchronization are planned optional layers, but no application-specific runtime integration is currently accepted.

## Privacy and security

The current source foundation includes redaction of sensitive source URLs from `Debug` output and bounds the initial core to HTTP/HTTPS source URLs. It does not yet perform network transfers or persist user download URLs, credentials, history, files, browser sessions, or transfer telemetry as an operational application.

These source-level boundaries do not establish Privacy Shield or Wardveil Security runtime acceptance. The planned runtime privacy/security model is described in `PRIVACY POLICY.md`, `SECURITY.md`, and `SPECIFICATIONS.md`.

## Getting help during Development

Use the repository documentation to understand current scope and verified implementation state. Do not rely on commands, package names, screenshots, or third-party builds that are not documented as supported by an authoritative GoreeCloud release.

This manual will be expanded when verified user-operable behavior exists. At that point it must describe only supported installation, configuration, workflows, limitations, recovery, and troubleshooting.
