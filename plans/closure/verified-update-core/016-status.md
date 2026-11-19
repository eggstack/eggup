# Verified Update Core M016 — Closure and Verification Record

## Status

**Closed; registry-verified; hosted-qualified.** Only `eggup-core 0.2.1` was published for M016. The registry archive contains all three authoritative contract documents, the registry-only consumer resolves the exact published version, the pre- and post-publication CI matrices are green on the fixed source commit, and the `v0.2.1` tag and GitHub Release point at that commit. Finding 14 is closed for consumers of Core 0.2.1.

The limitation for consumers pinned to immutable `eggup-core 0.2.0` remains: that archive does not contain these documents. Published `eggup-eggpack 0.2.0` remains pinned to Core `=0.2.0` and is unchanged. M016 published no other crate. No medium-or-higher finding introduced by M016 remains open.

## Source plan and roadmap

- Plan: `plans/implementation/verified-update-core/016-eggup-core-contract-doc-packaging.md`
- Roadmap: `plans/subsystems/verified-update-core-roadmap.md`
- Registry: `plans/registry.md`
- Long-term requirements: `plans/000-long-term-specification.md` and `plans/001-terminology-and-domain-model.md`
- Applicable ADRs: ADR-0001, ADR-0002, ADR-0003

## Reviewed baseline and source attribution

| Item | Value |
|---|---|
| M016 plan baseline | `94f2545ea12fa88ab7fb5e135ba587d0fa6f584c` |
| Publication source | `b5a989a72dc185b30cd8c0cb67b5110ed03acd86`; merge commit for PR [#3](https://github.com/eggstack/eggup/pull/3), merged to `main` |
| Workspace source version | `0.2.1` |
| Separate M021 test-only implementation commit | `5f7f074baa491f62a5e372d0caacaacea5148716` |

M016 itself changes package selection only. The publication source does contain a Core source diff relative to the M016 plan baseline: `git diff --stat 94f2545ea12fa88ab7fb5e135ba587d0fa6f584c b5a989a72dc185b30cd8c0cb67b5110ed03acd86 -- crates/eggup-core/src/` reports only `crates/eggup-core/src/lib.rs` (115 insertions, 42 deletions). This is M021's separately planned native bounded-candidate fixture replacement in the `#[cfg(test)]` module, not an M016 production change; see `plans/closure/verified-update-core/021-status.md`. The source diff from M021's implementation commit to the publication commit is empty under `crates/eggup-core/src/`. No production API, runtime behavior, or Core dependency changed for M016.

## Authorization, publication, and readback

The user explicitly authorized the release before upload, with scope limited to `eggup-core 0.2.1`. The final preflight confirmed that Core 0.2.1 and tag/release `v0.2.1` did not yet exist. Only `cargo publish -p eggup-core --locked` was performed; no other workspace crate was uploaded.

| Registry/release fact | Readback |
|---|---|
| Registry version | `eggup-core 0.2.1`; not yanked |
| Publisher | `dbowm91` |
| Registry checksum | `3841a8390f8c7b1be0207156e5c895aa4b6c0eec8b05772ac8331d3e43e7ef88` |
| Archive checksum comparison | Downloaded registry `.crate` matches both crates.io metadata and the pre-publication package |
| Package provenance | `.cargo_vcs_info.json` names source SHA `b5a989a72dc185b30cd8c0cb67b5110ed03acd86` |
| Tag | Annotated `v0.2.1` object `3e6ecae9158f834b66a56cd7a9d8776f22330652`, peeled target `b5a989a72dc185b30cd8c0cb67b5110ed03acd86` |
| GitHub Release | [eggup-core 0.2.1](https://github.com/eggstack/eggup/releases/tag/v0.2.1), non-draft and non-prerelease, read back at the same source commit |

The downloaded archive was inspected directly. It contains the three docs; the archived source SHA and checksum agree with registry metadata. Remote tag readback confirmed `v0.1.1`, `v0.1.2`, `v0.1.3`, and `v0.2.0` retain their prior targets; the only new tag is `v0.2.1`.

## Requirement → evidence matrix

| # | Requirement | Evidence and result |
|---|---|---|
| R1 | Only Core 0.2.1 is published; metadata, checksum, publisher, and source SHA agree | crates.io reports Core 0.2.1, publisher `dbowm91`, not yanked, checksum `3841a8390f8c7b1be0207156e5c895aa4b6c0eec8b05772ac8331d3e43e7ef88`; archive digest and `.cargo_vcs_info.json` match. Other six publishable crates remain at 0.2.0. **Passed.** |
| R2 | Package includes all three authoritative contract documents and excludes tests/benches | Local package inventory and downloaded registry archive both contain `docs/domain.md`, `docs/verification.md`, and `docs/transaction.md`; no `tests/` or `benches/` entries. **Passed.** |
| R3 | No new Core dependency, API, production behavior, or path/git/patch edge | Core dependency surface remains `sha2` plus Windows-only `self-replace`. The M016 production change is package selection only. The separate M021 source delta is confined to the test module, as its closure records. **Passed.** |
| R4 | Workspace, package, and documentation gates pass | `./scripts/check-local.sh`, `./scripts/check-docs-truth.sh`, `cargo tree --workspace --locked`, package build, and publish dry-run passed on the exact publication source. The dependency tree was reviewed; no prohibited Core boundary was introduced. **Passed.** |
| R5 | All five hosted lanes pass before and after publication on the exact source SHA | Pre-publication run `44391076419` attempt 1 passed all five lanes. Post-publication attempt 2 passed Linux systemd, MSRV, Stable, macOS, and Windows on SHA `b5a989a72dc185b30cd8c0cb67b5110ed03acd86`; the strict packaged-service step passed against registry Core 0.2.1. **Passed.** |
| R6 | A direct registry-only consumer proves Core 0.2.1 resolves from crates.io | Checked-in fixture at `plans/closure/verified-update-core/016-package-fixture/` declares only `eggup-core = "=0.2.1"`. Its lockfile/metadata use registry sources; no git or local-path dependency. Unlocked/locked checks, tree, and metadata passed. **Passed.** |
| R7 | Tag and GitHub Release follow registry proof and target the publication SHA | Remote annotated tag peels to `b5a989a72dc185b30cd8c0cb67b5110ed03acd86`; GitHub Release is published at the same target. Prior tags were read back unchanged. **Passed.** |
| R8 | Only intended package version is affected; old artifacts remain immutable | M016 published only `eggup-core 0.2.1`. The prior Core 0.2.0 archive, published eggup-eggpack 0.2.0 and its `=0.2.0` pins, older package versions, and existing tags remain unchanged. **Passed.** |
| R9 | Finding 14, roadmap, registry, and docs reflect the actual disposition | Finding 14 is closed for direct Core 0.2.1 consumers; the immutable 0.2.0 limitation remains explicit. The M016 plan, registry, Core roadmap, changelogs, release page, architecture, AGENTS, and skills are reconciled in this record's changes. **Passed.** |

## Package inventory

The output of `cargo package -p eggup-core --list --locked` was:

```text
.cargo_vcs_info.json
CHANGELOG.md
Cargo.lock
Cargo.toml
Cargo.toml.orig
README.md
docs/domain.md
docs/transaction.md
docs/verification.md
examples/custom_validator.rs
examples/multi_member.rs
examples/one_member.rs
examples/ownership.rs
examples/receipts.rs
examples/stale_lock_recovery.rs
src/candidate.rs
src/current_exe.rs
src/domain.rs
src/error.rs
src/failure_taxonomy_tests.rs
src/integrity.rs
src/lib.rs
src/lock.rs
src/stage.rs
src/test_support.rs
src/transaction.rs
```

The three documentation paths are present, and no `tests/` or `benches/` path is present. The package contains 26 entries (the 23-entry 0.2.0 package plus these three docs). The inline `src/failure_taxonomy_tests.rs` filename is part of `src/` and is not a packaged `tests/` directory.

## Commands and results

| Command | Result |
|---|---|
| `./scripts/check-local.sh` | Passed on publication source; formatting, Clippy, workspace tests, docs, dependency resolution/review, and documentation-truth checks all completed. |
| `./scripts/check-docs-truth.sh` | Passed on publication source. It was also rerun after this closure/documentation reconciliation; see the final verification below. |
| `cargo tree --workspace --locked` | Passed; resolved graph reviewed for the documented crate boundaries. |
| `cargo package -p eggup-core --list --locked` | Passed; exact 26-path inventory above. |
| `cargo package -p eggup-core --locked` | Passed on publication source. |
| `cargo publish --dry-run -p eggup-core --locked` | Passed on publication source. |
| `cargo check --manifest-path plans/closure/verified-update-core/016-package-fixture/Cargo.toml` | Passed; exact Core 0.2.1 registry dependency. |
| `cargo check --manifest-path plans/closure/verified-update-core/016-package-fixture/Cargo.toml --locked` | Passed. |
| `cargo tree --manifest-path plans/closure/verified-update-core/016-package-fixture/Cargo.toml --locked` | Passed; no local-path or Git source. |
| `cargo metadata --manifest-path plans/closure/verified-update-core/016-package-fixture/Cargo.toml --locked --format-version 1` | Passed; the Core package resolves from crates.io. |
| `gh run view 44391076419 --repo eggstack/eggup` | Post-publication attempt 2: workflow completed with conclusion `success` on the exact source SHA. |
| `gh release view v0.2.1 --repo eggstack/eggup` | Passed readback; correct tag, target, body, and public non-draft release. |
| `git ls-remote --tags origin ...` | Passed readback; `v0.2.1` peels to the source SHA and prior tags retain their targets. |

The first 300-second `gh run watch` process ended while the macOS job was still in progress; this was a watcher timeout, not a CI lane failure. A second watcher completed successfully, and the final run API readback confirms all five lanes are complete and green.

## Hosted platform evidence

Run [44391076419](https://github.com/eggstack/eggup/actions/runs/44391076419) uses exact head SHA `b5a989a72dc185b30cd8c0cb67b5110ed03acd86`. Attempt 1 passed all five lanes before publication. Attempt 2 passed all five lanes after publication.

| Lane | Pre-publication attempt 1 | Post-publication attempt 2 |
|---|---|---|
| Linux systemd service qualification | Passed | Passed |
| MSRV check (Rust 1.89) | Passed | Passed |
| Stable checks | Passed | Passed |
| macOS tests | Passed | Passed |
| Windows archive, acquisition, and service tests and check | Passed | Passed |

The post-publication Stable, MSRV, macOS, and Windows lanes each passed the strict `cargo package -p eggup-service --locked` step against the now-published registry Core 0.2.1. Linux systemd is a separate lane and passed its systemd qualifications.

## Compatibility, security, and failure review

- M016 adds package documentation only. No consumer API, production runtime behavior, dependency edge, feature, or migration requirement changed.
- M021's included source difference is test-only under `#[cfg(test)]` and remains separately owned by M021.
- Core remains policy-neutral; SHA-256 remains integrity evidence only. No authenticity or signature claim was introduced.
- No live installation, filesystem mutation, lock/recovery, or service lifecycle path changed.
- Registry publication is immutable; the checksum, archive, source provenance, and release targets agree. No retry, overwrite, yank, or tag move occurred.
- Consumers of Core 0.2.0 or the exact-pinned Eggpack 0.2.0 package continue to lack the M016 docs; no claim is made that an older archive was altered.

## Documentation and registry disposition

- The Core and root changelogs record the published 0.2.1 package and move M016 out of `Unreleased`.
- `docs/releases.md` reports Core 0.2.1 and the other six published crates at 0.2.0.
- The architecture version table, CI package-resolution note, AGENTS index, and affected skills reflect the registry state.
- The M016 implementation plan is marked closed and accurately distinguishes its package-only change from M021's test-only source delta.
- `plans/registry.md` records Finding 14 as closed for Core 0.2.1 and preserves the legacy-version limitation; the Core roadmap points to this closure.

## Final verification after record edits

- `./scripts/check-docs-truth.sh` — passed after the documentation edits. The script noted that check 7 was skipped because no `--baseline <ref>` was supplied; all documentation-truth checks completed successfully.
- `git diff --check` — passed after the documentation edits.
- `./scripts/check-local.sh` passed on the exact publication source before upload. It was not rerun after the closure/release-record edits, which changed documentation and planning records only; no Rust, manifest, CI, or gate source changed.
