# Archive Extraction M007 — Closure and Verification Record

## Status

**Conditionally closed — 2026-11-17.** The dependency refresh, code compatibility work, documentation, advisory check, package checks, local macOS gate, and local Rust 1.89 check are complete. Hosted Linux and Windows qualification was not run, and no commit or push was authorized. This record does not claim cross-platform closure or publication.

The workspace remains at package version `0.2.0`. All dependency changes here are local and unpublished. The already-published `eggup-archive 0.2.0` still resolves its original `zip 6.0.0` dependency; consumers receive none of this work until a separate, manually authorized publication milestone.

## Source plan and roadmap

- Source plan: `plans/implementation/archive-extraction/007-workspace-dependency-security-refresh.md`
- Source roadmap: `plans/subsystems/archive-extraction-roadmap.md`
- Affected boundaries: `plans/subsystems/acquisition-transport-roadmap.md` and `plans/subsystems/eggpack-manifest-interoperability-roadmap.md`
- Prior archive closure: `plans/closure/archive-extraction/006-status.md`

## Reviewed repository baseline

| Item | Value |
|---|---|
| Branch | `main` |
| Baseline / HEAD at review | `c789bef156aa09baaf9b349744dac12de2534a0a` |
| Relationship | HEAD remains exactly at the plan baseline; all M007 work is in the uncommitted worktree |
| Working tree | Intentionally dirty; no commit, push, tag, release, or publication was made |
| Host | macOS 26.6.2, Apple Silicon |
| Rust floor | Rust `1.89.0` checked locally |

## Implementation commits / PRs

None. The caller authorized local implementation and verification only; changes remain uncommitted.

## Executive finding

M007 updates the workspace lock and the direct requirements that prevented the identified security fixes from resolving. The archive and Eggpack crates now require `zip 9.6.1`; `eggup-eggpack` exact-pins `eggpack-manifest = "=0.1.1"`; and 60 package entries were resolved by `cargo update`. The resulting graph includes `rustls 0.23.46`, `rustls-webpki 0.103.17`, and `webpki-roots 1.0.12`, and `cargo deny check advisories` reports `advisories ok`.

The ZIP 6-to-9 API change required one test-helper type migration to `zip::write::SimpleFileOptions`. Eggup's production extraction algorithm remains explicit, allowlisted, bounded, regular-file-only, and does not call ZIP convenience extraction methods. The Eggpack adapter tests pass against manifest 0.1.1. No Eggup public API, workspace package version, dependency boundary, or Rust 1.89 floor changed.

The local macOS gate, the explicit Rust 1.89 workspace check, and both package checks pass. Linux tests and hosted CI lanes were not run. A Windows-target check was attempted but could not compile `ring` because this macOS host lacks the Windows MSVC C toolchain/headers (`assert.h`); this is an environment limitation, not Windows qualification. The milestone is therefore conditionally closed.

## Requirement → evidence matrix

| # | Requirement | Evidence | Result |
|---|---|---|---|
| 1 | Raise ZIP requirements and preserve feature selection | `crates/eggup-archive/Cargo.toml` and `crates/eggup-eggpack/Cargo.toml`: `zip = 9.6.1`, `default-features = false`, `deflate-flate2` retained | Passed |
| 2 | Raise Eggpack's exact producer-manifest pin | `crates/eggup-eggpack/Cargo.toml`: `eggpack-manifest = "=0.1.1"`; `tests/interoperability.rs` pin assertion updated | Passed |
| 3 | Preserve ZIP extraction safety and behavior | `crates/eggup-archive/src/lib.rs` remains unchanged; archive tests cover hostile paths, links, limits, duplicate entries, digest/size, cleanup, and no-clobber | Passed: 50 unit + 4 cleanup-contract + 3 member-write-parity tests |
| 4 | Preserve Eggpack adapter API and behavior | `cargo test -p eggup-eggpack --all-targets --all-features --locked`; test helper uses ZIP 9's `SimpleFileOptions` | Passed: 71 tests |
| 5 | Refresh compatible locked dependencies and review boundaries | `Cargo.lock`; full `cargo tree --workspace --locked`; Windows-target tree; version-qualified inverse trees | Passed; no forbidden Eggup edge observed |
| 6 | Remove applicable RustSec findings without suppressions | `cargo deny check advisories` → `advisories ok` (cargo-deny warned that no config path was found and used its default configuration) | Passed; no advisory suppression or new policy file |
| 7 | Retain Rust 1.89 floor | `cargo +1.89.0 check --workspace --all-targets --locked` on macOS | Passed |
| 8 | Package the changed crates | `cargo package -p eggup-archive --locked --allow-dirty`; `cargo package -p eggup-eggpack --locked --allow-dirty` | Both passed |
| 9 | Run the full local gate | `./scripts/check-local.sh` on macOS: format, Clippy, workspace tests, docs, tree review, and docs-truth | Passed on rerun; one pre-existing ignored test in `eggup-curl` |
| 10 | Preserve the platform qualification record | Local macOS evidence above; hosted Linux and Windows runs were not available/authorized | Conditional; not run |
| 11 | Record current dependency facts and unpublished state | `AGENTS.md`, `architecture/{overview,tooling-governance,eggpack-adapter}.md`, Eggpack README, both crate changelogs, root changelog, this closure, roadmap, and registry | Updated; publication explicitly not authorized |
| 12 | Keep the work uncommitted and unpublished | HEAD unchanged at baseline; no git publication action taken | Passed |

## Production implementation evidence

- `crates/eggup-archive/Cargo.toml`: ZIP requirement advanced from `6.0.0` to `9.6.1`; existing feature selection preserved.
- `crates/eggup-eggpack/Cargo.toml`: exact manifest pin advanced from `=0.1.0` to `=0.1.1`; ZIP test dependency advanced from `6.0` to `9.6.1`.
- `crates/eggup-eggpack/tests/interoperability.rs`: exact-pin assertion now checks `=0.1.1`; producer-source comments distinguish the original 0.1.0 promotion from the current dependency pin.
- `crates/eggup-eggpack/tests/archive_handoff.rs`: ZIP writer options use `SimpleFileOptions`, compatible with ZIP 9's two-lifetime `FileOptions` type.
- `cargo update` locked 60 compatible package changes. Key security/compatibility versions include `eggpack-manifest 0.1.1`, `zip 9.6.1`, `rustls 0.23.46`, `rustls-webpki 0.103.17`, `webpki-roots 1.0.12`, `eggfetch-core 0.2.2`, `eggfetch-http-connect 0.2.2`, `tokio 1.54.1`, `flate2 1.1.11`, and `serde_json 1.0.154`. `sha2` remains `0.10.9`; `eggfetch-core` remains on 0.2; ZIP is not feature-defaulted; `eggup-acquisition` remains dependency-free.
- The lockfile diff also adds packages required by the compatible graph and removes obsolete entries (`arbitrary` and `derive_arbitrary`). The reviewed tree retains the Windows-only `self-replace` edge and Windows-gated service dependencies; adapter composition and archive/core boundaries are unchanged.
- ZIP advisory applicability was considered at package level. Eggup iterates entries via `ZipArchive::by_index`, applies its own path/type/budget policy, and writes through its bounded member writer; no convenience extraction API was introduced.

## Exact commands and results

| Command | Result |
|---|---|
| `cargo update` | Passed; resolved 60 package changes |
| `cargo deny check advisories` | Passed: `advisories ok`; default-config fallback warning recorded above |
| `cargo tree --workspace --locked` | Passed; full graph reviewed |
| `cargo tree --workspace --locked --target x86_64-pc-windows-msvc --depth 1` | Passed; target-gated graph reviewed |
| `cargo tree --workspace --locked -i zip` | Passed; `zip 9.6.1` is used by archive and Eggpack tests |
| `cargo tree --workspace --locked -i eggpack-manifest` | Passed; Eggpack resolves `0.1.1` |
| `cargo tree --workspace --locked -i rustls` | Passed; TLS adapter path resolves `rustls 0.23.46` |
| `cargo tree --workspace --locked -i rustls-webpki` | Passed; resolves `0.103.17` under Rustls |
| `cargo tree --workspace --locked -i webpki-roots@1.0.12` | Passed; version-qualified tree resolves the patched roots package |
| `cargo test -p eggup-archive --all-targets --locked` | Passed; 57 tests across the library and two integration targets |
| `cargo test -p eggup-eggpack --all-targets --all-features --locked` | Passed; 71 tests across the library and integration targets |
| `cargo package -p eggup-archive --locked --allow-dirty` | Passed; package and package verification completed |
| `cargo package -p eggup-eggpack --locked --allow-dirty` | Passed; package and package verification completed |
| `./scripts/check-local.sh` | Passed on macOS after clarifying unpublished/published state in root changelog bullets; fmt, Clippy, tests, docs, dependency tree, and docs-truth all passed |
| `cargo +1.89.0 check --workspace --all-targets --locked` | Passed on macOS |
| `cargo +1.89.0 check --workspace --all-targets --target x86_64-pc-windows-msvc --locked` | Could not run to completion: `ring`'s C build failed because `assert.h` was unavailable; Windows MSVC compiler/SDK is absent on this host |
| `git diff --check` | Passed after all documentation and status edits |

The first Eggpack test run exposed the ZIP options lifetime change and failed to compile; after migrating the helper type, the complete Eggpack target passed. The first package attempts refused a dirty tree; the documented reruns used Cargo's explicit `--allow-dirty`. The first full local gate flagged missing per-bullet published-state markers in the newly added root changelog; after correcting those statements, the complete gate passed.

An unqualified `cargo tree -i webpki-roots` selector was ambiguous because the graph contains both `0.26.11` and `1.0.12`; the version-qualified selector above succeeded. This did not indicate a dependency-resolution failure.

## Platform results

| Platform/lane | Result |
|---|---|
| macOS local | `./scripts/check-local.sh`, archive/Eggpack focused tests, package checks, and Rust 1.89 check passed |
| Linux | Not run |
| Windows hosted test lane | Not run; no commit/push was authorized to trigger hosted CI |
| Windows target graph | Reviewed successfully with `cargo tree --target x86_64-pc-windows-msvc --depth 1` |
| Windows-target Rust check from macOS | Attempted; blocked in `ring` C compilation by missing MSVC toolchain headers; not platform test evidence |
| Hosted stable / MSRV / macOS / Windows / Linux systemd | Not run |

## Invariant, failure, and recovery review

- `eggup-core` still depends on `sha2 0.10.9` and Windows-only `self-replace`; it gains no archive, Eggpack, service, or transport dependency.
- `eggup-acquisition` remains a zero-dependency seam. `eggup-eggfetch` and `eggup-curl` remain separate adapters; composition remains outside them.
- `eggup-archive` remains optional and local, with explicit bounded extraction and no transport/authenticity/live-installation policy. ZIP convenience extraction is not used.
- `eggup-eggpack` remains the sole producer-type adapter, exact-pins all three in-workspace crates at `=0.2.0`, and pins the producer manifest exactly at `=0.1.1`.
- Package versions remain `0.2.0`; no public Eggup API changed. No authenticity or signature claim was introduced.
- The work changes manifests, lock resolution, one test-helper type, and documentation only. It performs no runtime install, filesystem ownership, service, process, or network-acquisition mutation. No runtime rollback or recovery action was required.

## Compatibility and migration review

No public Eggup API or runtime behavior changed. Existing archive security fixtures and Eggpack projection/handoff tests pass. The only source compatibility edit is test-only: ZIP's options type now has two lifetime parameters, so the helper names the supported `SimpleFileOptions` alias. The adapter remains compatible with `eggpack-manifest 0.1.1` under its existing API and semantics. No consumer migration is required for the local code change; no consumer receives it before a separate authorized publication.

`cargo package` for `eggup-eggpack` verified its published `0.2.0` Eggup dependency set against crates.io. That registry archive still has ZIP 6.0.0, which is expected because the local unpublished archive manifest is not the registry package. This is not evidence that the published archive package has been fixed.

## Security review and unresolved findings

At baseline, the audit recorded ten RustSec records involving the old Rustls graph and ZIP 6.0.0. After refreshing the graph, `cargo deny check advisories` reported no unresolved advisories. No advisory was suppressed and no cargo-deny policy was added. SHA-256 remains integrity evidence only.

No unresolved code or dependency finding was identified in the checked workspace graph. The remaining qualification condition is that hosted Linux and Windows execution was not run. The Windows-target check attempt is an environment-limited compile attempt only. This condition warrants **conditionally closed**, not fully closed.

## Documentation, roadmap, and registry disposition

- Current pin/version facts updated in `AGENTS.md`, `architecture/overview.md`, `architecture/tooling-governance.md`, `architecture/eggpack-adapter.md`, and `crates/eggup-eggpack/README.md`.
- Unreleased dependency notes added to `crates/eggup-archive/CHANGELOG.md`, `crates/eggup-eggpack/CHANGELOG.md`, and the root `CHANGELOG.md`; all state that the work is unpublished and does not authorize publication or migration. Historical release and plan evidence was left unchanged.
- This closure is the evidence for `plans/subsystems/archive-extraction-roadmap.md` M007 and the `plans/registry.md` active-control row.
- No consumer-facing `docs/` policy was added because public behavior did not change.

## Roadmap disposition

Record M007 as **conditionally closed** in the archive roadmap. The hosted Linux and Windows qualification remains explicitly `not run`; a later authorized CI run may discharge that condition. No publication, release, commit, or push is part of this milestone.

## Registry update

Update the archive extraction control row in `plans/registry.md` to M007 conditionally closed, identify the affected archive/TLS/Eggpack dependency surfaces, record local advisory/MSRV/package checks as passed, and retain hosted Linux/Windows as not run. Preserve the no-publication and uncommitted state.
