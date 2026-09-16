---
title: "ADR-0002 — Durable Job State and Recovery Contracts"
document_type: "Architecture Decision Record"
version: "v0.1"
product_version: "0.1.0"
release_lifecycle: "Development"
status: "Accepted"
classification: "Public"
last_updated: "2026-09-15"
---

# ADR-0002 — Durable Job State and Recovery Contracts

## Decision

Establish a backend-neutral durable-state contract before selecting or implementing the production persistence backend.

The initial contract is implemented in `crates/download-state` and defines schema compatibility, versioned job checkpoints, deterministic partial-file paths, optimistic store generations, atomic mutation-batch semantics, crash-recovery dispositions, partial-file length reconciliation, and final-artifact revalidation rules.

This decision intentionally separates **durability semantics** from the future database or serialization technology. A later SQLite or other persistence adapter must implement these rules rather than inventing independent recovery behavior.

## Durable-state rules

### Schema and migration

- The initial durable schema contract is version `1`.
- Schema version `0` is invalid.
- Older recognized schemas require an explicit upgrade path before normal use.
- A state store written by a newer unsupported schema must fail closed rather than being opened and rewritten as though it were compatible.
- A future migration must preserve a recoverable pre-migration state or execute inside a backend transaction that can roll back completely. Migration failure must leave the last verified pre-migration state authoritative.
- Destructive downgrade is not assumed. A downgrade path must be separately designed and validated if required.

### Transaction boundary and concurrency

Durable job mutations are grouped into an all-or-nothing commit batch. Each batch carries an expected store generation and advances that generation by exactly one.

A persistence backend must reject a batch when the authoritative generation no longer matches the batch's expected generation. This prevents one writer from silently overwriting changes committed by another writer or process.

The backend must not expose a partially applied batch as committed state.

### Partial-file isolation and promotion

Incomplete transfer bytes must live at a deterministic per-job staging path distinct from the final destination. The initial naming contract derives a sibling path in the destination directory using:

`<final-file-name>.gcdm-part.<job-id>`

The final destination must not be treated as complete merely because bytes exist. Promotion from the staging file to the final path remains gated by transfer completion and the applicable verification/processing requirements in the shared core.

### Checkpoint ordering

A persisted byte checkpoint must never claim more durable transfer data than actually exists in the staging artifact.

A future storage adapter must order writes so that the durable checkpoint does not advance ahead of the durable partial-file boundary. Exact flushing and synchronization mechanisms are backend/platform specific and remain a later implementation decision.

### Crash recovery

On restart, the staging artifact and checkpoint must be reconciled before resume:

- if staging length equals the committed checkpoint, the checkpoint is consistent;
- if staging length is greater than the checkpoint, the extra tail is uncommitted and must be truncated back to the checkpoint before resume;
- if staging length is less than the committed checkpoint, the checkpoint cannot be trusted as a contiguous durable prefix and safe recovery requires restart from the beginning unless a later stronger verified recovery mechanism proves otherwise.

Queued, downloading, paused, and waiting jobs recover through the existing HTTP request-planning contract. Partial progress resumes only when a safe validator permits it; otherwise the transfer restarts from the beginning.

A job interrupted while verifying re-enters verification. A job interrupted while processing re-enters processing rather than being promoted directly to completed.

A completed job is not trusted blindly after restart. The final artifact must still exist and, when a known expected length exists, its length must match before completed state is accepted by the recovery layer.

Failed and cancelled jobs remain terminal unless an explicit later user or policy action changes them.

## Context

Phase 1 requires download jobs to survive process termination and system restart without combining incompatible remote content, losing track of durable bytes, or presenting partial files as successful downloads.

Selecting a database before defining those invariants would risk embedding recovery policy inside a storage library or platform adapter. Establishing the state contract first gives future persistence implementations a portable target that can be tested on Linux, Windows, and Android.

## Consequences

The repository now has executable source-level contracts for durable-state validation and recovery planning, but it still has **no durable persistence backend**. No database file is created, no state is serialized, no filesystem mutation is performed, and no restart recovery occurs at runtime yet.

The persistence backend, serialization format, locking mechanism, filesystem synchronization strategy, migration runner, and corruption-repair tooling remain separate decisions that must satisfy this ADR.

The Android CI check now includes the portable HTTP and durable-state contract crates in addition to the core-domain crate so cross-platform assumptions are caught earlier.

## Revisit conditions

Revisit this ADR if verified backend or filesystem behavior shows that these contracts cannot be implemented safely on a supported platform, or if a stronger recovery model can materially improve correctness without weakening fail-closed behavior. Convenience alone is not sufficient reason to let backend-specific semantics diverge from the shared recovery model.
