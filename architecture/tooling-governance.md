# Tooling and Planning Governance — Deep Dive

Cross-cutting review of workspace tooling, verification, and planning governance.
Covers the root workspace, local/CI verification, the `plans/` system, per-crate
docs, and the changelog. Factual as of the current tree.

## 1. Workspace layout and lints

**Root:** `Cargo.toml` (workspace, `resolver = "2"`).

Members (`Cargo.toml:3-9`):

- `crates/eggup-core`
- `crates/eggup-acquisition`
- `crates/eggup-eggfetch`
- `crates/eggup-service`
- `crates/eggup-eggpack`

No `eggup-dist` member: retired per `plans/closure/distribution-bootstrap/004-status.md`
(see §5). No thin `eggup` facade crate yet (allowed but not built;
`plans/000-long-term-specification.md` §4.5).

Shared inheritance (`Cargo.toml:11-16`): `version = "0.1.0"`, `edition = "2021"`,
`rust-version = "1.89"`, `license = "MIT"`,
`repository = "https://github.com/eggstack/eggup"`. Every crate `Cargo.toml`
inherits all five via `*.workspace = true`.

Workspace lints (`Cargo.toml:18-22`):

```toml
[workspace.lints.rust]
unsafe_code = "deny"

[workspace.lints.clippy]
all = { level = "warn", priority = -1 }
```

Each crate opts in with `[lints] workspace = true` and additionally sets
`#![forbid(unsafe_code)]` + `#![deny(missing_docs)]` at the top of its
`src/lib.rs` (all five crates, verified against the current tree).

Per-crate package metadata diverges deliberately:

- `eggup-core`, `eggup-acquisition`, `eggup-eggfetch`, `eggup-service`:
  public-facing (`homepage`, `documentation = https://docs.rs/<crate>`,
  `description`, `readme = "README.md"`, `keywords`, `categories`, `exclude`
  of `tests/`/`benches/` and for core also `docs/`).
- `eggup-eggpack`: `publish = false`, no `homepage`/`documentation`/`keywords`;
  depends on `eggup-core` + `eggup-acquisition` by path (`=0.1.0`) and on
  `eggpack-manifest` by git rev
  (`678bbf04f5a02827003a1d9ab83ba4f0e6360e41`, package `eggpack-manifest`).
  This encodes the ADR-0004 rule that `eggup-core` stays Eggpack-independent
  while only the optional adapter touches producer types.

`Cargo.lock` is checked in. `scripts/` holds only `check-local.sh`.

## 2. Local verification and CI mapping

**Local gate:** `scripts/check-local.sh` (5 lines, `set -euo pipefail`):

1. `cargo fmt --all -- --check`
2. `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
3. `cargo test --workspace --all-targets --all-features --locked`
4. `cargo doc --workspace --no-deps --locked`
5. `cargo tree --workspace --locked` (dependency-surface review, no gate)

**CI:** `.github/workflows/ci.yml` (`permissions: contents: read`, triggers on
`push` + `pull_request`), four jobs:

| Job | Runner | Toolchain | Command(s) | Maps to local |
|---|---|---|---|---|
| `stable` (Stable checks) | `ubuntu-latest` | stable + `rustfmt, clippy` | `fmt --check`; `clippy --workspace --all-targets --all-features --locked -- -D warnings`; `test --workspace --all-targets --all-features --locked`; `doc --workspace --no-deps --locked` | steps 1–4 verbatim |
| `msrv` (MSRV check) | `ubuntu-latest` | pinned `1.89.0` | `cargo check --workspace --all-targets --locked` | MSRV floor from `rust-version`; not in local script |
| `macos` (macOS tests) | `macos-latest` | stable | `cargo test --workspace --all-targets --all-features --locked` | step 3 on second OS |
| `windows-check` (Windows check) | `windows-latest` | stable | `cargo check --workspace --all-targets --locked` | compile-only Windows lane (no test/doc/clippy) |

Notes:

- CI does **not** run `cargo tree`; that surface check is local-only.
- Local script does **not** replicate the MSRV pin or the macOS/Windows lanes;
  contributors must rely on CI (or manual matrix runs) for platform/MSRV
  evidence — closure records explicitly require per-platform reporting
  (`plans/closure/README.md`, `plans/003-planning-process.md` §15).
- Registry cites hosted CI at implementation wave `889a234c` as passing
  stable fmt/clippy/test/doc, Rust 1.89 check, macOS tests, Windows check
  (`plans/registry.md`).

## 3. Planning system

Entry point: `plans/README.md`. Normative process: `plans/003-planning-process.md`.
Control surface: `plans/registry.md`.

### 3.1 Canonical long-term documents (normative, not edited in ordinary work)

| File | Role |
|---|---|
| `plans/000-long-term-specification.md` | End-state product boundary, 27 sections (§1 product … §27 completion criteria); mechanism-vs-policy, layered crate model, transaction/verification/ownership/lock/rollback/service/bootstrap rules |
| `plans/001-terminology-and-domain-model.md` | 36 canonical terms (Deployment … Core boundary rule); `Ownership = Absent \| Owned \| Foreign \| Unknown`; `PostCommitFailurePolicy = KeepInstalled \| RollBack` |
| `plans/002-long-term-roadmap.md` | 14 dependency-ordered phases (Phase 0 foundation … Phase 13 API stabilization); cross-phase execution rules; Phase 9 freezes/retires `eggup-dist` to Eggpack |
| `plans/003-planning-process.md` | 17 normative sections: horizons, canonical docs, ADRs, roadmaps, 16-field plan template, Invariant/Capability/Infrastructure/Polish classes, hard/interface/soft/operational dependencies, sizing, consumer-evidence rule, security rule, correctives, closure content, registry rule, SHA baselines, verification honesty, API discipline, subsystem list |

Hierarchy (`plans/README.md`): spec + terminology → ADRs → master roadmap →
subsystem roadmaps → milestone implementation plans → implementation/verification
→ closure records + archive. Every roadmap/plan classifies work as
Invariant / Capability / Infrastructure / Polish; infrastructure alone is not a
completed capability until a real consumer proves the path.

### 3.2 ADRs (`plans/adrs/`, guide `plans/adrs/README.md`)

Naming `ADR-NNNN-short-title.md`; lifecycle
`proposed → accepted → deprecated/superseded` (or `rejected`); accepted ADRs are
superseded, never rewritten.

| ADR | Title / status |
|---|---|
| `ADR-0001-layered-mechanism-and-policy-ownership.md` | Layered mechanism/policy ownership; **accepted**, distribution ownership **superseded by ADR-0004** |
| `ADR-0002-multi-artifact-transaction-and-rollback.md` | ArtifactSet as mutation unit, explicit rollback; **accepted** |
| `ADR-0003-verification-layers-and-transport-neutrality.md` | Integrity/authenticity/candidate distinct; core has no mandatory transport; **accepted** |
| `ADR-0004-eggpack-producer-eggup-consumer-boundary.md` | Eggpack owns producer contracts/construction/evidence; Eggup owns local deployment; apps own release/install policy; **accepted** |

`003-planning-process.md` §3 requires classifying any cross-repo release/update
milestone against ADR-0004 before authoring Eggup work.

### 3.3 Subsystem roadmaps (`plans/subsystems/`, guide `plans/subsystems/README.md`)

Six files, each with purpose/boundary, classified work, non-goals, evidence,
target architecture, dependency graph, ordered milestones, cross-cutting
concerns, verification strategy, risks, completion definition, status table:

- `verified-update-core-roadmap.md` — M001–M007 closed/qualified
- `acquisition-transport-roadmap.md` — corrected through M004; M005 deferred
- `service-lifecycle-roadmap.md` — M001–M005 closed
- `distribution-bootstrap-roadmap.md` — archived/transferred, M001–M004 closed
- `consumer-adoption-roadmap.md` — simple/eggsearch/CodeGG M005 closed; Gregg gated
- `eggpack-manifest-interoperability-roadmap.md` — M001/M001a closed; M003 blocked; M004 blocked

`distribution-bootstrap` MUST NOT gain producer capabilities
(`003-planning-process.md` §17).

### 3.4 Implementation plans (`plans/implementation/`, template `plans/implementation/README.md`)

Layout `implementation/<subsystem>/NNN-short-title.md`; 16 required sections
(objective … handoff notes); must state SHA baseline, readiness, invariants,
explicit non-goals, failure/rollback semantics, compat/migration, tests,
verification commands, closure evidence. Current inventory (30 milestone files
plus the `README.md` template guide):

- `verified-update-core/`: 001 foundation, 002 domain/prepared-txn, 003
  commit/rollback, 004 integrity/candidate, 005 prequalification corrective,
  006 package qualification, 007 post-commit policy/deferred finalization
- `acquisition-transport/`: 001 seam + fixture, 002 Eggfetch adapter,
  003 contract/tempfile hardening corrective, 004 limits/promotion corrective
- `service-lifecycle/`: 001 manager-neutral state/ownership, 002 Unix adapters,
  003 Unix correctness/security corrective, 004 Windows SCM, 005
  prepared-transaction lifecycle integration
- `distribution-bootstrap/`: 001 contract schema, 002 uniqueness/template
  corrective, 003 conformance validators, 004 retire `eggup-dist`
- `consumer-adoption/`: 001 eggsact, 002 stegoeggo, 003 eggsearch
  service-aware, 005 CodeGG managed-runfile bundle (no M004 Gregg plan by
  design — footprint-gated)
- `eggpack-manifest-interoperability/`: 001 ReleaseManifest v1 adapter,
  001a qualification/closure-hardening corrective, 003 Eggsact real-consumer
  adoption (blocked)
- `planning-closure-hygiene-corrective/`: 001 post-batch reconciliation,
  002 C001 commit/registry baseline, 003 M003 registry/closure reconciliation

### 3.5 Closure records (`plans/closure/`, required structure `plans/closure/README.md`)

Layout `closure/<subsystem>/NNN-status.md`, same number as source plan. Must
include status (closed / conditionally closed / corrective-required / blocked),
baselines + commits, requirement→evidence matrix, exact commands run,
invariant/failure/compat/security/docs reviews, severitized unresolved
findings, roadmap + registry disposition. Compilation or happy-path-only is
explicitly insufficient; platform work reports Linux/macOS/Windows separately.

Current inventory (30 milestone records plus the `README.md` structure guide): core 001–007 (7); acquisition 001–004 (4);
service 001–005 (5); distribution 001–004 (4); consumer 001, 002, 003, 005 (4,
no 004); eggpack-interop 001, 001a, 003 (3); hygiene C001–C003 (3).

### 3.6 Registry (`plans/registry.md`)

Compact control surface: canonical-doc table, accepted-ADR table with
ADR-0001→ADR-0004 supersession note, producer/consumer ownership guard,
recently-closed foundation table, post-closure corrective findings, active
roadmap table, dependency-ready table, planned/blocked table, ASCII execution
graph, current-state bullets (Rust 1.89, per-subsystem disposition, manual
publication only, 0.1.1 lockstep patch eligible), next-handoff directive, and
the after-each-pass rule (closure → roadmap → registry → corrective →
downstream). Update rule: active/ready work, recent closure, blockers, next
transitions only — detail stays in plans/roadmaps.

### 3.7 Archive (`plans/archive/README.md`)

Retains completed/superseded/abandoned interim planning under original relative
paths. Canonical docs and accepted ADRs are never archived merely for
completion.

## 4. Per-crate docs and changelog

**Crate READMEs** (all five crates): `crates/<crate>/README.md` — package-level
boundary, scope/non-scope, and usage entry point. Package metadata in each
`Cargo.toml` points `readme = "README.md"`; `documentation` points at
`docs.rs` except unpublished `eggup-eggpack`.

**Extended docs** exist only for `eggup-core`: `crates/eggup-core/docs/`:

- `domain.md` — `InstallPlan`/`ArtifactSet`/`PreparedTransaction` contract:
  exact root, non-empty set, traversal/absolute/duplicate/symlink rejection,
  owner-private stage (0700 dirs, 0600 files), drop-cleans-own-stage-only
- `transaction.md` — `ValidatedTransaction::commit(ownership)` as sole sync
  commit path: preflight → create-new lock → under-lock ownership re-proof →
  staged-digest revalidation → backup set → rename commit; `AbsentPolicy`,
  no parent creation, `StageRevalidation`, rollback/`RecoveryRequired`
- `verification.md` — native SHA-256 streaming, single-entry sidecar parser,
  checksum≠authenticity, `NotRequired` never reaches execution/commit,
  `Prepared → Verified → Validated → Receipt` phase order

Other crates carry rustdoc + tests/fixtures instead of a `docs/` tree
(`eggup-service` platform adapters, `eggup-acquisition` seam + fixture
transport, `eggup-eggfetch` bounded adapter, `eggup-eggpack` adapter + `tests/`).

**Changelog:** `CHANGELOG.md`, single `## Unreleased` section (no versioned
history yet). Entries map 1:1 to closed corrective/qualification milestones:
core M005/M006/M007 + post-commit policy, acquisition M003/M004, Unix service
M002 + M003 corrective, Windows SCM M004, distribution M001/M002/M003
predecessor evidence. Each entry states scope, fail-closed behavior, and an
explicit no-publication / no-migration disclaimer. Registry notes published
0.1.0 crates (`eggup-acquisition`, `eggup-core`, `eggup-eggfetch`,
`eggup-service`); `eggup-dist` listed as unpublished predecessor evidence.

## 5. Review checklist (tooling/governance lens)

1. Workspace: new crate added to `Cargo.toml` members **and** inherits the five
   shared keys **and** `[lints] workspace = true`? Facade kept thin/optional?
2. Lints: `unsafe_code = deny` intact; clippy `-D warnings` green locally and
   in `stable` CI job?
3. Verification: `scripts/check-local.sh` steps 1–4 pass; `cargo tree` reviewed
   for new deps (esp. `eggup-core`: no HTTP/TLS, no service manager, `sha2`
   only)? MSRV 1.89 + macOS/Windows lanes checked in CI?
4. Planning: change touches only interim docs, or does it require ADR /
   canonical-doc update per `003-planning-process.md` §2? ADR-0004 boundary
   respected (producer work goes to Eggpack)?
5. Plan quality: SHA baseline, 16 sections, explicit non-goals + failure
   semantics + verification commands present?
6. Closure quality: requirement→evidence matrix, exact commands with results,
   invariant/failure/compat/security/docs reviews, severitized residuals,
   per-platform evidence where applicable?
7. Registry hygiene: `plans/registry.md` updated (status tables, blockers,
   execution graph, handoff) without duplicating plan detail; roadmap status
   table + archive disposition updated?
8. Docs/changelog: public API has rustdoc; `eggup-core` `docs/*.md` still
   accurate; crate `README.md` current; `CHANGELOG.md` Unreleased entry added
   with publication/migration disclaimer?
9. Release discipline: `cargo package` / `publish --dry-run` done for boundary
   changes; manual-publication rule observed (CI never publishes)?
