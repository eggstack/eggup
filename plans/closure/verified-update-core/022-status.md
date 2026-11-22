# Verified Update Core M022 — Closure and Verification Record

## Status

**Status: closed 2026-11-22; published; registry-verified; hosted-qualified.** M022 published only `eggup-core 0.2.2` as the exact-pin prerequisite for Eggpack M010. The published artifact is the byte-identical, qualified package from source SHA `cdfe0bf0452d64c50da00a9e21c62ad2a0140bff`; the permanent registry-only fixture resolves that exact release from crates.io. No production/API/dependency change, consumer migration, tag, GitHub Release, or PR is part of this milestone. Archive M009 and Acquisition M023 remain separate downstream publication gates; Eggpack M010 is still last.

No M022 product finding remains open. The default-parallel local gate's fake-curl child was terminated with signal 9 (`exit unknown`); its cause was not established. The full serialized gate passed, and all hosted lanes passed. This environment observation is recorded as unresolved, not misrepresented as a product fix or as a green default-parallel run. Existing registry findings outside this package-only milestone retain their prior dispositions.

## Source plan and roadmap

- Plan: `plans/implementation/verified-update-core/022-eggup-core-0.2.2-exact-pin-publication.md`
- Roadmap: `plans/subsystems/verified-update-core-roadmap.md`
- Registry: `plans/registry.md`
- Closure requirements: `plans/closure/README.md`
- Long-term requirements: `plans/000-long-term-specification.md` and `plans/001-terminology-and-domain-model.md`
- Applicable ADRs: ADR-0001, ADR-0002, ADR-0003

## Reviewed baseline and source attribution

| Item | Value |
|---|---|
| M022 plan baseline | `3be567e9839b555d7d9d5e6e43f14de4a13c901d` |
| Publication source SHA | `cdfe0bf0452d64c50da00a9e21c62ad2a0140bff` (`m022-workspace-0.2.2-release-readiness`) |
| Workspace source version | `0.2.2` |
| Core production-source diff from M016 publication | Empty: `git diff b5a989a72dc185b30cd8c0cb67b5110ed03acd86 cdfe0bf0452d64c50da00a9e21c62ad2a0140bff -- crates/eggup-core/src` |
| Package-only scope | Core changelog/package selection and release evidence; no Core source, manifest, dependency, or `Cargo.lock` change |

The initial release candidate was committed and pushed before hosted qualification. Its clean commit is the package provenance SHA. The fixture and closure records added after publication do not alter the already-published archive.

## Authorization, publication, and readback

The user explicitly authorized the selective 0.2.2 publication sequence. Immediately before publication the 0.2.2 Core version was absent and prior published versions were unyanked. Only `cargo publish -p eggup-core --locked` was performed for this milestone; no tag, GitHub Release, PR, yank, or other package publication was performed as part of M022.

| Registry fact | Readback |
|---|---|
| Exact version | [`eggup-core 0.2.2`](https://crates.io/api/v1/crates/eggup-core/0.2.2); `yanked=false` |
| Publisher and publication time | `dbowm91`; `2026-11-22T03:04:54.970630Z` |
| Registry checksum | `a0b0dba6035ec7d403ddf0298f1021a5bd544764f9bb4e3011cb31b2dacd614b` |
| Local/registry archive comparison | SHA-256 values match; `cmp` reports byte-for-byte identity |
| Package provenance | `.cargo_vcs_info.json` records source SHA `cdfe0bf0452d64c50da00a9e21c62ad2a0140bff` |
| Current same-SHA hosted rerun | [run 44978544558, attempt 2](https://github.com/eggstack/eggup/actions/runs/44978544558), after publication, all five lanes green on the same SHA |

The exact-version API response independently confirmed the live registry record, publisher, timestamp, checksum, and unyanked state. The downloaded registry archive was checked against the local `.crate`; package digest and VCS provenance agree. No existing immutable version was overwritten or yanked.

## Requirement → evidence matrix

| # | Requirement | Evidence and result |
|---|---|---|
| R1 | Publish only Core 0.2.2 from one clean, hosted-qualified source SHA | Only `eggup-core 0.2.2` was uploaded from `cdfe0bf0452d64c50da00a9e21c62ad2a0140bff`; candidate run 44978544558 attempt 1 and post-publication attempt 2 both passed all five hosted lanes on that SHA. **Passed.** |
| R2 | Registry metadata, artifact digest, and provenance agree | Exact crates.io record is unyanked and names publisher `dbowm91`, checksum `a0b0dba6035ec7d403ddf0298f1021a5bd544764f9bb4e3011cb31b2dacd614b`, and the same publication time. Local and downloaded registry archives match byte-for-byte; `.cargo_vcs_info.json` records the release SHA. **Passed.** |
| R3 | Core package contains all authoritative contract docs and expected dependency surface | The 26-path package inventory includes `docs/domain.md`, `docs/verification.md`, and `docs/transaction.md`; no `tests/` or `benches/` directory is packaged. The normalized manifest retains `sha2` and Windows-only `self-replace`, with no transport/service/Eggpack dependency. **Passed.** |
| R4 | A registry-only exact-version consumer resolves Core 0.2.2 | Permanent fixture at `plans/closure/verified-update-core/022-package-fixture/` declares only `eggup-core = "=0.2.2"`. Unlocked and locked checks, `cargo tree`, and metadata passed; metadata reports `eggup-core 0.2.2 registry+https://github.com/rust-lang/crates.io-index`, with no local path or Git source. **Passed.** |
| R5 | Full local gate and dependency boundary are honestly recorded | `RUST_TEST_THREADS=1 ./scripts/check-local.sh` passed. Default `./scripts/check-local.sh` failed in the fake-curl suite when its child received SIGKILL 9 (`exit unknown`); root cause remains unconfirmed. The dependency tree was reviewed and no Core boundary change exists. **Serialized gate passed; default-parallel result failed and is not claimed green.** |
| R6 | No public behavior, API, dependency, or migration change | Core `src/` diff from the M016 publication source is empty; package/version metadata only changed for this promotion. Consumers do not need to migrate. SHA-256 remains integrity evidence only, with no authenticity/signature claim. **Passed.** |
| R7 | Release records accurately describe this selective milestone | Core changelog, root changelog, plan, roadmap, registry, architecture, agent-facing guidance, and consumer release snapshot identify Core as the only 0.2.2 publication so far; other milestones and exact pins remain explicit. **Passed after the documentation-truth gate below.** |

## Package inventory

`cargo package -p eggup-core --list --locked` returned these 26 paths:

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

## Commands and results

| Command/check | Result |
|---|---|
| `cargo fmt --all -- --check` | Passed as part of the qualified local gate. |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | Passed as part of the serialized local gate and hosted Stable lane. |
| `cargo test --workspace --all-targets --all-features --locked` | Passed under `RUST_TEST_THREADS=1` and on hosted Stable/macOS lanes. |
| `./scripts/check-local.sh` | Default-parallel invocation failed at a fake-curl test child terminated with signal 9 (`exit unknown`); cause not established. Do not treat as a pass. |
| `RUST_TEST_THREADS=1 ./scripts/check-local.sh` | Passed the complete gate, including docs, dependency-tree resolution/review, and `scripts/check-docs-truth.sh`. |
| `cargo tree --workspace --locked` | Passed; reviewed for the documented dependency boundaries. |
| `cargo package -p eggup-core --list --locked` | Passed; exact 26-path inventory above. |
| `cargo package -p eggup-core --locked` | Passed on the clean publication source SHA. |
| `cargo publish -p eggup-core --locked --dry-run` | Passed on the clean publication source SHA. |
| `cargo publish -p eggup-core --locked` | Published once; exact version readback followed. |
| Registry archive SHA-256 + `cmp` | Passed; downloaded crates.io archive is byte-identical to the qualified local package. |
| `cargo check --manifest-path plans/closure/verified-update-core/022-package-fixture/Cargo.toml` | Passed; resolves Core 0.2.2 from crates.io. |
| `cargo check --manifest-path plans/closure/verified-update-core/022-package-fixture/Cargo.toml --locked` | Passed. |
| `cargo tree --manifest-path plans/closure/verified-update-core/022-package-fixture/Cargo.toml --locked` | Passed; the tree has no local-path or Git source. |
| `cargo metadata --manifest-path plans/closure/verified-update-core/022-package-fixture/Cargo.toml --locked --format-version 1` | Passed; reports Core 0.2.2 from the crates.io registry. |
| `git diff --check` | Passed on the release and closure documentation changes before commit. |
| `gh run view 44978544558` | Attempt 1 passed before upload; attempt 2 passed after upload on exact source SHA `cdfe0bf0452d64c50da00a9e21c62ad2a0140bff`. |

## Hosted platform evidence

Run [44978544558](https://github.com/eggstack/eggup/actions/runs/44978544558) uses exact head SHA `cdfe0bf0452d64c50da00a9e21c62ad2a0140bff`. Attempt 2 was triggered after Core 0.2.2 was visible on crates.io. All five hosted jobs completed successfully:

| Hosted lane | Result |
|---|---|
| Stable checks | Passed |
| MSRV check | Passed |
| Linux systemd service qualification | Passed |
| macOS tests | Passed |
| Windows archive, acquisition, and service tests and check | Passed |

This evidence is for the exact publication source. The hosted lanes are not inferred from the local macOS run. The later closure-documentation commit is independently requalified before Archive M009 is allowed to proceed.

## Invariant, compatibility, failure, and security review

- Core remains policy-neutral; its non-Windows runtime graph is `sha2` only and Windows additionally uses `self-replace`. No transport, service-manager, archive, or producer dependency was added.
- No production source/API/runtime behavior changed. No consumer migration or data/configuration change is required.
- Registry versions are immutable. No yank, rollback, or authenticity claim is made. SHA-256 is package integrity evidence only.
- No installation state is touched. An ambiguous upload/readback would have stopped dependent publication; the exact registry metadata, archive, digest, and provenance all agree.
- No new medium-or-higher finding was introduced. The unexplained default-parallel test child termination remains an environment observation; the serialized gate and hosted qualification passed.

## Documentation, roadmap, and registry disposition

Core M022 is closed as the Core exact-pin prerequisite. Archive M009 is next; Acquisition M023 follows Archive and must publish Acquisition before Eggfetch; Eggpack M010 remains the final exact-pin consumer. The immutable published Eggpack 0.2.1 continues to pin its three Eggup dependencies at `=0.2.1` and is unchanged. Curl, Service, and `eggup-transport-footprint` are outside this publication set.

The registry, Core roadmap, Core M022 plan, Core changelog, root changelog, architecture overview and governance release table, `docs/releases.md`, `AGENTS.md`, and docs-hygiene/verification skills are reconciled to the read-back. The closure-documentation commit is separately qualified on its exact pushed SHA before Archive M009 proceeds; the original publication SHA and artifact provenance above remain unchanged.