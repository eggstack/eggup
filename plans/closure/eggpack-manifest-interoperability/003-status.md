# Eggpack Manifest Interoperability M003 — Execution Status

Disposition: **blocked; real-consumer milestone not closed**

Source plan: `plans/implementation/eggpack-manifest-interoperability/003-eggsact-real-consumer-manifest-adoption.md`

Roadmap: `plans/subsystems/eggpack-manifest-interoperability-roadmap.md`

This record closes the current bounded execution pass and preserves its evidence. It does not claim M003 acceptance: the required normal Eggsact updater path, producer-valid release fixture through commit, and consumer verification matrix remain unimplemented because the plan's producer-evidence gate is not met.

## Baselines

- Eggup pre-change baseline: `fa02c00e5e12e12e2f7e1e768ce624d43082f222`.
- Eggup adapter/API implementation: `cbfa8fa3870dae22d16775408a3135ea682c9365`.
- Eggpack pinned interface baseline: `678bbf04f5a02827003a1d9ab83ba4f0e6360e41`.
- Eggpack current repository head inspected: `00a3399773045121baabbe86f062d41a4c2ce1fb`.
- Eggsact selected consumer baseline inspected: `576f4b0ac09238a42e5561c2da6da8ff4a47bce6`.

## Producer evidence and blocker

At the pinned Eggsact baseline, `.github/workflows/release-binaries.yml` creates these release assets:

| Target | Binary | Checksum sidecar |
|---|---|---|
| `x86_64-unknown-linux-gnu` | `eggsact-x86_64-unknown-linux-gnu` | same name + `.sha256` |
| `aarch64-unknown-linux-gnu` | `eggsact-aarch64-unknown-linux-gnu` | same name + `.sha256` |
| `x86_64-apple-darwin` | `eggsact-x86_64-apple-darwin` | same name + `.sha256` |
| `aarch64-apple-darwin` | `eggsact-aarch64-apple-darwin` | same name + `.sha256` |
| `x86_64-pc-windows-msvc` | `eggsact-x86_64-pc-windows-msvc.exe` | same name + `.sha256` |

The workflow assembles and uploads those binary assets, sidecars, and the installer scripts. It does not construct or upload a ReleaseManifest. `src/update.rs` independently confirms the live unversioned artifact table and currently downloads a sidecar after acquiring a release binary.

Eggpack HEAD's `crates/eggpack-contract/tests/fixtures/simple-direct.toml` is an Eggsact-shaped contract fixture with `{product}-{version}-{target}` artifact names; `observed-simple.toml` records `eggsact-1.2.6-x86_64-unknown-linux-gnu`. This is fixture evidence, not a live Eggsact producer contract. No Eggsact producer contract or adopted manifest artifact URL/name convention was found in the inspected Eggpack interface/workflow evidence. Eggpack's manifest roadmap describes a generic builder/`eggpack-release.json` architecture, but that does not establish which artifact a normal Eggsact release publishes or how Eggsact policy should address it.

Therefore the version-qualified-fixture versus live-unversioned-asset difference remains unresolved by producer authority, and no exact producer-owned manifest URL exists for the updater to fetch. M003 section 3's stop condition applies. No Eggsact production source or dependency was changed, and no manifest filename or release asset policy was invented in Eggup.

## Completed bounded Eggup work

`eggup-eggpack` now exposes:

- `MAX_MANIFEST_BYTES`, directly aliased to Eggpack's `MAX_DOCUMENT_BYTES`;
- `project_json(input, canonical_target)`, which bounds bytes before parsing, rejects invalid UTF-8, delegates schema parsing to `ReleaseManifest::from_json`, delegates target projection to `project`, and maps parser failures to content-free `AdapterError::InvalidManifest`;
- focused tests for valid direct JSON, exact size boundary, oversized input, invalid UTF-8, malformed JSON containing credential-like material, unsupported schema, exact target selection, and diagnostic non-disclosure.

The existing parsed-manifest `project` API and M001a interoperability behavior remain intact. The README documents the JSON consumer flow. No lower Eggup crate changed and no producer/build/CI/publication types were introduced.

## Requirement disposition

| Requirement | Result / evidence |
|---|---|
| Refresh actual Eggsact release names/workflow | Passed; evidence above from Eggsact SHA `576f4b0`. |
| Resolve naming mismatch using producer authority | **Blocked**; fixture uses version-qualified names, live workflow uses unversioned names, and no producer contract adoption resolves it. |
| Establish exact manifest artifact convention | **Blocked**; workflow publishes no manifest and no Eggsact convention is present. |
| Bounded adapter JSON parse/project API | Passed; `crates/eggup-eggpack/src/lib.rs`. |
| Parser/schema errors sanitized | Passed; errors are always `InvalidManifest`, and negative test checks raw fragment/secret absence. |
| M001a interoperability regression matrix | Passed within `cargo test -p eggup-eggpack --all-targets --all-features --locked` (28 interoperability tests). |
| Eggsact immutable Eggup dependency alignment/adoption | Not started; stop condition prevents consumer production edits. |
| Local HTTP release fixture through normal updater, materialization, candidate validation and commit | Not run/not implemented because there is no valid producer-owned manifest convention. |
| Manifest NotFound-only legacy fallback and post-manifest hard-failure truth table | Not implemented in Eggsact; the current updater remains on its existing binary/sidecar flow. |
| Dependency source identity in Eggsact | Not applicable; Eggsact dependency files were not changed. |
| Release binary-size delta | Not applicable/not measured; no Eggsact binary or dependency was changed. |
| Hosted Eggsact checks | Not run; no consumer code changed. |
| Public package or release publication | None. |
| Findings by severity | No new generic Eggup defect identified. The unresolved producer convention is a blocking cross-repository dependency, not an Eggup adapter defect. |

## Eggup verification

All commands below passed on the local macOS ARM64 host unless otherwise stated:

- `rtk cargo fmt --all -- --check`
- `rtk cargo check --workspace --all-targets --locked`
- `rtk cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
- `rtk cargo test -p eggup-eggpack --all-targets --all-features --locked` — 35 passed across unit and interoperability suites (7 unit, 28 integration).
- `rtk cargo test --workspace --all-targets --all-features --locked` — 213 passed across 11 suites.
- `rtk cargo doc --workspace --no-deps --locked`
- `rtk cargo +1.89.0 check --workspace --all-targets --locked`
- `rtk cargo tree -p eggup-eggpack --locked`
- `rtk cargo tree -p eggup-core --locked`
- `rtk cargo tree -p eggup-acquisition --locked`
- `rtk cargo tree -p eggup-service --locked`
- `./scripts/check-local.sh`
- `rtk git diff --check`

Dependency-tree evidence confirms that Eggpack is present only beneath `eggup-eggpack`; `eggup-core`, `eggup-acquisition`, and `eggup-service` have no Eggpack edge. There is exactly one local/path source identity for each Eggup package in this workspace. Eggsact's mixed-source tree was not applicable because dependency alignment was not attempted.

## Authority and compatibility review

The current Eggsact updater authority map remains unchanged: Eggsact owns crates.io stable-version selection, host/target eligibility, live release asset names and URL construction, 404 Cargo fallback, checksum sidecar acquisition/parsing, install destination, candidate identity, current-executable ownership, transaction handling, and CLI behavior. Eggup owns the existing acquisition and verified commit mechanisms. The new adapter helper adds only bounded parsing/projection; it performs no I/O and chooses no URL.

No new legacy fallback behavior was introduced. Existing Eggsact behavior remains: selected binary NotFound can choose Cargo fallback; binary transport failures are hard failures; after a binary is found, missing/invalid checksum evidence is hard failure and does not fall back. The required future manifest truth table is still outstanding and must be implemented with the consumer adoption, including exact manifest NotFound as the sole legacy sidecar entry.

No platform behavior changed. Local verification ran on macOS ARM64. Linux/Windows native or hosted Eggsact evidence was not run and is not inferred. The adapter test suite is host-independent but does not substitute for consumer platform verification.

## Hosted Eggup repository CI

Hosted Eggup CI run `36037573793` at head `da1b4a8048bf863e6a653c25f1ba56bc42f4531b` passed all four configured repository lanes:

- Stable checks;
- Rust 1.89 MSRV check;
- macOS tests; and
- Windows check.

This qualifies the current Eggup repository HEAD reviewed for the bounded adapter/API change. It is not hosted Eggsact consumer verification: hosted Eggsact checks remain unrun, and the M003 producer blocker remains unchanged.

## Future-plan transition

- M004 package/API promotion remains blocked: M003 has not met real-consumer acceptance, and `eggpack-manifest` remains unpublished at the inspected Eggpack head.
- M002 archive extraction remains independently blocked on the Phase 10 extraction contract; this execution supplies no evidence to unblock it.
- No other downstream plan is unblocked by this partial M003 qualification.

Resume M003 only after Eggpack/Eggsact establish a producer-owned Eggsact contract that describes the live artifact names and a manifest publication/addressing convention, then refresh all repository SHAs and implement the normal updater path. See the source plan for the complete acceptance matrix.
