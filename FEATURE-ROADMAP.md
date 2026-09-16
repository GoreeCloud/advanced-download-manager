---
title: "GoreeCloud Advanced Download Manager — Feature Roadmap"
product: "GoreeCloud Advanced Download Manager"
document_type: "Feature Roadmap"
status: "Active"
version: "v0.1"
classification: "Public"
last_updated: "2026-09-15"
authoritative_record: true
repository: "GoreeCloud/goreecloud-advanced-download-manager"
canonical_source: "repository"
drive_sync_target: "GoreeCloud/Feature Roadmap/GoreeCloud Advanced Download Manager/FEATURE-ROADMAP.md"
---

# GoreeCloud Advanced Download Manager — Feature Roadmap

> **Canonical source:** This repository `FEATURE-ROADMAP.md` is the canonical editable roadmap. The corresponding GoreeCloud Drive Markdown record is a synchronized ecosystem-wide representation and must remain materially consistent with this file.
>
> **Implementation truth:** Unless a roadmap item is explicitly backed by verified source, test, build, integration, release, or runtime evidence, it remains **Planned**. The existence of `SPECIFICATIONS.md`, this roadmap, a branch, pull request, task record, prototype, or design does not establish implementation or release readiness.

## Product direction

GoreeCloud Advanced Download Manager is planned as the universal download and transfer-orchestration layer for the GoreeCloud ecosystem. It is intended to operate as a powerful standalone application on Linux, Windows, and Android while also exposing a shared download service to authorized GoreeCloud applications.

The roadmap prioritizes a durable local-first transfer engine first, then policy, automation, user experience, platform integration, and cross-device orchestration. Specialized protocols such as BitTorrent remain delegated to the appropriate GoreeCloud service rather than being duplicated unnecessarily.

## Current verified state

Status: **Pre-implementation / documentation foundation**

Verified repository state currently establishes:

- the repository exists with `main` as its default branch;
- `SPECIFICATIONS.md` records the planned product specification;
- the specification is explicitly classified as planned rather than implemented;
- no download-engine implementation, platform client, release artifact, production deployment, or runtime acceptance evidence has yet been established in this repository.

The repository documentation baseline is still incomplete under current GoreeCloud repository governance. Missing mandatory repository documents and platform-contract declarations must be added through later governed work rather than being treated as complete by this roadmap.

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

**Exit criteria:** repository governance baseline exists, architecture boundaries are documented, initial implementation plan is testable, and no planned integration is represented as accepted without evidence.

## Phase 1 — Durable local download engine foundation

**Roadmap ID:** ADM-100  
**Status:** Planned

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

1. Finish Phase 0 repository/governance baseline.
2. Implement Phase 1 durable single-stream HTTP/HTTPS transfer and persistent job state.
3. Add validator-safe resume and crash/restart recovery.
4. Add checksum verification and reason-specific retry behavior.
5. Add controlled multipart acceleration and per-host limits.
6. Add queues, priorities, scheduling, bandwidth, and storage policy.
7. Establish the first Linux service/CLI client as the reference desktop implementation.
8. Build the Glaze UI client against the stable service contract.
9. Expand to Windows and Android using the common engine/service contracts.
10. Introduce browser integration, APIs, cross-device orchestration, and broader GoreeCloud integrations only after the local engine and authorization boundaries are stable.

This sequencing intentionally prevents browser interception, remote control, synchronization, or ecosystem integration from becoming dependencies of the basic download engine.

# Roadmap completion rule

A phase is complete only when its required implementation exists in the authoritative source, applicable automated and manual validation has passed, exact source/release identities are known, documentation reflects the verified state, related task records are reconciled, and no unresolved blocker makes the completion claim inaccurate.

Planned, partial, experimental, unmerged, or unverified work remains open.
