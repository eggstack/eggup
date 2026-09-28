# Verified Update Core M008a — Closure and Verification Record

Status: closed

Source plan: `plans/implementation/verified-update-core/008a-clean-package-evidence-corrective.md`

Source roadmap: `plans/subsystems/verified-update-core-roadmap.md`

Original plan: `plans/implementation/verified-update-core/008-core-archive-consumer-package-qualification.md`

Original closure: `plans/closure/verified-update-core/008-status.md` (amended with the M008a addendum; provisional `--allow-dirty` evidence superseded)

Hard dependency: Eggpack Interop M002a — satisfied by `plans/closure/eggpack-manifest-interoperability/002a-status.md` (hosted run `36477024102` green on all lanes).

Reviewed repository baseline: clean package evidence produced on the post-M002a tree; final head carries only `plans/` + fixture-vendor deltas over the green M002a code head (see Implementation commits). No production/package source changed.

Implementation commits:

- M002a code head `9a5500e` (green hosted run `36477024102`) — the valid final package/closure baseline the M008a plan was blocked on.
- `05433db` — docs(plans): close M002a qualification and C009 egress-evidence reconciliation (plans-only; packaged crate contents identical).
- this batch (M008a closure + M008 addendum + refreshed fixture vendor + roadmap/registry reconciliation) — SHA recorded in the post-batch registry head. Zero `crates/` source, manifest, workflow, or version changes.

## Executive finding

M008a replaces the provisional `--allow-dirty` package evidence with the exact clean-tree commands required by the M008 source plan. `cargo package` and `cargo publish --dry-run` pass without `--allow-dirty` for both `eggup-core` and `eggup-archive`, with no material difference from the provisional results — so no version bump, metadata corrective, or unrelated-crate publication was required. The clean external fixture passes 3/3 against the refreshed packaged tarballs, and the final head has a green hosted matrix. "Publication-ready" is proven; "published" remains a separate maintainer action.

## Requirement-to-evidence matrix

| Requirement (source plan Section 10) | Evidence | Result |
|---|---|---|
| exact core package + dry-run on clean tree, no `--allow-dirty` | `cargo package -p eggup-core --locked` → 19 files, 180.3 KiB (35.5 KiB compressed); `cargo publish -p eggup-core --dry-run --locked` → upload dry-run ok | passed |
| exact archive package + dry-run on clean tree, no `--allow-dirty` | `cargo package -p eggup-archive --locked` → 7 files, 138.2 KiB (26.7 KiB compressed); `cargo publish -p eggup-archive --dry-run --locked` → upload dry-run ok | passed |
| clean external fixture tar.gz + zip + missing-member | fixture `cargo test --offline`: `m001d_flow_through_packaged_core_and_archive_tar_gz`, `..._zip`, `tar_gz_mismatch_is_rejected` — 3 passed | passed |
| dependency tree checks | `cargo tree -p eggup-core` → sha2 only; `-p eggup-archive` → format stack (flate2/fs_at/sha2/tar/zip), no eggup-core production dep; fixture tree shows packaged 0.1.2 versions only | passed |
| full current-head hosted matrix green | inherited green run `36477024102` (M002a head) + final-head confirmation run (see Exact verification commands) | passed |
| Rust 1.89 all-target check | `cargo +1.89.0 check --workspace --all-targets --locked` clean (local 1.89.0 == MSRV) | passed |

## Production implementation and audit evidence

No production change: zero `crates/` source, manifest, or version edits in this pass. The packaged file lists are unchanged in shape from M008 (core 19 files incl. examples + docs; archive 7 files incl. CHANGELOG/README), and the only vendor-refresh deltas are the `.cargo_vcs_info.json` clean-commit stamp and the since-landed archive CHANGELOG M008 entry — both provenance improvements, not content changes.

Stop-condition review (all negative, as required):

- clean-tree package/dry-run does NOT differ materially from `--allow-dirty` (19/180.3 KiB vs 19/180.4 KiB; 7/138.2 KiB vs 7/138.3 KiB — sub-KiB packaging noise): no version bump needed.
- package dependency resolution required NO unrelated Eggup crates.
- no current-head hosted lane failed after M002a.

## Exact verification commands and results

Local environment: Darwin arm64; stable `rustc 1.89.0` / Cargo 1.89.0.

Passed on the post-M002a tree (packaged files clean; only `plans/` working-tree state outside packaged paths):

```text
test -z "$(git status --porcelain -- crates/ target/)"   # packaged inputs unmodified
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked   (all suites green)
cargo +1.89.0 check --workspace --all-targets --locked
cargo package -p eggup-core --locked                 # 19 files, 180.3 KiB
cargo publish -p eggup-core --dry-run --locked       # upload dry-run ok
cargo package -p eggup-archive --locked              # 7 files, 138.2 KiB
cargo publish -p eggup-archive --dry-run --locked    # upload dry-run ok
cargo tree -p eggup-core --locked                    # sha2 only
cargo tree -p eggup-archive --locked                 # no eggup-core production dep
cargo doc --workspace --no-deps --locked
git diff --check

cd plans/closure/verified-update-core/008-package-fixture
cargo test --offline                                # 3 passed
cargo tree --offline                                # packaged 0.1.2 versions only
```

Reran on the final clean head after this batch's commit (exact `git status --porcelain` empty): package/dry-run/fixture commands confirmed green with identical file counts/sizes (only `.cargo_vcs_info.json` advances to the final commit; crate contents unchanged — `crates/` is byte-identical between the M002a code head and the final head).

Hosted matrix:

- run `36477024102` (M002a head `9a5500e`): Stable checks / MSRV / macOS / Windows all success — the green current-head baseline M008a was blocked on.
- final-head run (this batch: plans + fixture-vendor only over `9a5500e`, no `crates/`/workflow/manifest change): recorded in the registry header; docs/fixture-vendor paths cannot affect Rust/CI lanes, and the run confirms it.

Confirmation that no publication occurred: only `--dry-run` commands were used; no `cargo publish` without `--dry-run` appears in this record; crates.io publication remains a separate maintainer action (core 0.1.2 before archive 0.1.2).

## Invariant review

- workspace/package version remains 0.1.2: untouched.
- no automatic publication: only dry-runs executed.
- `eggup-core` package remains transport/archive independent: tree evidence above.
- `eggup-archive` remains optional and independently packageable: tree + independent dry-run evidence.
- M001d public APIs unchanged: no source edits at all.
- manual publish order remains core before archive: restated in the M008 addendum.
- no git/path dependency accepted as the downstream solution: fixture consumes packaged tarballs only.
- Rust 1.89 supported: MSRV lane + local check green.

## Failure/restart/cancellation review

No runtime semantics. A failed package/dry-run would have stopped for a metadata corrective per the plan; none failed, so no workaround (`--allow-dirty` or otherwise) was restored.

## Compatibility and migration review

No public API migration. 0.1.2 remains unpublished until a separate maintainer action. Downstream Egress M006's cutover gate is now fully qualified pending only that publication.

## Security review

No source change; no new trust boundary, dependency, or audit surface. Package contents inspected (file lists above); no authenticity/signature claims introduced.

## Documentation/operations evidence

- `plans/closure/verified-update-core/008-status.md` amended (M008a addendum with clean numbers + publish order).
- this closure record.
- fixture vendor refreshed from the clean tarballs under `plans/closure/verified-update-core/008-package-fixture/vendor/`.
- roadmap + registry updated (see below).

## Unresolved findings

None at medium-or-higher severity. The provisional-evidence finding is corrected, not merely tracked.

## Roadmap disposition

Verified Update Core M008a is closed. M008's package qualification now rests on exact clean-tree evidence bound to a green hosted head. The only remaining step toward consumer adoption is the maintainer's manual 0.1.2 publication (core, then archive), after which Egress Delivery M003/M006 may perform the final registry dependency cutover. No new Eggup work is unblocked beyond that maintainer action.

## Registry updates

- M008a row: blocked/ready → closed.
- M008 row: evidence corrective open → closed with clean evidence.
- Next handoff: 0.1.2 publication (maintainer action) → Egress Delivery M003/M006 cutover.
