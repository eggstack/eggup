# Consumer Adoption Milestone 005 — CodeGG Managed-Runfile Bundle Adoption

Status: closed

Closure record: `plans/closure/consumer-adoption/005-status.md`.

Eggup repository baseline: `66acd739f792437cb9fa1701b8fa456c4ba4c403`

Consumer evidence baseline: `dbowm91/codegg@220d3638fe043b24e65a7817241543612f9e82af`

Source roadmap:

- `plans/subsystems/consumer-adoption-roadmap.md`

Long-term requirements:

- `plans/000-long-term-specification.md#6-multi-artifact-installation-is-fundamental`
- `plans/000-long-term-specification.md#22-consumer-adoption`
- `plans/002-long-term-roadmap.md#phase-8--multi-artifact-codegg-adoption`

Applicable ADRs:

- `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md`
- `plans/adrs/ADR-0002-multi-artifact-transaction-and-rollback.md`
- `plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md`
- `plans/adrs/ADR-0004-eggpack-producer-eggup-consumer-boundary.md`

Consumer predecessor evidence:

- CodeGG blocked updater plan: `plans/implementation/dependency-security-workspace-consolidation/005-generic-updater-interface-and-codegg-adoption.md`;
- CodeGG blocked closure/gate: `plans/closure/dependency-security-workspace-consolidation/005-status.md`;
- CodeGG managed-runfile bundle contract: `RELEASING.md`, `install.sh`, and the closed self-contained-installation corrective;
- current check-only updater: `src/upgrade/mod.rs` and `architecture/upgrade.md`.

Primary class: capability / compatibility

## 1. Objective

Make CodeGG the first Eggup multi-artifact consumer by replacing its current check-only in-place upgrade limitation with native verified replacement of the managed runfile bundle on the already-supported Linux/macOS prebuilt targets.

CodeGG retains release/version/target/origin policy and its archive layout. Eggup owns bounded acquisition contracts and verified local multi-artifact transaction mechanics. Eggpack remains the producer-side authority; this milestone must not recreate a release-contract generator in Eggup.

The normal `codegg upgrade` path must never fetch or execute `install.sh`.

## 2. Readiness and dependencies

Hard Eggup dependencies are closed:

- verified-update-core M006 package qualification;
- acquisition-transport M004 corrective;
- two independent single-binary consumers;
- one service-aware consumer;
- distribution producer authority has been removed from Eggup.

CodeGG's historical external-interface blocker is now satisfied in principle by Eggup's independently consumable core/acquisition APIs. CodeGG had explicitly refused to copy Gregg's updater or keep a shell/curl execution path; Eggup provides the missing generalized owner.

This milestone does **not** depend on Verified Update Core M007 because CodeGG has no service/post-install rollback policy in its current self-update path. It can use the existing immediate-finalize `ValidatedTransaction::commit()` contract.

Before implementation, refresh CodeGG HEAD. If its release bundle, updater policy, trust profile, or installation sibling contract has materially changed, record the new baseline and re-review this plan.

Operational dependency:

- current Eggup source contains corrections newer than the published 0.1.0 package set;
- qualification MAY use exact path patches or an immutable Eggup git revision;
- do not use a floating branch;
- crates.io publication/versioning is a separate maintainer action and is not authorized by this plan.

## 3. Current evidence

At the reviewed CodeGG baseline:

- `src/upgrade/mod.rs` is intentionally check-only. It queries GitHub release metadata through CodeGG's existing Eggfetch client and prints pinned manual `install.sh` guidance for a newer release.
- The normal path acquires no candidate bytes and performs no replacement.
- The earlier CodeGG M005 blocker explicitly required a generalized external updater with caller-owned release policy, staged verified candidates, safe replacement, and no Gregg/service-manager coupling.
- CodeGG's canonical prebuilt artifact is **one target archive**, not three independent GitHub assets:
  - `codegg-<target>.tar.gz`;
  - global `checksums.txt`.
- Supported installer targets are Linux/macOS x86_64/aarch64. Windows archive production is optional best-effort and is not consumed by the current installer.
- Each supported Unix archive contains exactly the top-level managed runfiles:
  - `codegg`;
  - `codegg-sandbox-helper`;
  - `codegg-eggsearch`;
  - optionally `THIRD-PARTY-NOTICES.txt`.
- CodeGG already depends on `flate2` and `tar`, so a narrow consumer-owned extraction path does not require a second archive stack.
- `codegg-eggsearch` is independently versioned and currently pinned to eggsearch 0.3.9; it must not be forced to report the CodeGG version.
- `codegg-sandbox-helper` uses a product-specific safe identity probe rather than a normal `--version` contract.
- CodeGG's current HTTP policy intentionally uses its existing Eggfetch/Rustls/WebPKI configuration.
- `eggup-eggfetch` currently enables additional native-root/proxy features. Blindly adding it could widen CodeGG's transport/trust feature ownership through Cargo feature unification.
- No `eggup-*` dependency is currently present in CodeGG.

## 4. Invariants

- CodeGG owns release authority, GitHub repository identity, version semantics, target mapping, archive naming, pinned eggsearch version, CLI presentation, and unsupported-platform policy.
- Eggup owns generic bounded acquisition contract, integrity/candidate transaction mechanics, ownership revalidation, multi-member commit/rollback, and recovery receipts.
- Eggpack owns producer distribution contracts, generated manifests/installers/CI, and release construction. No producer authority moves back into Eggup.
- No fetched shell script is executed by normal self-update.
- No external `curl` is required by normal self-update.
- No second HTTP/TLS stack or silent trust-store expansion is introduced.
- The release archive is SHA-256 verified against the exact `checksums.txt` entry before extraction.
- Extraction accepts only the declared CodeGG top-level runfile allowlist and fixed optional notice.
- Symlink, hardlink, device, FIFO, absolute/traversal/nested/duplicate/case-colliding unexpected members fail before transaction preparation.
- All extracted runfiles are candidate-validated according to their own identities before commit.
- A successful result never leaves mixed generations of the managed runfile set.
- Foreign/ambiguous live destinations fail closed.
- Historical single-CodeGG installs may gain the missing managed siblings only through explicit `Absent` creation policy, never by treating arbitrary existing files as owned.
- Windows running-image replacement is not claimed by this milestone.
- The inert/background `autoupdate` configuration is not activated.
- No Cargo/source fallback is introduced by this milestone.

## 5. Scope

### In scope

- add `eggup-core` and `eggup-acquisition` as CodeGG dependencies from an approved immutable source;
- implement a thin CodeGG acquisition adapter over the existing Eggfetch client/profile, unless direct `eggup-eggfetch` adoption is proven feature/trust-equivalent first;
- retain CodeGG's current release metadata/version selection;
- resolve the exact current-platform archive/checksum URLs from CodeGG-owned policy;
- fetch bounded `checksums.txt` and the selected archive;
- select exactly one checksum entry for the expected archive and reject missing/duplicate/malformed entries;
- verify archive SHA-256 before extraction;
- extract the verified archive into private temporary storage using CodeGG's existing `flate2`/`tar` dependencies under a strict allowlist;
- derive per-member SHA-256 digests from the verified extracted snapshot and use them as Eggup transaction integrity requirements;
- construct a multi-member `ArtifactSet` for the managed sibling installation;
- validate CodeGG, sandbox-helper, and eggsearch identities with CodeGG-specific validators;
- use Eggup commit/rollback/recovery semantics;
- wire supported `codegg upgrade` to the verified update path;
- preserve check/manual guidance on unsupported platforms or installations outside the qualified contract;
- delete/retire obsolete check-only blocker prose after CodeGG qualifies;
- record dependency/binary-size/duplicated-code impact.

### Out of scope

- producer ReleaseManifest/Eggpack integration;
- generating or modifying CodeGG release archives;
- replacing `install.sh` for fresh installation;
- changing CodeGG release/tag/version authority;
- generic Eggup archive extraction;
- service lifecycle;
- background automatic update;
- Cargo fallback;
- signing/authenticity;
- Windows live self-replacement;
- publishing Eggup or CodeGG releases.

## 6. Required production changes

### A. Resolve the historical CodeGG updater blocker

At implementation start, reconcile CodeGG's dependency-security M005 planning: Eggup now supplies the generalized updater mechanism that the closure recorded as absent.

Do not rewrite the historical blocked closure. Add current planning/closure evidence in CodeGG showing that the external dependency became available and that this Eggup adoption is the follow-up execution path.

### B. Transport integration without trust drift

Prefer a small CodeGG-owned implementation of `eggup_acquisition::AcquisitionTransport` around CodeGG's existing ordinary Eggfetch client/profile.

The adapter should:

- accept exact `AcquisitionRequest` URLs supplied by CodeGG policy;
- enforce `FetchLimits`;
- return `FetchOutcome::NotFound` only for exact 404 semantics;
- stream artifacts to the requested destination rather than buffer entire archives;
- map TLS/redirect/timeout/5xx/I/O failures to hard typed acquisition failure;
- preserve URL redaction;
- use CodeGG's existing Rustls/WebPKI trust ownership.

Before choosing `eggup-eggfetch` instead, inspect its resolved features. Stop if adoption would silently add native roots, proxy semantics, or another trust/transport authority that CodeGG does not currently own.

This adapter is glue, not a new general HTTP abstraction.

### C. Release selection stays CodeGG-owned

Retain the existing GitHub release authority and current-version comparison unless CodeGG separately changes that policy.

Resolve only CodeGG-supported prebuilt targets. The runtime mapping may reuse an existing CodeGG Rust table or introduce the smallest product-owned mapping needed to mirror its checked-in release contract.

Do not add target/asset mapping to Eggup.

Until Eggpack ReleaseManifest interoperability exists, add deterministic CodeGG-side drift tests against the current release fixture/asset names rather than parsing shell source at runtime.

### D. Verify the archive before extraction

Fetch `checksums.txt` under a small metadata bound.

Strictly locate one entry whose basename equals the selected `codegg-<target>.tar.gz`. Reject:

- missing entry;
- duplicate entry;
- malformed digest;
- path-bearing or control-character filename;
- mismatch to the expected archive basename.

Verify the downloaded archive bytes with Eggup's SHA-256 primitives or an exact equivalent already supplied by `eggup-core`. No extraction or candidate execution occurs before this passes.

This establishes integrity only, not publisher authenticity.

### E. Strict consumer-owned archive extraction

Use the already-present `flate2`/`tar` stack in CodeGG.

Extract only to private temporary storage.

For supported Unix bundles allow exactly:

- `codegg`;
- `codegg-sandbox-helper`;
- `codegg-eggsearch`;
- optional `THIRD-PARTY-NOTICES.txt`.

Reject directories and any undeclared/nested entry rather than trying to sanitize it. Reject absolute paths, `..`, dot components, backslashes used as separators, duplicate names, ASCII-case collisions, symlinks, hardlinks, devices, FIFOs, sparse/special entries, and entries exceeding explicit member/archive bounds.

Require all three runfiles exactly once. The notice may appear at most once.

Do not move this product-specific archive contract into Eggup merely to reduce a small amount of CodeGG code.

### F. Establish transaction integrity continuity

The release checksum binds the verified archive, while Eggup's transaction requires per-member integrity evidence.

After successful strict extraction of the verified archive:

1. hash each extracted regular member;
2. construct each `ArtifactMember` with that digest as its `IntegrityRequirement::Sha256`;
3. let Eggup copy into its private transaction stage and verify those digests;
4. let Eggup re-hash staged bytes under the mutation lock before live mutation.

This is transitive integrity continuity from the externally declared archive digest to the exact extracted snapshot. It must not be described as independent member signatures/authenticity.

### G. Candidate validation

Use a CodeGG-owned composite `CandidateValidator` over Eggup's verified staged paths.

Required checks:

- `codegg --version` identifies CodeGG and exactly matches the selected CodeGG release;
- `codegg-eggsearch --version` identifies eggsearch and matches CodeGG's exact pinned eggsearch version;
- `codegg-sandbox-helper` passes the same safe product-specific helper identity probe used by release verification, without granting it arbitrary target execution;
- optional notice is data and is not executed.

Use Eggup's bounded command runner/helpers where they fit. Product-specific output interpretation remains in CodeGG.

Do **not** use `CrossMemberAgreementValidator` across all runfiles: the eggsearch sidecar intentionally has its own version.

### H. Live destination ownership

The installation domain is the canonical directory containing the running CodeGG executable.

Create an Eggup ownership verifier that classifies each intended destination from CodeGG's installed-sibling contract.

At minimum:

- the current `codegg` destination must resolve to the running installation and identify as CodeGG;
- an existing managed eggsearch sibling must satisfy its CodeGG-owned identity/version expectations;
- an existing sandbox-helper sibling must satisfy its trusted sibling/identity contract;
- an existing unrelated or unclassifiable file at any managed name is `Foreign` or `Unknown`;
- missing managed siblings in the explicitly supported historical single-CodeGG layout are `Absent` and may be created;
- an optional notice destination is handled conservatively and must not overwrite an unrelated file.

Use `AbsentPolicy::AllowCreate` only because this migration intentionally supports adding missing managed siblings. The verifier still decides which destinations are genuinely absent versus foreign.

### I. ArtifactSet and commit

Construct one transaction for the entire managed runfile generation.

Executable members:

- CodeGG;
- sandbox helper;
- managed eggsearch.

Optional data member:

- third-party notice, if present in the verified archive.

Use executable permission intent for runfiles and non-executable/preserved private staging semantics for the notice as supported by the current core API.

Interpret `TransactionReceipt` explicitly:

- `Committed` -> updated;
- `RolledBack` -> update failed, prior coherent installation preserved;
- `RecoveryRequired` -> high-severity actionable error including the real recovery path/evidence;
- ownership conflict -> no live mutation.

Do not flatten `RecoveryRequired` into generic install failure.

### J. CLI behavior

For supported prebuilt Linux/macOS installations with a newer selected release, `codegg upgrade` invokes the native verified path.

Preserve:

- already-current output;
- explicit check/report behavior where supported;
- manual fresh-install guidance for platforms/installations outside the qualified in-place contract;
- `install.sh` as a user-invoked bootstrap path only.

Update tests that currently assert every newer release fails closed with manual `curl | sh` guidance.

Do not wire the existing inert `autoupdate` configuration into background execution.

### K. Platform boundary

Initial in-place replacement scope is the same Linux/macOS target set currently consumed by the prebuilt installer.

Windows remains check-only/manual until Eggup has native live replacement evidence and CodeGG's optional Windows archive is promoted to a supported consumer contract.

Windows compilation/tests must remain green; do not silently route Windows through Unix assumptions.

## 7. Ordered work packages

1. Refresh CodeGG HEAD and reconcile its historical external-updater blocker against the exact Eggup API/source being adopted.
2. Record before-state dependencies, updater code ownership, trust-profile features, binary size, and current upgrade behavior.
3. Add immutable Eggup core/acquisition dependencies and the CodeGG Eggfetch acquisition adapter.
4. Implement exact release-asset/checksum selection with deterministic local fixtures.
5. Implement archive checksum verification and strict allowlisted extraction.
6. Build the per-member Eggup `ArtifactSet` with digest continuity and product-specific candidate validation.
7. Implement live ownership classification and transactional commit/receipt mapping.
8. Wire the supported CLI upgrade path while retaining unsupported-platform/manual behavior.
9. Delete superseded local updater mechanics/prose that Eggup now owns; keep product policy and fresh-install tooling.
10. Run CodeGG focused/broad verification, release-tool regressions, Rust 1.89, and supported hosted CI.
11. Write Eggup consumer-adoption closure and update CodeGG planning/docs to close its external updater blocker truthfully.

## 8. Failure, restart, and contention semantics

- release metadata failure -> no candidate acquisition/mutation;
- checksum manifest missing/malformed/duplicate -> no extraction/mutation;
- archive download failure/timeout/cancel -> no extraction/mutation;
- archive SHA mismatch -> no extraction/mutation;
- extraction violation -> no candidate execution/mutation;
- candidate identity/version/helper-probe failure -> no live mutation;
- ownership Foreign/Unknown -> no live mutation;
- lock contention -> no live mutation;
- commit failure -> use Eggup rollback result;
- rollback failure -> surface `RecoveryRequired` and retained evidence;
- successful commit -> all managed runfiles belong to the same downloaded archive generation;
- no service restart behavior is introduced;
- unsupported platform -> preserve check/manual path, never unsafe fallback;
- no source/Cargo fallback is added.

## 9. Compatibility and migration

Preserve the current managed sibling names and release archive contract.

Support the historical single-CodeGG installed layout only when the current executable is positively identified and the missing sibling destinations are truly absent. Do not overwrite arbitrary sibling files merely to migrate.

Fresh installation remains owned by CodeGG's bootstrap installer until producer-side Eggpack work replaces or generates it in a separate project.

The future Eggpack ReleaseManifest/Eggup adapter may later remove duplicate runtime target/checksum mapping. This milestone must not wait for that future seam or invent its own generic producer manifest.

Dependency source/version must be explicit in CodeGG closure. A future crates.io switch is operational/package maintenance, not reason to weaken immutable qualification now.

## 10. Required tests

Release policy/acquisition:

- already current;
- newer release on each supported target;
- unknown/unsupported target;
- metadata 404/5xx/TLS/timeout/redirect behavior;
- bounded checksum metadata;
- bounded archive artifact;
- cancellation and partial download cleanup;
- URL redaction.

Checksum/archive:

- exact checksum entry success;
- missing/duplicate/wrong basename/malformed digest;
- archive digest mismatch;
- exact three-member bundle;
- optional notice;
- missing required member;
- extra member;
- duplicate/case-collision;
- nested/traversal/absolute/backslash entry;
- symlink/hardlink/device/FIFO/special member;
- oversized member/archive;
- extraction leaves only private controlled temporary files.

Candidate validation:

- exact CodeGG version;
- wrong CodeGG identity/version;
- exact pinned eggsearch identity/version;
- wrong eggsearch identity/version;
- sandbox-helper safe identity success/failure;
- bounded timeout/output where execution is used;
- notice never executes.

Ownership/transaction:

- fully managed current bundle -> replacement succeeds;
- historical single-CodeGG layout -> absent siblings created;
- unrelated sibling collision -> fail closed;
- ambiguous current executable/destination -> fail closed;
- successful multi-member commit;
- injected partial commit -> old generation restored;
- recovery-required mapping retains real evidence;
- lock contention;
- executable permissions.

CLI/repository:

- `codegg upgrade` supported-path success fixture;
- check/manual behavior on unsupported platform;
- no production shell/curl execution in `src/upgrade`;
- existing installer/release-tool tests remain green;
- clean-host sibling resolution remains green;
- Windows compile/check remains green without enabling Windows in-place mutation;
- CodeGG's existing Eggfetch trust/features remain unchanged unless closure explicitly proves an intentional equivalent change.

## 11. Required verification commands

At minimum in CodeGG, adapt to the repository's current documented commands at implementation time:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --test upgrade --locked -- --test-threads=1
cargo test --workspace --locked
cargo +1.89.0 check --workspace --all-targets --locked
cargo tree -i eggup-core --locked
cargo tree -i eggup-acquisition --locked
cargo tree -d --locked
scripts/release/test-release-tools.sh
scripts/release/test-installer.sh
scripts/verify.sh quick
git diff --check
```

Also run the self-contained-installation/clean-host and archive-security fixtures owned by the current CodeGG baseline. Record hosted Linux/macOS/Windows results separately.

If any generic Eggup code changes are required, rerun the full Eggup workspace qualification and create a separate Eggup corrective rather than hiding the change in consumer code.

## 12. Documentation updates

CodeGG:

- `architecture/upgrade.md`;
- README upgrade/install sections;
- dependency/security roadmap + registry;
- the historical M005 blocker gets a follow-up disposition without rewriting its prior closure;
- changelog;
- release/install documentation only where runtime self-update behavior changed.

Eggup:

- consumer-adoption roadmap;
- registry;
- M005 closure record.

Documentation must continue to distinguish bootstrap installer behavior from normal in-process update.

## 13. Acceptance criteria

M005 closes only when supported Linux/macOS CodeGG installations can acquire the exact selected managed archive through bounded native Rust transport, verify the archive checksum before extraction, strictly extract only the declared runfiles, validate each product-specific candidate identity, atomically/recoverably replace the whole managed sibling generation through Eggup, surface rollback/recovery truthfully, preserve CodeGG release/target policy, preserve the existing Eggfetch trust profile, execute no network-fetched shell/curl path, leave Windows and unsupported platforms on an explicit non-mutating path, pass focused/broad/release-tool/MSRV/hosted CI qualification, and leave no medium-or-higher generic updater finding unresolved.

## 14. Stop conditions

Stop and write a narrow Eggup corrective or consumer re-plan if:

- current Eggup public APIs require copying generic transaction/integrity logic into CodeGG;
- an Eggup transport choice would silently change CodeGG's trust-store/proxy/security policy;
- the current archive cannot be strictly extracted without introducing a new generic unsafe archive abstraction;
- CodeGG's live installation ownership cannot be proven without treating path existence as ownership;
- Windows behavior would need to be falsely claimed as supported;
- producer target/release authority would have to move into Eggup;
- a generic issue affects other Eggup consumers and would otherwise be patched only in CodeGG.

## 15. Closure evidence required

Record:

- exact Eggup and CodeGG implementation SHAs;
- Eggup dependency source/version;
- before/after CodeGG updater ownership;
- external-blocker reconciliation in CodeGG planning;
- transport/trust feature tree;
- target/archive/checksum matrix;
- archive extraction allowlist and negative matrix;
- per-member digest-continuity explanation;
- candidate identity/version matrix;
- live ownership matrix including historical single-binary migration;
- transaction/rollback/recovery results;
- unsupported/Windows disposition;
- deleted duplicated helpers/LOC;
- dependency and release-binary-size delta;
- Rust 1.89 and hosted platform CI;
- full CodeGG release/install regression results;
- any generic Eggup corrective created.

## 16. Handoff notes

This plan is closed before the requested Verified Update Core M007 pass. It did not require post-commit service rollback.

Do not add service lifecycle, Eggpack manifest generation, generic archive extraction, background autoupdate, Cargo fallback, or release publication to this pass.
