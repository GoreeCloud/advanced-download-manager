# goreecloud-download-state

Portable, backend-neutral durable job-state and recovery contracts for GoreeCloud Advanced Download Manager.

This crate defines schema compatibility, versioned checkpoints, deterministic staging paths, optimistic generation batches, restart recovery decisions, partial-artifact length reconciliation, and completed-artifact validation. It deliberately performs no database, serialization, network, or filesystem I/O.

See `docs/architecture/ADR-0002-durable-job-state-and-recovery-contracts.md` for the governing durability and migration rules.
