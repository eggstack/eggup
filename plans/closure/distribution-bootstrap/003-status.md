# Distribution and Bootstrap M003 — Closure and Verification Record

Status: closed

Source plan: `plans/implementation/distribution-bootstrap/003-release-installer-conformance-validators.md`

Source roadmap: `plans/subsystems/distribution-bootstrap-roadmap.md#M003--release-and-installer-conformance-validators`

Implementation baseline: `bd20bd2d33941cef86020a8709aafc76911ef48d`.
Implementation commit: `9941c58d7039410c728860f9e4e382881d4ccf54` (`feat(dist): validate release and installer conformance`).
Hosted CI: [run 35805605834](https://github.com/eggstack/eggup/actions/runs/35805605834), all jobs passed at the implementation SHA.
No crate publication, release generation, or consumer migration was performed.

## Executive finding

Distribution M003 is complete. `eggup-dist` now derives expected release asset/sidecar names and validates caller-supplied release inventories, archive-member lists, and typed runtime/bootstrap observations against contract v1. All validation is deterministic and local. It neither fetches release data nor inspects archives or consumer source code.

`AllowExtras` is the default policy and `Exact` is available for controlled release/archive sets. Inventory constructors reject invalid, exact-duplicate, ASCII-case-colliding, and over-limit entries. Reports are typed, sorted, and bounded. The library API plus checked-in observation fixtures provides sufficient CI value without a CLI.

## Public API inventory

- `expected_release_files(contract, target_or_alias, version)` returns stable field labels and exact flat names for direct, bundle, and archive forms.
- `ReleaseInventory::new` validates and sorts flat release filenames; `validate_release_inventory` reports missing files and, in exact mode, extras.
- `ArchiveMemberInventory::new` reuses the schema v1 traversal rules and portable ASCII-case duplicate rule; `validate_archive_member_inventory` reports missing required members and exact-mode extras without opening an archive.
- `ObservedTargetMapping`, `ObservedTargetAssets`, `ObservedDirectMapping`, and `ObservedArchiveMapping` define a serializable TOML observation of consumer target, asset, sidecar, install, and archive-member mapping facts.
- `validate_observed_mapping` resolves the observed triple/alias through the contract and compares the canonical target and complete mapping.
- `FindingKind`, `ConformanceFinding`, `ConformanceReport`, and `ConformanceReport::into_result` expose stable diagnostics and a simple pass/fail adapter.
- `MAX_OBSERVED_ENTRIES` is 256; reports retain at most 512 findings. Each finding string is bounded. No `eggup-dist` dependency was added to runtime crates.

## Conformance evidence

| Area | Evidence | Result |
|---|---|---|
| Direct release | Complete required files accepted with default extras; missing sidecar and unexpected exact-mode file reported | passed |
| Bundle release | Three-entry CodeGG fixture requires each asset and sidecar; missing first asset reported | passed |
| Archive release | Egress archive asset and sidecar required | passed |
| Archive members | All required members accepted; missing nested member reported; extras allowed/default and rejected in exact mode | passed |
| Inventory safety | Empty/overlong/control/separator names, traversal/absolute/drive/backslash/dot/empty-component paths, exact/case duplicates, and >256 entries rejected | passed |
| Direct mapping | Exact mapping, alias-to-canonical resolution, wrong canonical target, asset, sidecar, and install name | passed |
| Bundle mapping | All CodeGG asset, sidecar, and install names compared as exact sets | passed |
| Archive mapping | Egress archive/sidecar and source-to-install pairs compared; wrong install mapping reported | passed |
| Determinism and bounds | Golden reports sort deterministically; unsafe/over-limit observations produce bounded `InvalidObservation` | passed |
| Golden observations | `observed-simple.toml`, `observed-codegg.toml`, and `observed-egress.toml` all conform to their matching checked-in schema fixtures | passed |
| No hidden I/O/source parsing | Validators consume only typed values; no subprocess, network, archive, filesystem, or source-parser dependency added | passed |

## Policy, failure, and security review

- Missing required assets, sidecars, members, targets, and mapping facts are conformance failures. Target aliases resolve only through the declared contract; no nearest-target guessing occurs.
- `AllowExtras` is `ExtrasPolicy::default()` and permits unrelated release/archive content. `Exact` reports extra release filenames/member paths. Mapping observations always compare every declared fact and do not have a general filtering language.
- Release inventories accept flat filenames only. Archive inventories reuse `validate_member_source`, rejecting absolute/drive paths, backslashes, empty/dot/traversal components, controls, and ASCII-case duplicates.
- Invalid inventories fail at construction. Invalid/over-limit typed observations produce a structured finding before comparison. The report sorts all findings and caps output at 512 entries.
- Integrity and authenticity are outside this validator: no bytes, checksum values, signatures, or trust statements are accepted.
- No shell/PowerShell parser, archive extractor, network client, installer generator, or CLI was introduced. M004 can make a separate evidence-based CLI/generator decision.
- No medium-or-higher conformance ambiguity or security finding remains open.

## CLI and fixture-format decision

No CLI is included. Consumer integration tests already have language-specific access to their runtime tables and installer fixtures; they can deserialize or construct `ObservedTargetMapping` and call the pure library API. A CLI would add file/argument behavior without removing consumer-owned observation extraction.

TOML is the documented checked-in observation fixture format. The existing crate dependency supports the schema and fixtures; no additional format/runtime dependency was added.

## Exact verification commands and results

Environment: Darwin x86_64; stable `rustc 1.98.1` / Cargo 1.98.1; Rust 1.89.0.

Passed at the final source state:

```text
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p eggup-dist --all-targets --all-features --locked # 24 unit + 10 conformance + 4 fixture tests
cargo test --workspace --all-targets --all-features --locked # 182 passed across 12 suites
cargo doc --workspace --no-deps --locked
cargo package -p eggup-dist --locked --allow-dirty
cargo +1.89.0 check --workspace --all-targets --locked
cargo +1.89.0 test -p eggup-dist --all-targets --locked
./scripts/check-local.sh
```

The package dry run packaged and verified 6 crate files (91.0 KiB unpacked, 19.3 KiB compressed). Cargo noted that integration tests are excluded from the published package; they were run separately in stable and Rust 1.89 verification.

Hosted run `35805605834` passed stable formatting, Clippy, workspace tests, and docs; Windows workspace check; Rust 1.89 workspace check; and macOS workspace tests. Runner/action deprecation annotations were informational and did not fail a job.

During the first final parallel workspace run, an existing `eggup-core` test that checks environment clearing failed its unrelated one-second command timeout. The test passed when rerun alone. Its environment-only invocation now uses the runner's default five-second bound and reports command output if it fails; the workspace suite and hosted CI both passed afterward. Production candidate timeout behavior was not changed.

## Compatibility and dependency review

- M001/M002 contract parsing and expansion remain unchanged; all three existing contract fixtures and round-trip tests pass.
- Observations contain no URL, release authority/version policy, fallback command, privilege behavior, shell text, or service policy.
- `eggup-dist` retains its existing serde/TOML dependencies. The other workspace crates do not depend on `eggup-dist`; no runtime dependency was added.
- No CLI/package binary or release workflow was added. The crate remains unpublished.

## Downstream readiness and disposition

Distribution M003 is closed at `9941c58d7039410c728860f9e4e382881d4ccf54` and the closure/roadmap/registry updates are committed separately.

This evidence unblocks Distribution M004 for detailed plan authoring and makes Consumer Adoption CodeGG M005 ready for plan authoring: verified multi-artifact core qualification and the distribution observation contract are both available. Egress M006 remains blocked on an archive update/extraction transaction contract; M003 only validates archive names and mappings. Gregg remains gated on its own footprint measurement. Service M004 Windows SCM remains independently ready and is the next plan in the user's sequential batch.
