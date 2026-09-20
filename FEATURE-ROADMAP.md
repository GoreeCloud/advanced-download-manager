---
title: "GoreeCloud Advanced Download Manager — Feature Roadmap"
product: "GoreeCloud Advanced Download Manager"
document_type: "Feature Roadmap"
status: "Active"
version: "v0.6"
classification: "Public"
last_updated: "2026-09-17"
authoritative_record: true
repository: "GoreeCloud/advanced-download-manager"
canonical_source: "repository"
drive_sync_target: "GoreeCloud/Feature Roadmap/GoreeCloud Advanced Download Manager/FEATURE-ROADMAP.md"
---

# GoreeCloud Advanced Download Manager — Feature Roadmap

> **Canonical source:** This repository `FEATURE-ROADMAP.md` is the canonical editable roadmap. The corresponding GoreeCloud Drive Markdown record is a synchronized ecosystem-wide representation and must remain materially consistent with this file.
>
> **Implementation truth:** Unless a roadmap item is explicitly backed by verified source, test, build, integration, release, or runtime evidence, it remains **Planned**. The existence of `SPECIFICATIONS.md`, this roadmap, a branch, pull request, task record, prototype, or design does not establish implementation or release readiness.

## Product direction

GoreeCloud Advanced Download Manager is being developed as the universal download and transfer-orchestration layer for the GoreeCloud ecosystem. It is intended to operate as a powerful standalone application on Linux, Windows, and Android while also exposing a shared download service to authorized GoreeCloud applications.

The roadmap prioritizes a durable local-first transfer engine first, then policy, automation, user experience, platform integration, and cross-device orchestration. Specialized protocols such as BitTorrent remain delegated to the appropriate GoreeCloud service rather than being duplicated unnecessarily.

## Current verified state

Status: **Development — Phase 1 source foundation in progress**

The repository baseline entering this native-runtime milestone is PR #12 merge `bcd493029a2e1e11840465f7f2fe6f293e1e716a`. PR #12 exact head `88174e226ab36e50fd4ef3a201705b2ef9870692` passed pull-request Core Foundation run `35307559329`, was squash-merged into authoritative `main` as `bcd493029a2e1e11840465f7f2fe6f293e1e716a`, and post-merge Core Foundation run `35307637107` completed successfully for that exact merge revision across repository policy, Rust Ubuntu, Rust Windows, and Android shared-core validation.

Verified repository state currently establishes:

- product internal version `0.1.0` and Development lifecycle documentation;
- the governed repository documentation baseline, architecture boundaries, changelog, current feature state, user/privacy/security documentation, and Platform Contract declaration;
- the current Platform Contract representation evaluates exactly nine Integral Platform Systems — GoreeCloud Manager, Privacy Shield, Wardveil Security, Everkeep, Glaze UI, GoreeCloud Mesh, GoreeCloud Identity, GoreeCloud Policy, and GoreeCloud Observability — and requires Stable Glaze UI `1.5.1`; every application-specific Platform-System integration remains blocked/unaccepted pending implementation and evidence;
- `crates/download-core` portable job identity, lifecycle, progress, sensitive-URL redaction, validator, and final-promotion source contracts;
- `crates/download-http` dependency-free HTTP full/restart/resume planning and response-disposition contracts, including safe `If-Range` selection and range-offset validation;
- `crates/download-state` backend-neutral schema compatibility, deterministic staging-path, versioned checkpoint, generation-batch, crash-recovery disposition, partial-artifact reconciliation, and completed-artifact revalidation contracts;
- `crates/download-store-sqlite` transaction-safe durable job-state persistence with pinned bundled SQLite/rusqlite, explicit schema/version validation, generation-checked atomic mutation batches, integrity checks, lossless native-path persistence, sensitive-URL handling, and reopen/round-trip tests;
- `crates/download-fs` bounded staging-file durability primitives for checkpoint-length reconciliation, truncation of uncommitted tails, restart-on-short-partial recovery, synchronized append/truncate operations, refusal to overwrite an existing final artifact, and rename-based final promotion with Unix parent-directory synchronization;
- `crates/download-runtime` native single-stream HTTP/HTTPS execution/orchestration with pinned reqwest 0.13.5 + Rustls, fail-closed redirect/proxy defaults, identity transfer encoding, response/range/validator validation, filesystem-before-database checkpoint ordering, and bounded recovery across SQLite reopen and final-promotion crash windows;
- native Ubuntu/Windows tests covering real loopback HTTP full/resume requests, network-body interruption, persisted Paused-state resume, validator changes, and restart behavior;
- the minimal `gcdm` Development-stage shell;
- repository-policy, Rust Ubuntu/Windows, and Android portable-core CI coverage for the current bounded source foundation.

Bounded native single-stream execution and durability/reopen recovery are now verified on Ubuntu and Windows source CI, but no supported service or graphical client exposes the engine. Representative live HTTPS/TLS behavior, Android transport, active in-flight pause/cancel, redirect/authentication/proxy policy, storage-failure/corruption/locking hardening, installable release artifacts, production deployment, and Platform-System runtime acceptance remain unverified. All nine Integral Platform Systems remain unaccepted for this application, and the Glaze UI `1.5.1` requirement is a conformance target rather than evidence of a runtime UI or application-specific acceptance.

Phase 0 therefore remains open for outstanding governance/toolchain decisions and CI/release foundations, while Phase 1 is now actively in progress rather than merely planned.

# Delivery sequence

## Phase 0 — Governance, repository, and architecture baseline

**Roadmap ID:** ADM-000  
**Status:** In progress

Establish the project as a governed GoreeCloud application before implementation expands.

Planned outcomes:

- complete the mandatory repository documentation baseline;
- define the product-internal version and lifecycle model when implementation begins;
- establish `goreecloud.platform.yaml` with truthful integration states;
- document the architecture boundary between the Download Engine, Download Service, Download Database, UI clients, Browser Connector, Mesh Connector, and Integration Layer;
- define secure configuration, credential-storage, logging, privacy, and recovery boundaries;
- establish CI, test, packaging, dependency, and release-validation foundations appropriate to each target platform;
- keep `SPECIFICATIONS.md`, this roadmap, Tasks Management, user documentation, and future changelog records synchronized with verified reality.

Verified progress includes the repository documentation baseline, product version/lifecycle model, the current nine-system Platform Contract declaration with Stable Glaze UI `1.5.1` as the required consumer target, Rust shared-core decision in ADR-0001, durable-state/recovery rules in ADR-0002, the SQLite durable-store decision in ADR-0003, the initial native reqwest/Rustls transport decision in ADR-0004, architecture boundaries, and baseline cross-platform CI. Remaining Phase 0 work includes unresolved UI/IPC/Android transport implementation-stack decisions, licensing posture, broader dependency/security and release validation, and repository-governance gaps such as default-branch protection where provider capabilities permit.

**Exit criteria:** repository governance baseline exists, architecture boundaries are documented, initial implementation plan is testable, and no planned integration is represented as accepted without evidence.

## Phase 1 — Durable local download engine foundation

**Roadmap ID:** ADM-100  
**Status:** In progress

Build the common platform-independent engine and persistent job model.

Scope includes specification sections 1, 3, 4, 43, 44, 64, 65, and 69.

Planned outcomes:

- HTTP and HTTPS transfer foundation;
- persistent download-job state;
- pause/resume across process restarts and system reboots where platform capabilities permit;
- resilient handling of network interruption and network transitions;
- resumable partial files with validator checks such as ETag and Last-Modified;
- transaction-safe download database;
- explicit transfer states such as queued, waiting, downloading, verifying, processing, completed, and failed;
- structured diagnostics and error categories;
- local-first operation without mandatory GoreeCloud Identity or cloud dependencies.

Verified source progress:

- `download-core` establishes bounded job identity/state/progress contracts, sensitive URL debug redaction, remote-validator decisions, and final-file-promotion gating;
- `download-http` establishes fail-closed full/restart/resume request planning and response-body disposition, including strong-ETag/Last-Modified `If-Range` handling, exact `206` offset checks, full replacement on `200`, and restart behavior for `412`/`416`;
- `download-state` establishes schema version `1`, deterministic per-job staging paths, versioned checkpoints, optimistic generation-based mutation batches, recovery dispositions, partial-file length reconciliation, and completed-artifact revalidation;
- `download-store-sqlite` implements the first durable local metadata backend with atomic generation-checked batches, fail-closed schema validation, integrity checks, native-path preservation, sensitive-URL handling, and reopen/round-trip coverage;
- `download-fs` implements bounded staging-file operations with checkpoint-length reconciliation, synchronized append/truncate semantics, safe restart when the checkpoint is ahead of the partial artifact, non-overwriting final promotion, and filesystem-focused unit coverage;
- `download-runtime` implements native single-stream HTTP/HTTPS request execution and durability orchestration, including filesystem-before-checkpoint ordering, full-restart reset ordering, response/range/validator checks, real loopback HTTP full/resume coverage, SQLite-reopen recovery after interruption, persisted Paused-state resume, and promotion-window reconciliation;
- ADR-0002 defines transaction/checkpoint/recovery rules, ADR-0003 records the SQLite backend, and ADR-0004 records the initial native reqwest/Rustls transport and fail-closed redirect/proxy boundary;
- the exact candidate source revision is covered by successful repository-policy, Rust Ubuntu, Rust Windows, and Android shared-core CI; Android runtime transport itself is not part of that target gate.

The transaction-safe job store, staging-file durability primitive, native single-stream transport, filesystem-before-SQLite checkpoint ordering, validator-safe resume, and bounded recovery across SQLite reopen are now implemented and tested together on Ubuntu and Windows. Phase 1 remains in progress because active in-flight pause/cancel lifecycle, storage-failure/corruption/locking scenarios, representative live HTTPS/TLS behavior, network-transition handling, Android runtime transport, and a stable service/CLI control boundary remain open.

**Exit criteria:** a single-stream download can be created, persisted, interrupted, resumed safely, validated against remote-object identity, and recovered after an unexpected application stop without corrupting the target file.

## Phase 2 — Acceleration, retry, mirrors, and integrity

**Roadmap ID:** ADM-200  
**Status:** Planned

Add controlled performance acceleration without sacrificing correctness or server friendliness.

Scope includes specification sections 2, 4, 13, 14, and 24.

Planned outcomes:

- byte-range capability discovery;
- configurable multipart downloading;
- dynamic segment sizing and rebalancing;
- adaptive connection counts based on server behavior, latency, bandwidth, device resources, battery state, and user policy;
- per-host connection limits and clean fallback to single-stream transfers;
- mirror validation, selection, failover, and compatible multi-mirror segmentation;
- exponential-backoff retry policies with reason-specific behavior;
- duplicate detection using URL, metadata, and checksums;
- SHA-256, SHA-512, and BLAKE3 verification, with SHA-1 and MD5 restricted to compatibility/legacy verification roles.

**Exit criteria:** segmented transfers are provably reassemblable, remote changes cannot silently corrupt resumed downloads, retry behavior is bounded and observable, and checksum outcomes are explicit.

## Phase 3 — Queues, priorities, bandwidth, scheduling, and storage policy

**Roadmap ID:** ADM-300  
**Status:** Planned

Turn the engine into a controllable download scheduler.

Scope includes specification sections 5-9, 40, 41, and 43.

Planned outcomes:

- multiple independent queues;
- manual ordering and Critical/High/Normal/Low/Background priority classes;
- queue, job, host, network, and global bandwidth controls;
- adaptive bandwidth mode that yields to latency-sensitive foreground activity where technically feasible and privacy-respecting;
- schedules by time, day, power state, network class, metering, VPN state, and supported thermal constraints;
- storage preflight checks, reserve thresholds, temporary-space forecasting, and multiple storage targets;
- rule-based routing to suitable storage devices.

**Exit criteria:** download scheduling and bandwidth behavior are deterministic, user-visible, policy-controlled, and safe under storage pressure or changing network conditions.

## Phase 4 — Rules, organization, naming, and post-processing

**Roadmap ID:** ADM-400  
**Status:** Planned

Automate repetitive download handling while keeping actions inspectable and reversible where practical.

Scope includes specification sections 10-12, 37-39, 63, and 67.

Planned outcomes:

- filter-style Smart Download Rules;
- categories, tags, custom destinations, and naming templates;
- duplicate-name conflict policy;
- File Manager integration;
- safe archive extraction with traversal and malicious-structure defenses;
- post-download workflows for verification, scanning, extraction, moving, renaming, tagging, importing, synchronization, and notifications;
- import/export for settings, queues, rules, checksums, and URL lists;
- controlled provider/plugin architecture with permission-scoped APIs.

**Exit criteria:** automation decisions are explainable, rule evaluation is testable, unsafe archive behavior is blocked, and plugins cannot obtain unrestricted application authority by default.

## Phase 5 — Browser, clipboard, drag-and-drop, batch, and Link Grabber

**Roadmap ID:** ADM-500  
**Status:** Planned

Provide user-friendly acquisition paths from browsers and desktop/mobile workflows.

Scope includes specification sections 16-21 and 58.

Planned outcomes:

- browser-extension protocol and secure local connector;
- Firefox, Chromium, Chrome, Edge, Brave, and GoreeCloud Browser integration targets;
- configurable interception and "Download with GoreeCloud" actions;
- GoreeCloud Browser native handoff with queue, history, and Webspace/container context where supported;
- optional privacy-conscious clipboard monitoring;
- drag-and-drop URL and list ingestion;
- batch URL generation/import;
- Link Grabber discovery and filtering;
- Android Share Sheet submission.

**Exit criteria:** browser and application handoff cannot silently broaden permissions, interception remains user-controlled, and source/referrer/session data is handled as sensitive metadata.

## Phase 6 — Authentication, media manifests, and protocol expansion

**Roadmap ID:** ADM-600  
**Status:** Planned

Expand protocol coverage only after the durable core is verified.

Scope includes specification sections 1, 22-24, and 36.

Planned outcomes:

- HTTP/2 and HTTP/3/QUIC where supported by the chosen network stack;
- FTP, FTPS, SFTP, and WebDAV support where justified and securely maintainable;
- object-storage and signed/temporary URL handling;
- secure username/password, HTTP auth, token, cookie, browser-session, and supported OAuth handoff;
- platform-native secure credential storage;
- HLS and MPEG-DASH parsing for downloadable non-DRM media;
- stream/resolution/codec/subtitle selection and safe segment assembly;
- `.torrent` and magnet delegation to GoreeCloud Swarm rather than duplicating the Swarm BitTorrent stack.

**Exit criteria:** each supported protocol has explicit security, authentication, resume, error, and compatibility behavior; DRM bypass is outside product scope.

## Phase 7 — Glaze UI application experience

**Roadmap ID:** ADM-700  
**Status:** Planned

Build the primary cross-platform experience using the currently approved Stable Glaze UI contract at implementation time.

Scope includes specification sections 45-52, 68, and 70.

Planned outcomes:

- Overview, Downloads, Queues, Link Grabber, Scheduler, Devices, History, Automation, and Settings navigation;
- download cards, compact mode, expanded dashboard, information panel, and segment diagnostics;
- real-time speed, connection, disk, and network graphs without overwhelming primary workflows;
- explicit state text and symbols in addition to color;
- reduced transparency, reduced motion, high contrast, keyboard and touch navigation, scalable typography, screen-reader semantics, and accessible progress reporting;
- responsive layouts for phone, tablet, and desktop form factors.

**Exit criteria:** representative workflows meet current Glaze UI and accessibility requirements on each implemented platform and advanced diagnostics remain discoverable without degrading basic usability.

## Phase 8 — Linux application, service, CLI, and headless operation

**Roadmap ID:** ADM-800  
**Status:** Planned

Make Linux a first-class target and establish the shared-service operating model.

Scope includes specification sections 53-55, 61, and 62.

Planned outcomes:

- desktop application integration for major environments;
- Wayland-first behavior with X11 compatibility where required;
- Secret Service credential integration;
- XDG directories and desktop notifications;
- optional systemd user-service integration;
- tray/background operation where supported;
- headless daemon for servers, NAS systems, mini PCs, and remote workstations;
- CLI and local automation interface;
- packaging strategy covering supported native/repository formats and portable distribution only when maintainable.

**Exit criteria:** GUI and headless clients can control the same durable service contract, service shutdown/restart preserves job state, and local privilege boundaries are documented and tested.

## Phase 9 — Windows integration

**Roadmap ID:** ADM-900  
**Status:** Planned

Adapt the common engine and service model to Windows without forking core behavior unnecessarily.

Scope includes specification sections 53, 56, 61, and 62.

Planned outcomes:

- Windows notifications, taskbar progress, startup behavior, and tray integration;
- File Explorer and protocol-handler integration where appropriate;
- Windows credential storage;
- Windows Defender scanning integration where supported;
- browser connector and CLI parity with the shared engine;
- Windows-specific installer, update, signing, and recovery validation.

**Exit criteria:** core download semantics remain consistent with Linux while Windows-specific integration follows platform security and packaging expectations.

## Phase 10 — Android integration and background execution

**Roadmap ID:** ADM-1000  
**Status:** Planned

Implement Android as a native client of the common download model while respecting current Android storage, background, power, and privacy restrictions.

Scope includes specification sections 57-60.

Planned outcomes:

- Share-to-GoreeCloud and supported browser handoff;
- foreground service behavior when required for active transfers;
- actionable persistent download notifications;
- Wi-Fi, cellular, roaming, charging, and battery-aware policies;
- Storage Access Framework and supported SD-card destinations;
- Android secure credential storage;
- optional Quick Settings controls;
- restart/reboot recovery consistent with Android platform constraints.

**Exit criteria:** active and deferred downloads behave correctly under process death, background restrictions, storage permissions, network transitions, and reboot scenarios on representative Android devices/versions.

## Phase 11 — Shared Download Service, API, and GoreeCloud SDK integration

**Roadmap ID:** ADM-1100  
**Status:** Planned

Promote the engine from an application-internal component into reusable GoreeCloud infrastructure only after its local contract is stable.

Scope includes specification sections 33-35 and 62.

Planned outcomes:

- authenticated local management API;
- stable download/queue lifecycle API;
- local IPC where preferable to network exposure;
- GoreeCloud SDK bindings;
- shared-service submission for approved GoreeCloud applications;
- explicit caller identity, purpose, permissions, destination, and post-processing authority;
- backward-compatible API versioning and clear error contracts.

**Exit criteria:** authorized applications can submit and control downloads without receiving unnecessary access to unrelated jobs, credentials, history, or filesystem locations.

## Phase 12 — Privacy Mode, Privacy Shield, and Wardveil Security

**Roadmap ID:** ADM-1200  
**Status:** Planned

Treat download URLs, credentials, referrers, browsing context, and history as sensitive information throughout the lifecycle.

Scope includes specification sections 15 and 25-28, plus the privacy principles in section 66.

Planned outcomes:

- user-controlled history retention;
- visible Privacy Mode with minimized persistence;
- removal or suppression of unnecessary URL/referrer/thumbnails/analytics metadata;
- Privacy Shield policy integration for approved networking, tracker/redirect handling, parameter minimization, DNS/privacy policy, insecure HTTP warnings, and fail-closed protected-network requirements where configured;
- Wardveil reputation, domain, file-type, quarantine, enterprise policy, and post-download security hooks;
- configurable platform malware scanners;
- no silent execution of downloaded content;
- auditable explanation of why a download is blocked, warned, delayed, or quarantined.

**Exit criteria:** privacy/security policy is enforced at runtime, sensitive URLs are not unnecessarily exposed in logs or sync, and protection state is specific to the actual accepted integration rather than implied globally.

## Phase 13 — Identity, Mesh, Manager, Everkeep, File Manager, and Swarm ecosystem workflows

**Roadmap ID:** ADM-1300  
**Status:** Planned

Add cross-device and administration capabilities without making them mandatory for local downloading.

Scope includes specification sections 29-32, 36-37, and 42.

Planned outcomes:

- optional GoreeCloud Identity synchronization for user-owned configuration;
- GoreeCloud Mesh trusted-device discovery and remote submission;
- cross-device progress, pause/resume, destination, and storage awareness;
- GoreeCloud Manager inventory, policy, bandwidth, storage, version, and security-event administration;
- Everkeep protection workflows for selected completed downloads;
- File Manager actions and destination integration;
- unified progress presentation for transfers delegated to GoreeCloud Swarm;
- explicit device authorization and revocation for every remote-control relationship.

**Exit criteria:** remote actions are authenticated, scoped, revocable, auditable, and cannot silently convert local-only use into synchronized or remotely controllable behavior.

## Phase 14 — Recovery, observability, compatibility, and performance qualification

**Roadmap ID:** ADM-1400  
**Status:** Planned

Harden the product for long-lived real-world use.

Scope includes specification sections 43-48, 63-68, and cross-cutting requirements from sections 2-9.

Planned outcomes:

- crash/restart/reboot recovery testing;
- database corruption and interrupted-write safeguards;
- low-storage, unplugged-drive, network-change, expired-URL, authentication, VPN, and server-mutation scenarios;
- transfer diagnostics, logs, graphs, and segment inspection with privacy-safe redaction;
- notification and Quiet Mode behavior;
- import/export and migration validation;
- accessibility acceptance;
- large-file, high-count queue, high-latency, bandwidth-limited, and long-duration performance tests;
- representative server-compatibility matrix for range requests, redirects, validators, throttling, authentication, and HTTP versions.

**Exit criteria:** recovery and performance claims are backed by repeatable evidence rather than happy-path tests alone.

## Phase 15 — Cross-platform release qualification

**Roadmap ID:** ADM-1500  
**Status:** Planned

Qualify actual releases only after the implemented feature set satisfies the applicable gates.

Planned outcomes:

- exact-revision source and build provenance;
- platform-specific packaging and signing;
- malware/security/privacy review;
- current Glaze UI acceptance;
- accessibility acceptance;
- upgrade, rollback, and state-migration validation;
- Linux, Windows, and Android representative-device qualification for the declared support matrix;
- verified platform-system integration states;
- user manual, specifications, roadmap, features, privacy, security, changelog, and task reconciliation;
- release artifacts traceable to immutable source identities.

**Exit criteria:** a release may be promoted only when the declared lifecycle state is supported by source, build, test, integration, packaging, release, and representative runtime evidence appropriate to that claim.

# Cross-cutting engineering requirements

The following requirements apply throughout the roadmap rather than belonging to only one phase:

- **Local-first:** core downloading, queues, rules, scheduling, history, and verification must not require an account.
- **Sensitive URL handling:** URLs, query parameters, cookies, bearer tokens, referrers, and signed links must be treated as sensitive data.
- **No silent execution:** downloaded files must never be executed automatically as a generic completion action.
- **Server friendliness:** acceleration must adapt to server behavior and avoid uncontrolled connection multiplication.
- **Durable authority:** remote, cross-device, API, and plugin actions must carry explicit authorization and purpose rather than relying only on ambient identity.
- **Clear delegation:** transfers handled by Swarm or another GoreeCloud service must remain distinguishable from engine-native transfers while still appearing coherently in the user experience.
- **Accessibility:** non-color status indicators, screen-reader semantics, keyboard/touch access, scalable type, high contrast, reduced motion, and reduced transparency must remain part of implementation acceptance.
- **Recovery before promotion:** state migration, rollback, backup/recovery, and crash resilience must be designed and tested before lifecycle promotion.
- **Evidence-backed documentation:** roadmap status must be reconciled whenever implementation, validation, cancellation, replacement, or dependencies materially change.

# Near-term implementation order

The next bounded engineering work should proceed in this order unless a later authoritative decision changes the sequence:

1. Close remaining Phase 0 governance and implementation-foundation gaps that materially gate safe Phase 1 work, without treating documentation completion as runtime completion.
2. Continue qualifying the implemented SQLite persistence backend for migration/rollback, locking, corruption, and service-lifecycle requirements as Phase 1 wiring expands.
3. Harden the integrated SQLite/filesystem/runtime path with storage-failure, missing/conflicting-artifact, database-lock/corruption, and network-transition fixtures while preserving filesystem-before-checkpoint ordering.
4. Add active in-flight pause/cancel lifecycle control and qualify representative live HTTPS/TLS behavior; define redirect, authentication/cookie, and proxy policy explicitly before enabling those surfaces.
5. Establish the first service/CLI control boundary around the verified local runtime and prove process-lifecycle recovery through that boundary without expanding to remote control.
6. Add checksum verification and reason-specific retry behavior.
7. Add controlled multipart acceleration and per-host limits.
8. Add queues, priorities, scheduling, bandwidth, and storage policy.
9. Establish the first Linux service/CLI client as the reference desktop implementation, then build the Glaze UI client against the stable service contract and expand to Windows and Android.
10. Introduce browser integration, APIs, cross-device orchestration, and broader GoreeCloud integrations only after the local engine and authorization boundaries are stable.

This sequencing intentionally prevents browser interception, remote control, synchronization, or ecosystem integration from becoming dependencies of the basic download engine.

# Roadmap completion rule

A phase is complete only when its required implementation exists in the authoritative source, applicable automated and manual validation has passed, exact source/release identities are known, documentation reflects the verified state, related task records are reconciled, and no unresolved blocker makes the completion claim inaccurate.

Planned, partial, experimental, unmerged, or unverified work remains open.
