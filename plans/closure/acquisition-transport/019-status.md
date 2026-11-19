# Acquisition Transport M019 — Closure and Verification Record

## Status

**Closed; registry-verified; hosted-qualified.** Exactly `eggup-acquisition 0.2.1` was published to crates.io from the qualified source commit. The registry checksum matches both the locally qualified and independently downloaded artifact; the published dependency list is empty; and the exact-pin registry-only fixture builds against the registry release.

No production Rust source, public API, runtime behavior, manifest dependency, or dependency boundary changed. The publication satisfies Eggpack 0.2.1's exact seam pin. Published curl and eggfetch use compatible `^0.2` requirements and were not republished.

## Source plan and roadmap

- Plan: `plans/implementation/acquisition-transport/019-eggup-acquisition-0.2.1-publication.md`
- Roadmap: `plans/subsystems/acquisition-transport-roadmap.md`
- Registry: `plans/registry.md`
- Predecessor: Acquisition M018, hosted-qualified in run `44155438351`.
- Downstream handoff: Eggpack M009 may proceed now that Archive M008 and Acquisition M019 are closed and their exact `=0.2.1` packages are registry-visible.

## Reviewed baseline and publication source

| Item | Value |
|---|---|
| Plan baseline | `b5a989a72dc185b30cd8c0cb67b5110ed03acd86` |
| Publication source SHA | `5a04f68f60c8c3ad6392036fa314ed2749fa572c` |
| Package version | `0.2.1` (workspace version; no version edit in M019) |
| Publication time | `2026-11-19T19:55:39.979394Z` |
| Publisher | `dbowm91` (David Bowman) |
| Yanked | no |
| Registry SHA-256 | `8832b13c4488a59ec4dc8920aea610ba5a1c50434e9cd1e5209c35e7235d7f7d` |
| Local package SHA-256 | `8832b13c4488a59ec4dc8920aea610ba5a1c50434e9cd1e5209c35e7235d7f7d` |
| Downloaded package SHA-256 | `8832b13c4488a59ec4dc8920aea610ba5a1c50434e9cd1e5209c35e7235d7f7d` |
| Cargo provenance | `.cargo_vcs_info.json` records SHA `5a04f68f60c8c3ad6392036fa314ed2749fa572c` and path `crates/eggup-acquisition` |
| Existing tag | `v0.2.1` remains peeled to Core source commit `b5a989a72dc185b30cd8c0cb67b5110ed03acd86`; no tag was moved or created |

The hosted qualification was run on the exact publication source SHA. The M019 plan and source were already present at that commit; M019 added no production changes.

## Published manifest and package contents

The normalized registry manifest and crates.io dependency endpoint report **zero dependencies**. The independent fixture's Cargo metadata resolves `eggup-acquisition 0.2.1` from `registry+https://github.com/rust-lang/crates.io-index`, with an empty dependency list and no path, Git, or patch edge.

The published `.crate` contains exactly:

- `.cargo_vcs_info.json`
- `CHANGELOG.md`
- `Cargo.lock`
- `Cargo.toml`
- `Cargo.toml.orig`
- `README.md`
- `src/lib.rs`

The compressed registry artifact is 22,044 bytes. The downloaded artifact is byte-identical to the locally qualified package.

## Requirement → evidence matrix

| Requirement | Evidence and result |
|---|---|
| Publish exactly `eggup-acquisition 0.2.1` from the qualified source | Direct crates.io version readback: version `0.2.1`, publisher `dbowm91`, `yanked=false`, created `2026-11-19T19:55:39.979394Z`; Cargo provenance names SHA `5a04f68f60c8c3ad6392036fa314ed2749fa572c`. Passed. |
| Registry artifact equals the qualified package | Registry checksum, local package checksum, and downloaded package checksum all equal `8832b13c4488a59ec4dc8920aea610ba5a1c50434e9cd1e5209c35e7235d7f7d`; `cmp` passed. Passed. |
| Published dependency graph remains empty | crates.io dependency endpoint returns `dependencies: []`; Cargo metadata for the downloaded registry package also reports an empty dependency list. Passed. |
| Package works for an independent registry-only consumer | `plans/closure/acquisition-transport/019-package-fixture` pins `eggup-acquisition = "=0.2.1"`, compiles public request/error types, and its lockfile resolves the exact crate from crates.io. Unlocked/locked checks, `cargo tree`, and `cargo metadata` passed. Passed. |
| Local package and dry-run qualification pass | Package inventory, `cargo package`, and `cargo publish --dry-run` passed on the publication source. Passed. |
| All hosted lanes pass on the publication source | GitHub Actions run [44591516672](https://github.com/eggstack/eggup/actions/runs/44591516672), head SHA `5a04f68f60c8c3ad6392036fa314ed2749fa572c`: all five jobs succeeded. Passed. |
| Existing versions, tags, and compatible adapters remain unchanged | `eggup-acquisition 0.2.0` remains unyanked; `v0.2.1` still peels to Core SHA `b5a989a72dc185b30cd8c0cb67b5110ed03acd86`; published curl and eggfetch `^0.2` requirements already accept 0.2.1. No other crate was published under M019. Passed. |

## Commands and platform evidence

| Check | Result |
|---|---|
| `./scripts/check-local.sh` | Passed on publication source SHA `5a04f68f60c8c3ad6392036fa314ed2749fa572c`: formatting, Clippy, workspace tests, docs, dependency resolution/review, and documentation-truth checks. |
| `cargo package -p eggup-acquisition --list --locked` | Passed; exact seven-file inventory above. |
| `cargo package -p eggup-acquisition --locked` | Passed; packaged sources verified. Cargo emitted a non-failing redundant-homepage warning. |
| `cargo publish --dry-run -p eggup-acquisition --locked` | Passed; Cargo aborted the upload because dry-run was enabled. |
| `cargo owner --list eggup-acquisition` | Passed; publisher identity `dbowm91 (David Bowman)` is listed. |
| `cargo publish -p eggup-acquisition --locked` | Explicitly confirmed immediately before upload; exit 0; Cargo reported `Published eggup-acquisition v0.2.1`. |
| Direct crates.io version / versions / dependencies readback | Passed; 0.2.1 exists, is unyanked, publisher and checksum match, dependency list empty; prior 0.2.0 remains unyanked. |
| Download, `shasum -a 256`, `cmp`, and `tar -xOf ... .cargo_vcs_info.json` | Passed; downloaded bytes match the qualified package and provenance SHA is exact. |
| `cargo check --manifest-path plans/closure/acquisition-transport/019-package-fixture/Cargo.toml` | Passed; Cargo downloaded and compiled registry `eggup-acquisition 0.2.1`. |
| `cargo tree --manifest-path plans/closure/acquisition-transport/019-package-fixture/Cargo.toml --locked` | Passed; only the exact registry dependency is present. |
| Locked fixture check and `cargo metadata --locked --format-version 1` | Passed; metadata reports crates.io as the source and no dependencies for the published package. |
| `./scripts/check-docs-truth.sh` and `git diff --check` | Passed after M019 publication-record reconciliation. |
| `git ls-remote origin refs/tags/v0.2.1 'refs/tags/v0.2.1^{}'` | Passed; annotated tag object `3e6ecae9158f834b66a56cd7a9d8776f22330652` still peels to Core SHA `b5a989a72dc185b30cd8c0cb67b5110ed03acd86`. |

| Hosted lane | Run 44591516672 |
|---|---|
| Stable checks | Passed |
| MSRV check (Rust 1.89) | Passed |
| Linux systemd service qualification | Passed |
| macOS tests | Passed |
| Windows archive, acquisition, and service tests and check | Passed |

## Compatibility, security, and failure review

- The public API, runtime behavior, manifest dependencies, and transport-neutral boundary are unchanged; no direct-consumer migration is required.
- No new code path, transport, execution policy, release-selection policy, authenticity claim, or secret handling was introduced. SHA-256 is package integrity evidence only.
- Publication is irreversible by version. Registry readback and checksum/provenance comparison completed without retry, overwrite, yank, or tag movement.
- No medium-or-higher finding was introduced by M019. The existing low-severity load-sensitive curl test finding remains open and outside this milestone; M019 does not republish curl or eggfetch.

## Documentation and registry disposition

M019 is recorded as closed in its plan, acquisition roadmap, and the authoritative milestone table in `plans/registry.md`. The Acquisition changelog, root changelog, `docs/releases.md`, architecture overview and governance snapshot, documentation-hygiene skill, and existing `v0.2.1` GitHub Release notes now identify the publication. The GitHub Release and `v0.2.1` tag still target Core's original source commit. Archive M008 remains closed; both prerequisites for Eggpack M009 are registry-visible.
