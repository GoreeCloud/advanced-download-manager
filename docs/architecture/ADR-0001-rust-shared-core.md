---
title: "ADR-0001 — Rust Shared-Core Foundation"
document_type: "Architecture Decision Record"
version: "v0.1"
product_version: "0.1.0"
release_lifecycle: "Development"
status: "Accepted"
classification: "Public"
last_updated: "2026-09-15"
---

# ADR-0001 — Rust Shared-Core Foundation

## Decision

Use **Rust** for the portable GoreeCloud Advanced Download Manager shared core and transfer-engine foundation. Pin the current repository toolchain to Rust 1.98.1, use the Rust 2024 edition, and keep the portable core independent from any one graphical UI or operating-system shell.

The initial workspace is deliberately small:

- `crates/download-core` — portable job/state/resume contracts and future transfer-engine logic;
- `crates/gcdm` — minimal Development-stage command-line shell and future local client surface.

Shared core code forbids `unsafe` Rust by default. Any future exception would require a separate explicit architectural/security justification rather than being silently introduced.

## Context

The product is required to share durable transfer semantics across Linux, Windows, and Android while retaining platform-native integration for lifecycle, notifications, storage, credentials, and user experience. The common core therefore needs strong memory safety, predictable native deployment, low runtime overhead, portable library boundaries, deterministic testing, and the ability to expose narrow bindings or IPC contracts to platform-specific clients.

The initial foundation also needs to remain local-first and capable of becoming a headless service without forcing a browser, cloud account, or graphical framework into the core.

## Considered direction

Go was considered as a strong alternative for service-oriented networking and simple deployment. Rust was selected for the shared core because its ownership/type model and native library model align better with the intended cross-platform engine boundary, especially where core transfer state may later be embedded behind platform-specific adapters. This does not prohibit Go in an unrelated future GoreeCloud service where Go is independently the better fit.

## Consequences

The decision establishes the common core language/runtime direction but does not select the persistence backend, HTTP/HTTP3 libraries, desktop UI toolkit, Android UI framework, IPC transport, credential stores, package formats, or release tooling. Those remain separate decisions and must be chosen with their own compatibility, security, recovery, and maintenance evidence.

Android support is initially limited to verifying that the portable core can compile for the Android target. A target compilation check is not an Android application acceptance result.

## Revisit conditions

Revisit this ADR only if verified implementation evidence shows that the chosen Rust boundary materially prevents required Linux, Windows, Android, security, performance, recovery, or maintainability outcomes. Convenience alone is not sufficient reason to fragment the shared transfer model across multiple independent engines.
