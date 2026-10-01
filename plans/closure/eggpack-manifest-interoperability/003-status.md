# Eggpack Manifest Interoperability M003 — Closure Record

Disposition: **closed** (real-consumer adoption landed and qualified 2026-10-01)

Source plan: `plans/implementation/eggpack-manifest-interoperability/003-eggsact-real-consumer-manifest-adoption.md`

Roadmap: `plans/subsystems/eggpack-manifest-interoperability-roadmap.md`

This record preserves the historical bounded execution pass (§"Historical
bounded pass" and below) and then records the final consumer adoption pass
that closes M003. The historical `Blocked` / `Not started` entries describe
the bounded pass only, not the final disposition.

## Final baselines (consumer adoption pass)

- Eggup pinned revision: `e336b3203183aa84d174e7d7bfa087ce4b61b077` (main at
  M003a closure; contains M003a implementation `39ff626`).
- Eggpack pinned manifest interface: `678bbf04f5a02827003a1d9ab83ba4f0e6360e41`
  (`eggpack-manifest v0.1.0`, unchanged).
- Eggpack reviewed producer head: `56ed7e747fd39e4d6a32a9f1fe3e09dd44355069`
  (manifest schema crate unchanged from the `678bbf04` pin).
- Eggsact pre-change baseline: `75839d2` (clean tree, release binary
  11,810,912 bytes).
- Eggsact implementation: `65c916b` (rebased onto `685fa37`; see M005a
  re-review below).
- No Eggup production source changed in this pass; the pinned Eggup revision
  is the M003a-qualified HEAD.

## M005a re-review (§3.1 preflight)

Between the M003a closure and the M003 push, the Eggsact remote gained two
commits: `f135210` (M005a Windows byte-reproducibility: target-scoped
`/BREPRO` + `/DEBUG:NONE` link flags, one determinism guard test, release
workflow unchanged) and `685fa37` (M005a closure; M005 closes outright). The
M003 implementation was rebased onto `685fa37` and the delta re-reviewed:

- no change to asset names, sidecars, installers, updater mapping, the
  five-target matrix, `release/eggpack/distribution.toml`, or the
  `release-manifest.json` staging convention;
- `src/update.rs` delta is one guard test; updater destination/fallback
  semantics are unchanged;
- producer contract and manifest convention match the §3.1 review, so no
  re-plan was required. Full local verification was re-run after the rebase
  and hosted CI ran on the rebased head.

## Resolved producer convention and naming comparison (§8A, §12)

- Producer authority: `eggstack/eggsact: release/eggpack/distribution.toml`
  (five targets, unversioned `eggsact-{target}` / `eggsact-{target}.exe`
  names, `{asset}.sha256` sidecars, `install` defaults `eggsact` /
  `eggsact.exe`). Unchanged since the §3.1 review.
- Published manifest convention: `release-manifest.json` at the release
  root, proven by the real `v1.2.7` 15-asset release (5 binaries, 5 sidecars,
  manifest, 4 installers; see Eggsact M005 closure). The updater addresses
  `{release-origin}/download/v{version}/release-manifest.json` only after
  Eggsact selects and authorizes the release/tag and origin.
- Historical version-qualified-fixture vs live-unversioned-asset mismatch
  (Eggpack `simple-direct.toml` used `eggsact-1.2.6-{target}`): resolved by
  producer authority, not by renaming. The consumer fixture in this pass uses
  the live unversioned naming (`eggsact-{target}`) with the v1 schema shape
  the adapter qualifies against.
- ARMv7 stays Cargo-fallback-only (absent from the contract and the updater
  table); the manifest path is never consulted for unsupported hosts.

## Before/after updater authority map

Before (baseline `75839d2`): Eggsact owned crates.io version selection,
target/asset policy, release origin, binary + sidecar acquisition, local
checksum parsing/mapping into Eggup inputs, candidate identity, destination
(basename of current exe), ownership proof, transaction handling, CLI mapping,
and Cargo fallback on selected-binary 404 only.

After (`65c916b`): all of the above is retained, plus Eggsact owns manifest
source selection (exact 404 → legacy sidecar compatibility; everything else
hard), product/release/target binding after projection, exact artifact URL
construction under the authorized origin, and caller-bound destination binding
(exact basename of `current_exe`). `eggup-eggpack` owns bounded
parse/project, request tightening, and ArtifactSet materialization with
manifest size/digest evidence. No release discovery, origin construction, or
replacement authorization entered the adapter. The manifest branch performs no
checksum parsing/mapping; `parse_checksum` remains for the legacy branch only.

## Updater code inventory (after)

New in `eggstack/eggsact: src/update.rs` (all else unchanged):

- `MANIFEST_FILE_NAME`, `RELEASE_ORIGIN`, `ARTIFACT_MAX_BYTES` (128 MiB
  finite Eggsact-owned ceiling; replaces the historical
  `max_artifact_bytes: None`, which the current Eggup `FetchLimits` no
  longer accepts), `manifest_url[_for_origin]`,
  `manifest_artifact_url_for_origin`, `origin_asset_url` (origin-threaded;
  production call sites use the single `RELEASE_ORIGIN` const).
- `MetadataFetch::{Present, Absent}` + `eggup_fetch_metadata`; the existing
  `eggup_get_text` is rebased on it with byte-identical messages, so
  crates.io/sidecar behavior is unchanged.
- `fetch_release_manifest`, `resolve_manifest_projection` (product == eggsact,
  release == authorized StableVersion, exact canonical target; content-free
  diagnostics), `ManifestAcquired`,
  `acquire_manifest_artifact` (manifest-backed 404 is hard),
  `prepare_manifest_candidate_async` (exactly one installable artifact;
  bundle/archive forms fail closed), `PreparedUpdate::{Manifest, Legacy}`,
  `prepare_candidate_async` dispatcher (legacy constructed only in the
  `Absent` arm), `prepare_legacy_candidate_async` (moved existing body,
  origin-threaded), `commit_manifest_candidate` (adapter materialization
  with caller destinations + `Executable` intent), `commit_artifact_set`
  (shared commit tail extracted from `commit_candidate`; receipt mapping,
  validators, ownership, and Windows staged path unchanged),
  `is_already_current`.
- 15 new tests covering matrix rows 1–13 and 15–18 (row 14 behaviorally via
  the RolledBack mapping; row ordering in §"Behavior matrix evidence").

## Exact legacy-fallback truth table

| Manifest outcome | Source taken | Evidence |
|---|---|---|
| fetch succeeds, projection/binding/acquisition/materialization succeed | manifest path through commit | e2e tests (rows 2, 17) |
| exact manifest 404 | legacy sidecar path only | row 8 test; single legacy call site guarded by `legacy_construction_lives_only_behind_manifest_absence` |
| product / release / target mismatch | hard error, no legacy | row 3–5 tests; messages contain no legacy reference |
| malformed / unsupported-schema / oversized / non-UTF-8 | hard error, no legacy | row 6 tests |
| timeout / TLS / proxy / 5xx / other transport | hard error, no legacy | row 7 tests (503 fixture, refused connection) |
| valid manifest + artifact 404 | hard error, no Cargo/legacy fallback | row 9 test (`absent from the authorized release`) |
| size / digest / candidate-identity mismatch, ownership conflict | failure before commit, no mutation | rows 10–13 tests |
| unsupported host | existing Cargo path, manifest never consulted | unchanged `target_for_host` → `None` arm |

## Focused end-to-end fixture

Deterministic local HTTP server (existing `serve` harness, no new
production dependency) routes `release-manifest.json`, `.sha256`, and
artifact paths under a test origin. The candidate artifact is a shell script
printing the exact `eggsact {version}\n` identity with empty stderr, served
with manifest-declared size/digest. Tests drive the real
`prepare_candidate_async` → `commit_manifest_candidate` path into a temp
install root with a stale executable (canonical and renamed basenames) and
assert replacement bytes, absence of manifest-default files, and receipt
disposition. No public endpoint is contacted.

## Behavior matrix evidence (M003 §9 rows)

1. already-current → no download: `already_current_predicate_is_exact`;
   `run_async` returns before staging creation (code path unchanged).
2. valid manifest + exact artifact → update: `manifest_path_commits_selected_artifact`
   (`ReplacementOutcome::Complete`, bytes replaced).
3.–5. product/release/target mismatch → hard failure:
   `manifest_binding_rejects_identity_mismatch`.
6. malformed/unsupported/oversized/non-UTF-8 → hard failure:
   `manifest_binding_rejects_unusable_documents`.
7. 5xx/transport → hard failure: `manifest_transport_failures_are_hard`.
8. manifest 404 → legacy sidecar path only:
   `manifest_absence_uses_legacy_sidecar_path` (full legacy verify + commit).
9. valid manifest + artifact 404 → hard, no Cargo fallback:
   `manifest_artifact_absence_is_hard_without_cargo_fallback`.
10. size mismatch → failure before commit (undersize at materialization,
    oversize at tightened acquisition ceiling):
    `manifest_size_mismatch_fails_before_commit`.
11. digest mismatch → failure before commit, stale preserved:
    `manifest_digest_mismatch_fails_before_commit`.
12. wrong candidate identity → failure before commit:
    `manifest_wrong_candidate_identity_fails_before_commit`.
13. ownership conflict (symlinked destination → `Foreign`) → rolled back,
    no mutation: `manifest_ownership_conflict_fails_closed_without_mutation`.
14. rollback vs RecoveryRequired stay distinct: behavioral RolledBack proof
    in row 13 + existing `eggup_receipt_mapping_is_explicit` structural
    guard; RecoveryRequired cannot be forced deterministically (residual).
15. unsupported host → Cargo policy unchanged: existing
    `eggup_fallback_classifier_is_preserved` + unchanged `None` arm.
16. one Eggup source identity, no direct producer crates:
    `eggup_dependency_identities_are_single_git_source` (four `eggup-*`
    deps, one full-SHA rev, no `eggpack-manifest`) and
    `eggup_tree_has_single_source_per_package_and_no_direct_producer_edge`
    (`cargo tree` assertions).
17. renamed executable updated in place:
    `manifest_path_updates_renamed_executable_in_place`.
18. finite caller ceiling tightened to exact size, never widened (plus
    `CallerLimitTooSmall` close-out):
    `manifest_artifact_ceiling_is_tightened_never_widened`.

## Dependency evidence

`cargo tree` (consumer checkout at `65c916b`): `eggup-core`,
`eggup-acquisition`, `eggup-eggfetch`, `eggup-eggpack` all resolve to
`v0.1.2 (https://github.com/eggstack/eggup.git?rev=e336b32…#e336b32)` — one
source identity per package, no crates.io/git duplication.
`eggpack-manifest v0.1.0 (...?rev=678bbf04…#678bbf04)` appears only as a
transitive edge beneath `eggup-eggpack` (with `eggup-archive`, likewise
Eggup-owned); depth-1 tree has no `eggpack-*` edge and `Cargo.toml` has no
direct producer-schema dependency.

## Binary-size impact

Same profile and toolchain (`--release --locked`, rustc 1.89.0):

- before (`75839d2`): 11,810,912 bytes;
- after (`65c916b`, post-rebase rebuild): 11,961,440 bytes;
- delta: +150,528 bytes (+1.27%) for the manifest path, its tests excluded
  (test code is not in the release binary), and the `eggup-eggpack` +
  transitive manifest-schema code.

## Local verification (consumer)

macOS ARM64, consumer checkout at `65c916b`:

- `cargo fmt --all -- --check` — clean.
- `cargo run --locked --features dev-tools --bin generate-docs -- --check` — clean.
- `cargo clippy --locked --all-targets --all-features -- -D warnings` — clean.
- `cargo test --locked --all-features -- --skip parity --test-threads=4` —
  675 + 59 + 14 + 3119 + 51 + 11 passed, 0 failed (parity skips are the
  repo's standing Python-parity exclusion).
- `cargo test --locked --doc` — 11 passed.
- `cargo build --release --locked` — clean (size evidence above).

## Hosted verification (consumer)

- Eggsact CI run `36902758482` on `65c916b`: Linux correctness success
  (fmt, docs check, clippy, full tests, doc tests).
- Release drift guard run `36902758396` on `65c916b`: success (Eggpack
  contract + drift checks unaffected).
- Windows and macOS consumer lanes do not exist for this consumer (CI is
  Linux-only); commit-executing tests are `#[cfg(unix)]` and Windows
  staged-replacement behavior is unchanged but unexercised here — recorded
  as unavailable, not inferred.

## Eggup verification (no production change)

Eggup HEAD is unchanged by this pass (`e336b32`, M003a-qualified). The
pinned adapter was re-verified locally at the pinned revision:
`cargo test -p eggup-eggpack --all-targets --all-features --locked` —
7 + 11 + 15 + 28 = 61 passed. M003a closure (`39ff626`, hosted run
`36890986000` green on Stable/MSRV/macOS/Windows) plus the M001a/M002
regression matrices remain the adapter qualification; this pass adds the
first real consumer on top.

## Authority and compatibility review

- Producer/consumer boundary (§5): Eggpack still owns schema/content/naming;
  Eggup owns acquisition and verified commit plus bounded parse/project;
  Eggsact still owns version selection, origin, fallback, destination,
  candidate identity, CLI. Eggsact has no direct dependency on
  `eggpack-manifest`, `eggpack-core`, `eggpack-contract`, bootstrap, or CI
  machinery.
- Trust/integrity: manifest SHA-256 stays integrity evidence only; HTTPS
  origin trust stays Eggsact transport policy; product/release/target checks
  precede acquisition; a present valid manifest with a missing artifact is a
  hard failure, never a downgrade.
- Fallback: only exact manifest 404 enters legacy; only genuine
  selected-binary 404 (or unsupported host) reaches Cargo — both preserved
  from the prior policy and tested.
- Deployment: candidate validation, current-executable ownership,
  staging/locked revalidation, `Committed`/`RolledBack`/`RecoveryRequired`
  mapping, and Windows staged-replacement scope are unchanged (shared
  `commit_artifact_set` tail).
- Compatibility: additive consumer feature; no manifest-default helper
  semantics changed in Eggup; no migration needed. Legacy sidecar path is
  retained while producer rollout requires it; its removal is out of scope.
- Security: diagnostics stay redacted/content-free (manifest errors name
  expectations, never manifest contents); artifact ceiling is finite and
  tightened, never widened; no authenticity claims; no publication.

## Publication

None. No public release or package publication occurred: `eggup-eggpack`
stays `publish = false`, no Eggsact release was cut with Git dependencies,
and the consumer change is commit `65c916b` on `main` for qualification only.

## Unresolved findings

| Severity | Finding | Disposition |
|---|---|---|
| Low | `RecoveryRequired` mapping is covered structurally but cannot be forced deterministically in the consumer fixture. | Accepted residual; Eggup core qualifies the state machine and the consumer maps all three dispositions distinctly. |
| Info | Windows staged-replacement path is unchanged but unexercised (no Windows consumer lane; commit tests are `#[cfg(unix)]`). | Recorded as unavailable; Unix commit proves the shared tail up to the platform branch. |
| Info | Two Eggup sandbox-only timing flakes (`eggup-core` bounded runner, `eggup-curl` fake-curl) seen during M003a remain unrelated to this pass. | No action; recorded in the M003a closure. |

No medium-or-higher finding remains. M003 meets all §10 acceptance criteria:
the normal updater contains and exercises the `eggup-eggpack` manifest path;
a producer-valid direct manifest flows through real update code to Eggup
acquisition, M003a caller-bound materialization, candidate validation, and
commit; Eggsact never parses ReleaseManifest schema semantics; exact
product/release/target policy stays in Eggsact; manifest size/digest replace
manifest-branch mapping while Eggsact binds the destination; lower Eggup
crates remain Eggpack-independent; no silent downgrade exists on any later
failure; legacy needs exact manifest 404; live asset naming is preserved; no
archive/service/CI/bootstrap/publication/authenticity authority entered the
adapter; size impact is measured (+1.27%).

## Roadmap disposition

- M003 row: ready → closed (consumer `65c916b`, hosted CI `36902758482` +
  drift `36902758396` green).
- M004 package/API promotion becomes the next interoperability decision
  point, subject to a publishable `eggpack-manifest` version; no M004 work
  is authorized by this closure.

## Historical bounded pass

The sections below preserve the original bounded execution pass and its
evidence. They do not describe the final disposition above.

## Baselines

- Eggup pre-change baseline: `fa02c00e5e12e12e2f7e1e768ce624d43082f222`.
- Eggup adapter/API implementation: `cbfa8fa3870dae22d16775408a3135ea682c9365`.
- Eggpack pinned interface baseline: `678bbf04f5a02827003a1d9ab83ba4f0e6360e41`.
- Eggpack current repository head inspected: `00a3399773045121baabbe86f062d41a4c2ce1fb`.
- Eggsact selected consumer baseline inspected: `576f4b0ac09238a42e5561c2da6da8ff4a47bce6`.

## Historical producer evidence and blocker

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

## Producer-gate resolution addendum — 2026-10-01

The external producer condition recorded above is now satisfied without moving producer authority into Eggup.

- Eggpack Ecosystem M001 is closed in `eggstack/eggpack: plans/closure/ecosystem-adoption/001-status.md`.
- Eggsact Distribution M005 is closed in `eggstack/eggsact: plans/closure/distribution-update-release/005-status.md`.
- `eggstack/eggsact: release/eggpack/distribution.toml` is now the producer authority for the real unversioned five-target artifact names, resolving the historical fixture/live-name mismatch.
- The real `v1.2.7` generated release staged a 15-asset set containing five binaries, five checksum sidecars, four installers, and `release-manifest.json`; the maintainer subsequently published that release.
- Current reviewed reconciliation heads are `eggstack/eggpack@57c150f34ddffea34dac03b5fc0a9a0c956b2865` and `eggstack/eggsact@f90e85bc6eb09ea3fb94eab0f07d3407247d6472`.

Therefore the historical stop condition no longer blocks M003. The milestone is **ready to resume**, but it is not closed: Eggsact still has to adopt the manifest path in its normal updater, preserve the manifest-NotFound-only legacy fallback boundary, align immutable Eggup dependency identities for qualification, and complete the end-to-end consumer/hosted verification matrix.

The original blocker analysis and requirement table below remain historical evidence of the bounded pass; their `Blocked` entries describe that pass, not the current execution gate.

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

- M003 is ready to resume under the existing implementation plan. Before consumer edits, refresh the exact current Eggup/Eggsact SHAs and re-confirm the producer contract plus `release-manifest.json` convention.
- M002 archive extraction handoff has subsequently closed with M002a hosted qualification; the historical blocked statement from this bounded pass is superseded by `plans/closure/eggpack-manifest-interoperability/002a-status.md`.
- M004 package/API promotion remains blocked until M003 meets real-consumer acceptance and the publication/versioning decision for the adapter boundary is separately authorized.
- No new Eggpack producer implementation is required merely to resume M003; runtime manifest acquisition/projection and update policy remain Eggup/Eggsact-owned. Any newly discovered producer defect should still be routed back to Eggpack rather than duplicated locally.
