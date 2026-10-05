# Eggpack Manifest Interoperability M004a — Closure Record

Status: closed (readiness preflight complete 2026-10-01; at the time of writing, M004 proper remained **blocked** on the proven prerequisites below — no publication occurred and none is authorized by this closure; M004 has since closed, see `plans/closure/eggpack-manifest-interoperability/004-status.md`)

Source plan: `plans/implementation/eggpack-manifest-interoperability/004a-package-api-promotion-readiness-preflight.md`

Roadmap: `plans/subsystems/eggpack-manifest-interoperability-roadmap.md`

## Baselines

- Eggup pre-implementation HEAD: `4c8c0e05a01f8d22fe50b4d616621ee6c32cc9d1` (post-M004a roadmap registration head; the source plan lists `538e3e5605cf3c315c10e5be200c8896de7379b1` — `crates/`, `Cargo.toml`, `Cargo.lock` byte-identical between the two, so no code drift; only `plans/` registration text moved).
- Newest qualified Eggup runtime delta in the tree: M003a implementation `39ff62602e39b14b31f5a8f154905d3343034db7` (hosted run `36890986000` green); `git log 39ff626..HEAD -- crates/` is empty, so no post-M003a runtime change bears on this preflight.
- M003 consumer qualification: `eggstack/eggsact@65c916ba3b0f02916203ec1aad09d7e5c023c278` on Eggup pin `e336b3203183aa84d174e7d7bfa087ce4b61b077` (hosted CI `36902758482` + drift `36902758396` green).
- Workspace version: `0.1.2`; `eggup-eggpack` is `publish = false` with direct deps `eggup-core =0.1.2` (path), `eggup-archive =0.1.2` (path), `eggup-acquisition =0.1.2` (path), `eggpack-manifest =0.1.0` (immutable Git rev `678bbf04f5a02827003a1d9ab83ba4f0e6360e41`, package `eggpack-manifest`), `sha2 0.10.9`.
- Eggpack pinned manifest interface: `678bbf04f5a02827003a1d9ab83ba4f0e6360e41` (`eggpack-manifest v0.1.0`, unchanged since M003). External producer baseline reviewed at authoring: `eggstack/eggpack@0413e806945cfdd31e16bd266b0318efb3afa498` (package metadata present, no `publish = false`; Eggpack's own interoperability roadmap stale, no manifest-publication milestone). Registry absence below is proven mechanically via crates.io API (404), not inferred from metadata.

## Executive finding

M004a answers all seven preflight questions mechanically, without publishing anything and without committing any manifest change:

1. `eggpack-manifest 0.1.0` is **not registry-resolvable** (crates.io API 404 "crate does not exist") — hard Eggpack-owned prerequisite.
2. Of `eggup-eggpack 0.1.2`'s Eggup dependencies, **only** `eggup-core 0.1.2` and `eggup-archive 0.1.2` are on the registry; `eggup-acquisition 0.1.2` is absent (max published `0.1.1`).
3. The minimum Eggup publication set is **`eggup-acquisition 0.1.2` → `eggup-eggfetch 0.1.2` → `eggup-eggpack 0.1.2`**, after the Eggpack prerequisite. Core/archive need no republish; no service/curl/footprint crate is required.
4. No coherent registry-only Eggsact-shaped graph exists today: the adapter-only probe fails (no `eggup-eggpack` on the registry) and the Eggsact-shaped probe fails on `eggup-acquisition =0.1.2` (candidates `0.1.1`, `0.1.0` only). A published-pair probe **without** the adapter (`core 0.1.2` + `acquisition 0.1.1` + `eggfetch 0.1.1` + `archive 0.1.2`) resolves with one source per package, proving the registry is coherent today and the adapter's exact `=0.1.2` acquisition requirement is what breaks it.
5. A new **`eggup-eggfetch 0.1.2` publication IS required** — proven by a compile failure, not assumed: published `eggfetch 0.1.1` source expects `FetchLimits::max_artifact_bytes: Option<u64>` while workspace `acquisition 0.1.2` provides `u64`; a compat probe (0.1.1 source + 0.1.2 seam) fails with `E0308`. The workspace `0.1.2` source already carries the migration. Since published `eggfetch 0.1.1` declares `eggup-acquisition ^0.1.0`, any fresh resolve after an `acquisition 0.1.2` publication would select the incompatible pair.
6. **No public API/version adjustment is required before promotion.** The adapter API is additive-stable (M003a wrappers proven byte-for-value), rustdoc is clean, package contents are minimal, and the only manifest change M004 needs is the mechanical Git→registry dependency swap plus `publish = false` removal. Missing package-metadata polish (below) is not a packaging blocker.
7. The producer-side action — publishing `eggpack-manifest 0.1.0` and reconciling Eggpack's stale interoperability roadmap — **belongs in Eggpack**, not Eggup. No vendoring, duplication, or Eggpack edit occurred here.

M004 proper therefore stays blocked. The expected path from the source plan is confirmed with one correction: the Eggup publication set is three crates, not two (`eggfetch 0.1.2` is now proven required).

## Requirement-to-evidence matrix (source plan §6/§12)

| Requirement | Evidence | Result |
|---|---|---|
| registry availability matrix for every exact candidate version (§6.1) | crates.io API `GET /api/v1/crates/<name>` 2026-10-01 + `cargo search` cross-check (see matrix below) | passed |
| adapter package graph simulation in scratch only (§6.2) | standalone candidate (`publish` removed, Git→registry `=0.1.0`) → `cargo package --no-verify` exit 101 "no matching package `eggpack-manifest`, location searched crates.io"; committed `cargo package -p eggup-eggpack --no-verify` fails identically on the Git dep; `cargo package --list` (19 files) passes as a contents proof only | passed |
| minimum Eggup publication set proven, not assumed (§6.3) | graph simulation + Eggsact-shaped probe failure on `acquisition =0.1.2` + published-pair coherence proof + eggfetch incompatibility proof → `acquisition 0.1.2` → `eggfetch 0.1.2` → `eggpack 0.1.2`; core/archive excluded (already published, byte-identical sources) | passed |
| registry-only consumer simulations, adapter-only + Eggsact-shaped (§6.4) | adapter-only probe (`eggup-eggpack =0.1.2`) → "no matching package"; Eggsact-shaped probe (`core/acquisition/eggpack =0.1.2` + `eggfetch =0.1.1`) → "failed to select version for `eggup-acquisition =0.1.2`, candidates 0.1.1/0.1.0"; published-pair probe without adapter resolves with one source per package (tree recorded) | passed |
| `eggup-eggfetch 0.1.2` decision by external resolution test (§6.4B) | published `0.1.1` requires `acquisition ^0.1.0` (registry dependency API) but its source uses `Option<u64>`; compat probe against workspace `0.1.2` seam fails `E0308`; workspace `0.1.2` source migrated — new publication required | passed |
| API/package review (§6.5) | metadata gap analysis, rustdoc clean, 19-file contents, `cargo tree` dep footprint, MSRV `1.89` check green, license/readme present, no Git/path in candidate, no producer crates in graph | passed |
| Eggpack producer handoff recorded, not executed (§6.6) | `eggpack-manifest` 404 + stale-roadmap drift note; exact package/version specified; no Eggpack edit, no vendoring | passed |
| no package published; scratch removed | `cargo publish` never run; scratch lived under the temp dir only, outside the repo; `git status --short` shows only `plans/` paths | passed |
| roadmap/registry tell the truth about M004 readiness | M004a row closed with this closure link; M004 rows blocked on the proven set (see Registry updates) | passed |

## Registry availability matrix (2026-10-01, crates.io)

| Package + exact version | Source packageable | Registry state | Evidence |
|---|---|---|---|
| `eggpack-manifest =0.1.0` | yes in principle (plan baseline: metadata present, no `publish=false`, deps only `serde`+`serde_json`) | **absent** — crate does not exist | `GET /api/v1/crates/eggpack-manifest` → HTTP 404 `{"errors":[{"detail":"crate \`eggpack-manifest\` does not exist"}]}`; `cargo search eggpack-manifest` → no matches |
| `eggup-core =0.1.2` | yes | **present, not yanked** | API versions `['0.1.2','0.1.1','0.1.0']`, yanked all false, max `0.1.2`; `cargo search` shows `0.1.2` |
| `eggup-archive =0.1.2` | yes | **present, not yanked** | API versions `['0.1.2']`, max `0.1.2`; `cargo search` shows `0.1.2` |
| `eggup-acquisition =0.1.2` | yes (`cargo package --no-verify` clean, 6 files) | **absent** — max published `0.1.1` | API versions `['0.1.1','0.1.0']`, max `0.1.1`; `cargo search` latest `0.1.1` |
| `eggup-eggfetch` (published line) | yes (`cargo package --no-verify` clean, 7 files) | `0.1.1` + `0.1.0` present; **`0.1.2` absent** | API versions `['0.1.1','0.1.0']`, max `0.1.1`; published `0.1.1` deps include `eggup-acquisition ^0.1.0` (registry dependency API) |
| `eggup-eggpack =0.1.2` | no as-committed (`publish=false` + Git dep) | **absent** (unpublished by design) | adapter-only probe `eggup-eggpack =0.1.2` → "no matching package named `eggup-eggpack`, location searched crates.io" |
| `sha2 0.10.9`, `serde/serde_json`, `flate2/tar/zip` etc. | — | resolvable (workspace lockfile + registry probes resolve) | published-pair probe locks 140 packages cleanly |

Distinguisher used throughout: *source packageable* (local `cargo package` passes) ≠ *present on crates.io* (API/search/probe resolves) ≠ *exact version resolvable* (probe with `=` requirement locks). No presence was inferred from Cargo metadata.

## Package-list / dry-run results

All run on the committed tree (no manifest edits) except the scratch candidate noted separately:

```text
cargo package -p eggup-core --locked --no-verify
  → Packaging eggup-core v0.1.2 … Packaged 19 files, 180.3KiB (35.5KiB compressed) — pass
cargo package -p eggup-archive --locked --no-verify
  → Packaging eggup-archive v0.1.2 … Packaged 7 files, 137.5KiB (26.5KiB compressed) — pass
cargo package -p eggup-acquisition --locked --no-verify
  → Packaging eggup-acquisition v0.1.2 … Packaged 6 files, 75.8KiB (16.3KiB compressed) — pass
cargo package -p eggup-eggfetch --locked --no-verify
  → Packaging eggup-eggfetch v0.1.2 … Packaged 7 files, 75.8KiB (19.0KiB compressed) — pass
  (note: one warning — package `yoke-derive v0.8.3` in Cargo.lock is yanked in registry crates-io;
   pre-existing lockfile/registry drift, unrelated to Eggup packaging; recorded as info below)
cargo package -p eggup-eggpack --locked --list
  → 19 files (.cargo_vcs_info.json, Cargo.lock, Cargo.toml, Cargo.toml.orig, README.md, src/lib.rs,
     tests/archive_handoff.rs, tests/caller_destinations.rs, 8 fixtures incl. fixtures/README.md,
     tests/interoperability.rs) — pass (contents proof only)
cargo package -p eggup-eggpack --locked --no-verify   (committed Git-pinned manifest)
  → error: failed to prepare local package for uploading
    Caused by: no matching package named `eggpack-manifest` found, location searched: crates.io index
    required by package `eggup-eggpack v0.1.2` — fails as expected (Git deps are not publishable)
```

## Scratch candidate manifest diff (uncommitted, removed after evidence)

Committed `crates/eggup-eggpack/Cargo.toml` → scratch standalone candidate:

```diff
-[package] name="eggup-eggpack" version.workspace/edition/rust-version/license/repository + publish=false
+[package] name="eggup-eggpack" version="0.1.2" edition="2021" rust-version="1.89" license="MIT"
+           repository="https://github.com/eggstack/eggup"   (workspace values materialized; publish removed)
-eggup-core = { version="=0.1.2", path="../eggup-core" }  →  eggup-core = { version="=0.1.2" }
-eggup-archive = { version="=0.1.2", path="../eggup-archive" }  →  eggup-archive = { version="=0.1.2" }
-eggup-acquisition = { version="=0.1.2", path="../eggup-acquisition" }  →  eggup-acquisition = { version="=0.1.2" }
-eggpack-manifest = { version="=0.1.0", git="https://github.com/eggstack/eggpack.git", rev="678bbf04…", package="eggpack-manifest" }
+eggpack-manifest = { version="=0.1.0" }
 sha2 = "0.10.9"  (unchanged)
```

Result:

```text
cargo package --locked --no-verify --manifest-path <scratch>/candidate/Cargo.toml
  → Packaging eggup-eggpack v0.1.2 … Updating crates.io index
    error: failed to prepare local package for uploading
    Caused by: no matching package named `eggpack-manifest` found, location searched: crates.io index
    exit 101 — branch stopped cleanly per plan §6.2 (producer-side prerequisite; no vendoring)
```

No scratch edit was committed; the scratch tree lived outside the repo and was removed after evidence collection.

## Minimum Eggup publication set (proven)

```
Eggpack publishes eggpack-manifest 0.1.0   (external, Eggpack-owned)
   → Eggup publishes eggup-acquisition 0.1.2   (absent; packageable today)
   → Eggup publishes eggup-eggfetch 0.1.2      (absent; required — see below)
   → Eggup publishes eggup-eggpack 0.1.2       (absent; after the above + manifest)
```

- `eggup-core 0.1.2` / `eggup-archive 0.1.2`: already published; `git diff e8e07eb..HEAD -- crates/eggup-core crates/eggup-archive` empty — no republish.
- `eggup-service`, `eggup-curl`, `eggup-transport-footprint`: no edge in the candidate graph (`cargo tree -p eggup-eggpack` shows only core/archive/acquisition/manifest/sha2 + transitive); publication not required and explicitly out of scope.
- Ordering is load-bearing: `eggfetch 0.1.2` depends on `acquisition ^0.1.0` (satisfied by `0.1.2` once published); `eggpack 0.1.2` pins all three at `=0.1.2` plus manifest `=0.1.0`.

## Registry-only consumer simulations (temporary external projects, removed after)

A. Adapter-only (`eggup-eggpack =0.1.2`, no path/Git):

```text
cargo generate-lockfile  →  error: no matching package named `eggup-eggpack` found, location searched: crates.io index — exit 101
```

B. Eggsact-shaped (`eggup-core =0.1.2`, `eggup-acquisition =0.1.2`, `eggup-eggfetch =0.1.1`, `eggup-eggpack =0.1.2`, no Git/path):

```text
cargo generate-lockfile  →  error: failed to select a version for `eggup-acquisition = "=0.1.2"`
  candidate versions found which didn't match: 0.1.1, 0.1.0; location searched: crates.io index — exit 101
```

C. Published-pair control (no adapter: `core =0.1.2`, `acquisition =0.1.1`, `eggfetch =0.1.1`, `archive =0.1.2`):

```text
cargo generate-lockfile  →  Locking 140 packages … success — exit 0
cargo tree (head)  →  published-pair-probe
  ├── eggup-acquisition v0.1.1
  ├── eggup-archive v0.1.2 … ├── eggup-core v0.1.2 … └── eggup-eggfetch v0.1.1 …
  one source identity per package, no git/path Eggup edge — the registry is coherent today;
  the adapter's exact =0.1.2 acquisition requirement is what breaks the Eggsact-shaped graph.
```

No Eggsact source was modified for these simulations.

## `eggup-eggfetch 0.1.2` publication decision and evidence

**Decision: a new `eggup-eggfetch 0.1.2` publication IS required for downstream coherence.**

- Published `eggup-eggfetch 0.1.1` registry deps (dependency API): `eggfetch-core ^0.2.0`, **`eggup-acquisition ^0.1.0`**, `futures-util ^0.3`, `tokio ^1` — so a fresh resolve after `acquisition 0.1.2` publishes would select `acquisition 0.1.2` for the existing `eggfetch 0.1.1`.
- But published `0.1.1` source is incompatible with the `0.1.2` seam: `src/lib.rs:384 if let Some(max) = max_artifact` plus `max_artifact_bytes: Some(256*1024)` (×3 test sites) expect `Option<u64>`; workspace `eggup-acquisition 0.1.2` declares `pub max_artifact_bytes: u64` (finite-bounds corrective; `None` no longer representable; migration: `Some(n)` → `n`).
- Mechanical proof — compat probe (published `0.1.1` lib source + `eggup-acquisition` path dep at workspace `0.1.2`):
  ```text
  cargo check → error[E0308]: mismatched types --> src/lib.rs:384:28
    if let Some(max) = max_artifact {  expected `u64`, found `Option<_>` — exit 101
  ```
- Workspace `eggup-eggfetch 0.1.2` source already carries the migration (direct `u64` comparison, `max_artifact_bytes: 256*1024`, UTF-8-safe `bound()`, `FetchLimits::new(1024,1024,…)`), plus the `Unavailable`/composition API the `0.1.2` seam qualifies. `git diff v0.1.1..HEAD -- crates/eggup-eggfetch crates/eggup-acquisition` records the full delta; `crates/` has no post-`e8e07eb` delta beyond M003a's `eggpack` change, so the `0.1.2` sources are the publish candidates.
- Therefore the Eggsact-shaped registry graph cannot stay on `eggfetch 0.1.1` once `acquisition 0.1.2` exists. `eggfetch 0.1.2` joins the minimum publication set. This corrects the "possibly no others" expectation in source plan §6.3 — the plan explicitly required proving, not assuming, the set, and the proof ran.

## API / package metadata findings

- Package metadata: `eggup-eggpack` currently sets `description` + `readme` + workspace `license`/`repository`, but **lacks** `homepage` / `documentation` / `authors` / `keywords` / `categories` that `eggup-core`, `eggup-acquisition`, and `eggup-eggfetch` all carry. Verdict: **polish, not a blocker** — `cargo package --list` passes and the only `cargo package` failure is the missing registry dependency, not metadata validation. Recommend backfilling the five fields in the eventual M004 plan for crates.io discoverability; no API change needed for it.
- Public rustdoc: complete for the promotion surface (`default_destinations`, `materialize_artifact_set_with_destinations`, `core_plan_for_archive_with_destinations` + delegation notes on both wrappers + authority-split docs); `cargo doc -p eggup-eggpack --no-deps --locked` clean, no warnings.
- Packaged contents (19 files via `--list`): no fixtures with secrets, no large binaries, no test-only bloat beyond the adapter's compatibility fixtures (all small JSON + `fixtures/README.md` with provenance); `exclude` needs no change.
- No workspace-only assumptions: candidate manifest resolves all workspace inheritance to literals; no Git/path dep remains in the candidate.
- MSRV 1.89: `cargo +1.89.0 check -p eggup-eggpack --all-targets --locked` green.
- License/readme: `MIT` + `README.md` present; README documents the default-vs-caller-bound destination split and the unpublished/Git-pinned status truthfully.
- Dependency surface (`cargo tree -p eggup-eggpack --locked`): `eggup-core`, `eggup-archive`, `eggup-acquisition`, `eggpack-manifest` (pinned rev), `sha2` + dev-only `serde/serde_json/flate2/tar/zip/sha2` — limited to intended crates; no `eggpack-core`/`eggpack-contract`/bootstrap/CI edge; `eggup-core`/`-acquisition`/`-service`/`-archive` show zero `eggpack` edges.
- No public API/version adjustment required: M003a additive API preserved byte-for-byte compatibility (wrapper-equivalence tests); no breaking change, no version bump beyond the existing workspace `0.1.2`.

## Exact verification commands and results (source plan §10)

Local: Darwin arm64, stable toolchain, HEAD `4c8c0e0` (docs-only delta over the M003a-qualified tree):

```text
cargo fmt --all -- --check  → clean, exit 0
cargo check --workspace --all-targets --locked  → Finished dev profile, exit 0
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings  → Finished, exit 0
cargo test -p eggup-eggpack --all-targets --all-features --locked
  → lib 7 passed; archive_handoff 11 passed; caller_destinations 15 passed; interoperability 28 passed (61 total), exit 0
cargo doc -p eggup-eggpack --no-deps --locked  → Generated target/doc/eggup_eggpack/index.html, exit 0
cargo +1.89.0 check -p eggup-eggpack --all-targets --locked  → Finished, exit 0
cargo package -p eggup-core --locked --no-verify  → 19 files 180.3KiB (35.5KiB compressed), exit 0
cargo package -p eggup-archive --locked --no-verify  → 7 files 137.5KiB (26.5KiB compressed), exit 0
cargo package -p eggup-acquisition --locked --no-verify  → 6 files 75.8KiB (16.3KiB compressed), exit 0
cargo package -p eggup-eggfetch --locked --no-verify  → 7 files 75.8KiB (19.0KiB compressed), exit 0
cargo tree -p eggup-eggpack --locked  → one pinned eggpack-manifest edge; lower crates Eggpack-free (full tree in §API above)
git diff --check  → clean
```

Scratch/external probes (exact outputs in §§ above; all temporary, removed): candidate `cargo package` exit 101 (missing `eggpack-manifest`); adapter-only lockfile exit 101 (no `eggup-eggpack`); Eggsact-shaped lockfile exit 101 (`acquisition =0.1.2` unselectable); published-pair lockfile exit 0 + one-source-per-package tree; compat-probe `cargo check` exit 101 (`E0308`).

No `cargo publish` command was run or authorized.

## Invariant / failure / compatibility / security / docs reviews

- Ownership: Eggpack owns `eggpack-manifest` publication; Eggup owns `eggup-eggpack` + support-crate publication; Eggsact owns its Git→registry migration. M004a published, yanked, tagged, or released nothing.
- Package coherence: the promoted graph (once prerequisites clear) has one registry source per package, no path/Git edge, no unrelated-crate publication, no weakened exact `=0.1.2`/`=0.1.0` pinning, and no API change to satisfy packaging.
- Architecture: `eggup-core`/`-acquisition`/`-archive`/`-service` stay Eggpack-independent (tree proof); adapter touches only the lightweight schema crate; integrity stays evidence-only; no producer build/CI/bootstrap crate in the graph.
- Failure: all scratch/external failures are recorded as evidence with exact exit codes and diagnostics; no `--allow-dirty`, vendoring, or bypass was used. Registry state is timestamped 2026-10-01; any mid-execution publication would require re-running the affected probe (none observed — matrix re-queried at the end of the pass).
- Compatibility: Git-pinned M003 Eggsact qualification stays valid and untouched; eventual M004 preserves M003 runtime/API byte-for-byte (no corrective trigger found); Eggsact registry migration stays separately authorized.
- Security: no authenticity claim added; diagnostics in probes echo only package/version identities, never manifest contents or credentials; candidate contents leak nothing.
- Docs: roadmap + registry updated to the M004a-closed/M004-blocked truth (see below); no Eggpack file edited.

## Cross-repo Eggpack prerequisite (Eggpack-owned)

- Required: `eggpack-manifest` version **`0.1.0`** published to crates.io (the exact version the qualified adapter pins by Git rev `678bbf04f5a02827003a1d9ab83ba4f0e6360e41`).
- Provenance: Eggpack source package `0.1.0` with package metadata present and no `publish = false` (authoring baseline `eggstack/eggpack@0413e80`); M003 proved the Git-pinned schema in production consumer use.
- Recommended Eggpack-side follow-up (not authored or executed here): a small manifest-publication plan in the Eggpack repository, plus reconciling Eggpack's stale interoperability roadmap (still reports M003 as ready-to-resume; registers no manifest-publication milestone).
- If Eggpack instead publishes a newer `0.1.x`+ schema, M004 must re-qualify the adapter pin before swapping — not assumed compatible here.

## Unresolved findings

| Severity | Finding | Disposition |
|---|---|---|
| Info | `yoke-derive v0.8.3` in `Cargo.lock` is flagged yanked during `cargo package -p eggup-eggfetch` (registry/lockfile drift). | Pre-existing, unrelated to Eggup packaging (all four `cargo package` runs succeed); refresh the lockfile entry in the eventual M004 plan, not here. |
| Info | `eggup-eggpack` package metadata lacks `homepage`/`documentation`/`authors`/`keywords`/`categories`. | Polish; backfill in M004 (no validation blocker today). |
| Info | Windows/macOS native package runs were not re-executed in this preflight; qualification rests on the M003a hosted matrix (run `36890986000` green on all four lanes) plus local stable/MSRV evidence, which is sufficient for a no-code-change preflight. | Accepted; M004 publication plan must re-run the hosted matrix on the registry-swapped manifest. |
| Info | Eggpack roadmap drift (above). | External, Eggpack-owned. |

No medium-or-higher finding remains. No stop condition triggered: no breaking adapter/core API change is needed, no lower crate needs an Eggpack dep, the adapter needs only `eggpack-manifest` (not producer build/CI crates), no integrity/destination/fallback semantic needs weakening, published core/archive `0.1.2` are compatible with the candidate, and source duplication resolves with ordinary versioning (`acquisition` → `eggfetch` → `eggpack` order).

## Roadmap disposition

- M004a: ready → **closed** (this record; qualification only, zero committed package-graph change).
- M004 package/API promotion: remains **blocked** — now on the proven set: Eggpack-owned `eggpack-manifest 0.1.0` publication + Eggup-owned `eggup-acquisition 0.1.2` → `eggup-eggfetch 0.1.2` → `eggup-eggpack 0.1.2` publications in dependency order, then a separately authorized M004 implementation plan. No M004 plan is authored by this closure.
- No other milestone changes state. In particular, no future plan is unblocked by this closure: M004 stays blocked until every external registry version above is actually resolvable, at which point its implementation plan may be authored — not before.

## Registry updates

- Dependency-ready + planned/blocked tables: M004a ready → closed (this closure link); M004 blocked with the exact prerequisite chain (Eggpack `eggpack-manifest 0.1.0` + Eggup `acquisition`/`eggfetch`/`eggpack` `0.1.2` in order; core/archive already published).
- Execution graph: M004a node closed; M004 decision point gated on M004a evidence + all external registry versions resolvable.
- (Shared C011 pass: header refreshed to the post-C011/M004a closure head with `39ff626` runtime + `65c916b` consumer evidence; stale single-condition M004 prose replaced throughout.)
