# Rust Full Port Plan (NanoClaw)

This document is the execution plan for fully porting NanoClaw from Node/TypeScript to Rust.

## Goal

Ship a Rust-native runtime that replaces the current Node.js process while preserving behavior:

- Message routing, group isolation, and trigger logic
- SQLite-backed state/session/task persistence
- Group queue and concurrent processing controls
- IPC watcher and authorization rules
- Container orchestration and streaming output parsing
- Scheduler behavior and task lifecycle

## Progress

- [x] Rust workspace scaffolded at `rust/` (`nanoclaw-core`, `nanoclawd`)
- [x] Initial core modules added (`config`, `db`, `queue`) with unit tests
- [x] `nanoclawd --dry-run` bootstrap binary added
- [x] Ported in-memory DB API surface parity for core state/session/group/message access (SQLite-backed persistence pending)
- [x] Ported runtime queue/message prompt assembly flow into `nanoclaw-core::runtime`
- [x] Ported container output marker parsing into `nanoclaw-core::container_output` with multi-marker tests


## Non-Goals (Phase 1)

- Feature expansion beyond current behavior
- New providers/channels beyond migration requirements
- UI/dashboard additions

## Current TS Surface To Port

Primary runtime files:

- `src/index.ts` (orchestration + message loop)
- `src/db.ts` (SQLite operations)
- `src/group-queue.ts` (queue + concurrency)
- `src/ipc.ts` (IPC auth + command processing)
- `src/router.ts` (formatting/routing)
- `src/container-runner.ts` (container invocation + output markers)
- `src/container-runtime.ts` (runtime adapter)
- `src/task-scheduler.ts` (scheduling loop)

## Architecture Target (Rust)

- Runtime: `tokio`
- DB: `rusqlite` (or `sqlx` + sqlite, choose one and standardize)
- HTTP/Callback server (for future OAuth flows): `axum`
- Serialization: `serde`, `serde_json`
- Logging: `tracing`, `tracing-subscriber`
- Scheduling: cron parser crate + tokio timers
- Process control: `tokio::process`
- File watch / polling IPC: poll first (behavior parity), watch crate optional later

## Phased Migration

### Phase 0 — Compatibility Contract (1-2 days)

Define strict behavior contract from current implementation:

- DB schema + SQL semantics
- Message ordering and cursor handling
- Group authorization rules (main vs non-main)
- Container timeout and streaming semantics

Deliverable: migration checklist + golden test vectors from TS behavior.

### Phase 1 — Rust Core Library (3-5 days)

Build `nanoclaw-core` crate with:

- Config parsing + validation
- DB access layer
- Queue primitives
- Router formatting logic

Deliverable: unit tests with parity cases.

### Phase 2 — Rust Runtime Daemon (4-7 days)

Build `nanoclawd` binary:

- Main loop
- IPC watcher
- Scheduler
- Container runner integration

Deliverable: runs without channel integration in dry-run mode.

### Phase 3 — Channel Boundary Strategy (4-10 days)

Two options:

1. Sidecar-first: keep existing TS WhatsApp adapter as sidecar via IPC.
2. Native-first: replace channel directly in Rust.

Recommended: sidecar-first for risk reduction.

Deliverable: end-to-end message in/out through Rust orchestrator.

### Phase 4 — Cutover + Cleanup (2-4 days)

- Service scripts to launch Rust daemon
- Remove TS runtime from primary path
- Keep fallback runbook for rollback

Deliverable: Rust as default runtime.

## Risk Register

1. **Behavior drift in queue/cursor logic**
   - Mitigation: golden regression suite against production-like fixtures.
2. **Container output parser incompatibility**
   - Mitigation: byte-for-byte marker parser tests.
3. **Scheduler edge cases (timezone, missed runs)**
   - Mitigation: deterministic clock tests + migration canary.
4. **Ops migration friction**
   - Mitigation: dual-run mode and rollback script.

## Definition of Done

Rust runtime is considered complete when:

- All parity tests pass
- Existing groups/tasks continue without data loss
- Main/non-main authorization rules are preserved
- Container lifecycle and timeout behavior match current runtime
- Setup/verify flows recognize Rust runtime as primary process

## Immediate Next Tasks

1. Create `rust/` workspace with `nanoclaw-core` and `nanoclawd` crates.
2. Port DB schema and core queries with compatibility tests.
3. Port group queue and cursor semantics from `src/index.ts` + `src/group-queue.ts`.
4. Port container runner marker protocol from `src/container-runner.ts`.
5. Add integration tests for IPC auth and scheduler transitions.


## Rust Workspace Layout

- `rust/nanoclaw-core`: config, router, state/db API, queue, scheduler, runtime orchestration primitives
- `rust/nanoclawd`: daemon bootstrap binary and entrypoint for the Rust runtime

## Local E2E Smoke Run (Current Rust Port)

Run a minimal end-to-end flow with file input/output:

```bash
cat > /tmp/nanoclaw-e2e-input.tsv <<'EOF'
2026-01-01T00:00:01Z	group-a	Alice	Hello from A
2026-01-01T00:00:02Z	group-b	Bob	Hello from B
EOF

cd rust
cargo run -p nanoclawd -- --e2e --input /tmp/nanoclaw-e2e-input.tsv --output /tmp/nanoclaw-e2e-output.tsv
cat /tmp/nanoclaw-e2e-output.tsv
```

This validates the current Rust path for:

- input parsing
- queue dispatch
- runtime prompt assembly
- output generation

It is a migration bridge and not yet a full replacement for production channel/container integration.

### Troubleshooting: `No such file or directory` on `--e2e --input`

If you see an input-file error, create the input fixture first:

```bash
cat > /tmp/nanoclaw-e2e-input.tsv <<'EOF'
2026-01-01T00:00:01Z	group-a	Alice	Hello from A
2026-01-01T00:00:02Z	group-b	Bob	Hello from B
EOF
```

Then run:

```bash
cd rust
cargo run -p nanoclawd -- --e2e --input /tmp/nanoclaw-e2e-input.tsv --output /tmp/nanoclaw-e2e-output.tsv
```


## Rust-First Local Commands

If you are migrating away from Node.js, use this Rust-only command set:

```bash
cd rust
cargo build
cargo test

printf "2026-01-01T00:00:01Z\tgroup-a\tAlice\tHello from A\n2026-01-01T00:00:02Z\tgroup-b\tBob\tHello from B\n" > /tmp/nanoclaw-e2e-input.tsv

cargo run -p nanoclawd -- --e2e --input /tmp/nanoclaw-e2e-input.tsv --output /tmp/nanoclaw-e2e-output.tsv
cat /tmp/nanoclaw-e2e-output.tsv
```

Parse raw container stdout markers into TSV records:

```bash
cargo run -p nanoclawd -- --parse-container-output --input /tmp/container-stdout.log --output /tmp/container-parsed.tsv
cat /tmp/container-parsed.tsv
```
