# Acquisition Transport M023 — Closure and Verification Record

## Executive finding and status

**Status: closed 2026-11-22; published; registry-verified; hosted-qualified.** M023 published only `eggup-acquisition 0.2.2` and `eggup-eggfetch 0.2.2`, in dependency order, from the same hosted-qualified source SHA. Each crates.io archive is byte-identical to its locally qualified package, its checksum matches registry metadata, and Cargo provenance records the package path and source SHA. The permanent registry-only fixture compiles both new versions alongside the immutable Curl and Service 0.2.0 caret consumers. All five hosted lanes passed on the source SHA after both uploads.

Acquisition remains a zero-dependency, transport-neutral seam with no production source, API, or runtime change. Eggfetch carries the already-qualified M021 `eggfetch-core 0.4.0` implementation update with the same four features, public API, and acquisition policy. No migration is required. Eggpack M010 is now the next publication gate. No Curl, Service, or transport-footprint package was published; no tag, GitHub Release, PR, or prior registry version was changed.

## Source plan, roadmap, and handoff

- Plan: `plans/implementation/acquisition-transport/023-eggup-acquisition-and-eggfetch-0.2.2-publication.md`
- Roadmap: `plans/subsystems/acquisition-transport-roadmap.md`
- Downstream handoff: Eggpack M010; the three exact Eggup 0.2.2 dependencies are now registry-visible and unyanked.
- Registry: `plans/registry.md`
- Applicable ADRs: ADR-0001, ADR-0003, and ADR-0004 (classification only).

## Reviewed baseline and publication source

| Item | Value |
|---|---|
| M023 plan baseline | `3be567e9839b555d7d9d5e6e43f14de4a13c901d` |
| Publication source SHA for both packages | `cf6fcc1706f353a0c5e4c4d3e0071c482993f974` |
| Branch | `m022-workspace-0.2.2-release-readiness` |
| Workspace source version | `0.2.2` |
| Production changes in M023 | None. Acquisition source is unchanged; Eggfetch's source/manifest dependency update was qualified earlier under M021. |
| Candidate hosted CI | Run [44985695980](https://github.com/eggstack/eggup/actions/runs/44985695980), attempt 1; all five lanes passed on the publication SHA. |
| Post-publication hosted CI | Run [44985695980](https://github.com/eggstack/eggup/actions/runs/44985695980), attempt 2; all five lanes passed on the same publication SHA after both uploads. |
| Local gate | `RUST_TEST_THREADS=1 ./scripts/check-local.sh` passed on the publication SHA before upload. |

The later closure, fixture, and documentation commit does not change either immutable package. The downloaded registry archives' `.cargo_vcs_info.json` files both record source SHA `cf6fcc1706f353a0c5e4c4d3e0071c482993f974` and their respective crate path.

## Authorization and pre-upload registry state

The user authorized all actions needed for the selective release. Before each upload, the exact 0.2.2 crates.io version endpoint returned 404; the immediately previous Eggup release was verified `yanked=false`. Before Eggfetch packaging and upload, Acquisition 0.2.2 and eggfetch-core 0.4.0 were also read back as available and unyanked. Each package was uploaded once; no ambiguous upload or retry occurred.

## Acquisition publication and registry readback

| Registry fact | Readback |
|---|---|
| Version | [eggup-acquisition 0.2.2](https://crates.io/api/v1/crates/eggup-acquisition/0.2.2); `yanked=false` |
| Publisher / published at | `dbowm91` / `2026-11-22T04:43:06.858604Z` |
| Registry checksum | `4000935b1d4d384eaba066c9742e70f91d5cb3dd12618ac3aa79d654e2aab865` |
| Local qualified package checksum | Same as registry checksum. |
| Downloaded registry package checksum | Same as registry checksum; direct archive comparison passed. |
| Cargo provenance | `.cargo_vcs_info.json` records SHA `cf6fcc1706f353a0c4e5c4d3e0071c482993f974` and path `crates/eggup-acquisition`. |
| Published normal dependencies | None; [crates.io dependency readback](https://crates.io/api/v1/crates/eggup-acquisition/0.2.2/dependencies) is empty. |
| Package inventory | Seven files: `.cargo_vcs_info.json`, `CHANGELOG.md`, `Cargo.lock`, `Cargo.toml`, `Cargo.toml.orig`, `README.md`, and `src/lib.rs`. |

Acquisition remains the zero-dependency seam. Its package promotion satisfies the exact `=0.2.2` dependency required by Eggpack M010; no source/API/runtime behavior or migration requirement changed.

## Eggfetch publication and registry readback

| Registry fact | Readback |
|---|---|
| Version | [eggup-eggfetch 0.2.2](https://crates.io/api/v1/crates/eggup-eggfetch/0.2.2); `yanked=false` |
| Publisher / published at | `dbowm91` / `2026-11-22T04:44:54.351694Z` |
| Registry checksum | `f683e0c3b377a1713a5af4a64257f785da24f17771a927c07508b0d73ad35df1` |
| Local qualified package checksum | Same as registry checksum. |
| Downloaded registry package checksum | Same as registry checksum; direct archive comparison passed. |
| Cargo provenance | `.cargo_vcs_info.json` records SHA `cf6fcc1706f353a0c4e4c4d3e0071c482993f974` and path `crates/eggup-eggfetch`. |
| Package inventory | Eight files: `.cargo_vcs_info.json`, `CHANGELOG.md`, `Cargo.lock`, `Cargo.toml`, `Cargo.toml.orig`, `README.md`, `examples/eggfetch_fetch.rs`, and `src/lib.rs`. |

The crates.io dependency endpoint reports four normal dependencies:

| Dependency | Requirement | Features / defaults |
|---|---|---|
| eggfetch-core | ^0.4.0 | `http1`, `tls-rustls`, `tls-native-roots`, `proxy`; defaults disabled |
| eggup-acquisition | ^0.2 | default features |
| futures-util | ^0.3 | `alloc`; defaults disabled |
| tokio | ^1 | `rt`, `time`, `fs`, `io-util`, `net`; defaults disabled |

Dev-dependencies remain tokio ^1 and url ^2. The published package has no Curl edge. Its API and acquisition policy are unchanged; only the already-qualified upstream implementation dependency advances from 0.3.0 to 0.4.0.

## Registry-only compatibility fixture

Fixture: `plans/closure/acquisition-transport/023-package-fixture/`. Its independent manifest pins Acquisition/Eggfetch at `=0.2.2` and directly includes published Curl/Service `=0.2.0`. It constructs the public FetchLimits, EggfetchTransport, CurlTransport, and ServiceSpec values without network requests or service operations.

- Unlocked `cargo check` generated the permanent fixture lockfile and compiled the downloaded registry packages; the locked check passed.
- `cargo tree` resolves Acquisition 0.2.2, Eggfetch 0.2.2, Curl 0.2.0, Service 0.2.0, Core 0.2.2, eggfetch-core 0.4.0, and eggfetch-http-connect 0.4.0.
- Cargo metadata identifies the four direct dependencies and every transitive dependency as crates.io registry packages. The fixture root is the only local package; no path, Git, or patch dependency source exists.
- Curl 0.2.0's `^0.2` seam requirement resolves to Acquisition 0.2.2. Service 0.2.0's `^0.2` Core requirement resolves to Core 0.2.2.
- The lockfile records Acquisition checksum `4000935b1d4d384eaba066c9742e70f91d5cb3dd12618ac3aa79d654e2aab865` and Eggfetch checksum `f683e0c3b377a1713a5af4a64257f785da24f17771a927c07508b0d73ad35df1`, matching registry metadata.

## Requirement → evidence matrix

| Requirement | Evidence and result |
|---|---|
| Publish Acquisition first, only once, from the qualified SHA | Cargo reported success once; crates.io confirms version 0.2.2, publisher, timestamp, unyanked state, checksum, and matching Cargo provenance. **Passed.** |
| Keep the seam zero-dependency and transport-neutral | Registry dependency endpoint is empty; source/API/runtime behavior is unchanged. **Passed.** |
| Publish Eggfetch only after Acquisition is registry-visible | Eggfetch package verification downloaded Acquisition 0.2.2 from crates.io before upload; Eggfetch then published once from the same qualified SHA. **Passed.** |
| Preserve Eggfetch's expected upstream features and dependency surface | Registry dependency readback reports eggfetch-core ^0.4.0 with exactly four expected features and defaults disabled; no Curl edge. **Passed.** |
| Prove legacy caret consumers resolve the new 0.2.2 dependency set | Locked fixture builds Curl 0.2.0 against Acquisition 0.2.2 and Service 0.2.0 against Core 0.2.2; metadata contains registry sources only. **Passed.** |
| Qualify both package artifacts and provenance | Local package hashes equal crates.io checksums and downloaded registry archives byte-for-byte; both Cargo provenance records name the same source SHA and correct crate path. **Passed.** |
| Pass hosted qualification before and after publication | Run 44985695980 attempt 1 and post-publication attempt 2 passed Stable, MSRV 1.89, Linux systemd, macOS, and Windows on the exact source SHA. **Passed.** |
| Preserve scope and immutable state | No API/policy changes, migration, version overwrite/yank, Curl/Service/Eggpack/footprint publication, tag change, GitHub Release change, or PR. **Passed.** |

## Commands and results

| Command/check | Result |
|---|---|
| `RUST_TEST_THREADS=1 ./scripts/check-local.sh` | Passed on source SHA `cf6fcc1706f353a0c4e4c4d3e0071c482993f974` before upload. |
| `cargo package -p eggup-acquisition --list --locked` | Passed; seven-file inventory above. |
| `cargo package -p eggup-acquisition --locked` | Passed; package verification succeeded. |
| `cargo publish -p eggup-acquisition --locked --dry-run` | Passed; dry-run stopped before upload. |
| `cargo publish -p eggup-acquisition --locked` | Passed once; Cargo reported publication. |
| `cargo package -p eggup-eggfetch --list --locked` | Passed; eight-file inventory above. |
| `cargo package -p eggup-eggfetch --locked` | Passed; verification compiled against registry Acquisition 0.2.2 and eggfetch-core 0.4.0. |
| `cargo publish -p eggup-eggfetch --locked --dry-run` | Passed; dry-run stopped before upload. |
| `cargo publish -p eggup-eggfetch --locked` | Passed once; Cargo reported publication. |
| Registry API and direct archive comparisons for both packages | Passed; metadata, checksums, byte equality, and Cargo VCS provenance verified. |
| Fixture `cargo fmt --manifest-path .../023-package-fixture/Cargo.toml -- --check` | Passed after one formatting-only correction. |
| Fixture unlocked and locked `cargo check` | Passed; exact registry packages compiled. |
| Fixture `cargo tree --locked` and `cargo metadata --locked --format-version 1` | Passed; registry-only graph and all selected versions verified. |
| GitHub Actions run `44985695980` attempt 1 | Passed all five hosted lanes on the publication SHA. |
| GitHub Actions run `44985695980` attempt 2 | Passed all five lanes after publication on the same SHA. |

## Implementation commits and PRs

M023 is a publication-only milestone; no production implementation commit or pull request was needed. Both packages were built from the already-qualified source commit `cf6fcc1706f353a0c5e4c4d3e0071c482993f974`. The closure and documentation reconciliation are repository records, not changes to either published package.

## Production implementation evidence

- Acquisition source is unchanged from its previous release; the published normal-dependency list is empty.
- Eggfetch carries the previously qualified M021 `eggfetch-core 0.4.0` dependency update; the public API, four explicit features, and acquisition policy are unchanged. The published dependency graph contains no Curl edge.
- No Rust source, API, runtime policy, or installation behavior changed for M023.

## Failure, rollback, and recovery review

Both uploads completed with unambiguous success responses and were read back from crates.io before proceeding. The package checksums, downloaded archive bytes, and Cargo provenance matched the qualified source. No ambiguous upload required retry; no prior package was overwritten or yanked. This publication does not mutate any user's installation state, and no rollback is claimed.

## Security review

No new runtime code, dependency, transport, authenticity, destination, ownership, or service policy was introduced. SHA-256 values here establish package integrity only; they do not establish authenticity or signatures. The registry-only fixture performs no network request or service operation.

## Documentation and operations evidence

Acquisition and Eggfetch changelogs, the root changelog, `docs/releases.md`, architecture summaries, the docs-hygiene skill, the registry, both affected roadmaps, and the M010 handoff plan record the verified publication state. The permanent fixture at `plans/closure/acquisition-transport/023-package-fixture/` has a locked crates.io-only graph and compiles the two new packages alongside published Curl/Service caret consumers.

## Unresolved findings and roadmap disposition

No M023-specific unresolved finding remains. Unrelated workspace findings are outside this milestone and unchanged. M023 is closed in `plans/subsystems/acquisition-transport-roadmap.md`; Eggpack M010 is the next and final package publication in this authorized sequence.

## Registry updates

The registry now records Acquisition 0.2.2 and Eggfetch 0.2.2 as published, unyanked, checksum/provenance verified, from `cf6fcc1706f353a0c5e4c4d3e0071c482993f974`. Acquisition's checksum is `4000935b1d4d384eaba066c9742e70f91d5cb3dd12618ac3aa79d654e2aab865`; Eggfetch's is `f683e0c3b377a1713a5af4a64257f785da24f17771a927c07508b0d73ad35df1`.

## Compatibility, safety, and handoff

- Acquisition is a package-version promotion only. Eggfetch includes M021's implementation dependency update; neither package adds release selection, authenticity, destination, or service policy.
- SHA-256 records package integrity only; it is not authenticity/signature evidence.
- Existing published versions remain immutable and unyanked. Curl 0.2.0 and Service 0.2.0 remain the only publications for those crates. No producer crate/schema was republished.
- No M023-specific unresolved finding remains. Existing unrelated workspace findings are not changed by this publication-only milestone.
- Acquisition M023 is closed. Eggpack M010 may proceed after its own full local/hosted/package gates; it remains the sole package publication in this selective release sequence.
