# Closure record — acquisition-transport M016

## Status

**Conditionally closed.**

The coupling the milestone targets is now **recorded**, the two documents that
asserted the opposite are **corrected**, and the regression guard is
**proven red before the correction and green after it**. The condition is the
same one the sibling acquisition M012/M013/M015 closures carry: the **Linux**
and **Windows** lanes and the hosted CI matrix were **not run** at this stage,
and the parent session owns the whole-tree gate. Everything not run is
recorded as **not run**, never inferred from the macOS lane.

One deviation is recorded below as an unresolved finding with an explicit
severity:

- **UF-1 (low)** — `cargo test --workspace --all-targets --all-features
  --locked` and `./scripts/check-local.sh` were **not run**; the parent session
  scoped this session to the three acquisition crates. The three crates' own
  suites, clippy, fmt, doc, and the dependency-boundary review **were** run.

No publication was performed, no tag was moved, no published version was
bumped, and **no `Cargo.toml` was changed**.

## Source plan and roadmap

- Plan: `plans/implementation/acquisition-transport/016-adapter-publication-coupling-corrective.md`
- Roadmap: `plans/subsystems/acquisition-transport-roadmap.md`
- Closed work affected:
  - `plans/implementation/acquisition-transport/013-adapter-assertion-and-failure-typing-corrective.md`
    (the `0.2.0` next-release mandate)
  - `plans/closure/acquisition-transport/013-status.md` (§ "The `0.2.0`
    consequence")
  - `plans/implementation/acquisition-transport/014-adapter-capability-parity-corrective.md`
    (§6.1, §9.3, §6.3 — the superseded consequences)
  - `plans/closure/acquisition-transport/014-status.md` (§ "Publication status")

## Reviewed repository baseline

`bf439f519300354ac14823e74acd48c7995033a6` (branch
`plans/corrective-batch-2026-10-23`).

## Implementation commits / PRs

**None.** The parent session owns all git mutations; this milestone produced
working-tree changes only. No commit, tag, branch, stash, or publication was
made.

## Executive finding

The seam's next release is `eggup-acquisition 0.2.0` and it is
**source-breaking** (M013). `0.2.0` does **not** satisfy the `^0.1.0`
requirement the two adapters declare today, nor the `=0.1.3` pin the leaf
adapter declares. The coupling was recorded nowhere, and the curl adapter's
changelog asserted the **opposite** — that a future republication "would be a
non-breaking `0.1.3`" and that "no consumer migration is required".

That claim was false, and the defect was not merely documentary:

- `cargo package -p eggup-curl --locked --offline` → **exit 101**, 11 compile
  errors, `failed to verify package tarball`.
- `cargo package -p eggup-eggfetch --locked --offline` → **exit 101**, 4
  compile errors, `failed to verify package tarball`.

`cargo package` resolves a dependency from the **registry**, so both adapters
were already being built against the **published** `eggup-acquisition 0.1.3`,
which contains neither `__adapter_io` nor `__adapter_bound_detail`. **Neither
adapter can be published in its current state**, and the seam's `0.2.0`
release makes the requirement widening **mandatory** rather than optional.

This milestone records the coupling, corrects the false consequence, and adds a
guard that fails when a dependent neither accepts the declared next release nor
has the required widening recorded. It does **not** widen a requirement, bump a
version, or publish: the publication milestone owns all of that, and its exact
action list is in the plan's §9.3.

The `User-Agent` grammar defect found alongside it is a documentation defect:
the rustdoc claimed `SP` is "permitted only between two non-`SP` characters",
while the implementation has always permitted a **run** of interior `SP`. RFC
7230 §3.2 permits the run, and the rules that protect the request head are
enforced exactly. The **documentation** was corrected; the accepted value set
is unchanged.

## Requirement → evidence matrix

Every row names the file:line anchor that carries the requirement and the
**exact** command whose result proves it. All commands were run on **macOS
(darwin 25.6.0, arm64)**, Rust 1.89-compatible toolchain, with `--offline` (no
network). Per-platform reporting is in the next section; no platform is
inferred from another.

| # | Requirement (plan §) | Evidence (file:line) | Command and result |
|---|---|---|---|
| 1 | The seam's changelog carries exactly **one** `## Unreleased` section, at the **top**, with the M013 entry (§6.1) | `crates/eggup-acquisition/CHANGELOG.md:3` (the only `## Unreleased`), `:80` (the next heading, `0.1.3`); the stray second heading and its duplicated boilerplate are gone | `cargo test -p eggup-acquisition --locked --offline --test publication_coupling` → `seam_changelog_declares_exactly_one_unreleased_section_with_a_next_release ... ok` |
| 2 | The seam declares its **next release** explicitly (machine-readable) (§6.1) | `crates/eggup-acquisition/CHANGELOG.md:8` — "**Declared next release: `0.2.0`** — source-breaking" | `cargo test -p eggup-acquisition --locked --offline --test publication_coupling` → same test `... ok` (it parses the declared version) |
| 3 | The coupling record names **every** in-workspace dependent and the requirement it must carry (§6.1, §9.3) | `crates/eggup-acquisition/CHANGELOG.md:18` (`eggup-curl`), `:19` (`eggup-eggfetch`), `:20` (`eggup-eggpack` → `=0.2.0`), `:21` (`eggup-transport-footprint`) | `cargo test -p eggup-acquisition --locked --offline --test publication_coupling` → `dependent_requirements_accept_the_next_release_or_record_the_widening ... ok` |
| 4 | Every dependent's declared requirement is read from its own `Cargo.toml` (§6.4) | `crates/eggup-acquisition/tests/publication_coupling.rs:233` (`declared_requirement`), `:45` (the dependent table with `include_str!` of the four manifests) | `cargo test -p eggup-acquisition --locked --offline --test publication_coupling` → `dependent_requirements_accept_the_next_release_or_record_the_widening ... ok` |
| 5 | The guard asserts **either** the requirement accepts the declared next release **or** a changelog records the required widening (§6.4) | `crates/eggup-acquisition/tests/publication_coupling.rs:387` (the test), `:309` (`widening_is_recorded`), `:136` (`Requirement::parse` / `accepts`, Cargo's caret/exact/tilde semantics implemented in-crate) | `cargo test -p eggup-acquisition --locked --offline --test publication_coupling` → `dependent_requirements_accept_the_next_release_or_record_the_widening ... ok` |
| 6 | The curl adapter's changelog states the seam's next release, the mandatory widening, that its own next version is **not** `0.1.3`, and that a consumer inherits the source break (§6.2) | `crates/eggup-curl/CHANGELOG.md:8-37` (the M016 entry): `:11-18` (the correction), `:19-21` (the widening marker), `:22-25` (the version consequence), `:26-32` (the consumer consequence), `:33-37` (what is unchanged) | `cargo test -p eggup-acquisition --locked --offline --test publication_coupling` → `adapter_changelogs_record_the_widening_and_assert_no_contradicted_republication_version ... ok` |
| 7 | The eggfetch adapter's changelog states the same four facts (§6.2) | `crates/eggup-eggfetch/CHANGELOG.md:45-74` (the M016 entry): `:48-55` (the correction), `:56-58` (the widening marker), `:59-62` (the version consequence), `:63-69` (the consumer consequence), `:70-74` (what is unchanged) | `cargo test -p eggup-acquisition --locked --offline --test publication_coupling` → `adapter_changelogs_record_the_widening_and_assert_no_contradicted_republication_version ... ok` |
| 8 | The false "non-breaking `0.1.3`" claim is **gone** from the adapters' changelogs (§6.2) | `crates/eggup-curl/CHANGELOG.md:113-116` (the M014 entry, corrected in place, now pointing at the M016 entry); `grep -n "non-breaking" crates/eggup-*/CHANGELOG.md` → **no match** | `grep -n "non-breaking" crates/eggup-curl/CHANGELOG.md crates/eggup-eggfetch/CHANGELOG.md` → **no output** (exit 1) |
| 9 | The "nothing published / no migration yet" disclaimers stay **accurate** (§6.2) | `crates/eggup-curl/CHANGELOG.md:5-6` (the published-baseline boilerplate), `:37` ("nothing published has changed … performs no publication, no version bump, and no manifest change"); `crates/eggup-eggfetch/CHANGELOG.md:42-43`, `:70-74` | Document review; the per-entry "no consumer migration is required" sentences were narrowed to "by this crate's own API change" and point at the M016 entry |
| 10 | The `User-Agent` rustdoc and the implementation agree; the accepted value set is unchanged (§6.3) | `crates/eggup-curl/src/lib.rs:222-223` (the corrected `SP` line), `:227-233` (the RFC 7230 justification); the implementation at `:295` (`validate_user_agent`) is untouched — it still refuses only a **leading** and a **trailing** `SP` | `cargo test -p eggup-curl --locked --offline` → `33 passed; 0 failed; 1 ignored` (the crate's own `User-Agent` suites unchanged) |
| 11 | The decision is **documented**, not behavioural: no rejection rule, no bound, and no argv change (§6.3) | `git diff` of `crates/eggup-curl/src/lib.rs`: the only hunk is inside the `///` rustdoc of `with_executable_and_user_agent` | `git diff -- crates/eggup-curl/src/lib.rs` → one rustdoc hunk; no code hunk |
| 12 | The publication-time action list is recorded, not performed (§9.3) | `plans/implementation/acquisition-transport/016-adapter-publication-coupling-corrective.md` §9.3 (the five ordered steps) and `crates/eggup-acquisition/CHANGELOG.md:24-31` | `git status --short` → **no `Cargo.toml` is modified** |
| 13 | The guard is covered by the crate's existing vacuity and reason guards (§6.4, §10) | `crates/eggup-acquisition/tests/assertion_guard.rs:23` (the new file joins the scanned sources), `:130-144` (M016's three behaviors join the named witness list) | `cargo test -p eggup-acquisition --locked --offline` → `every_ignored_test_records_a_reason ... ok`, `crate_sources_have_no_vacuous_early_exit ... ok`, `required_behaviors_have_named_asserting_tests ... ok` |
| 14 | The seam keeps **zero** dependencies (§4.2) | `crates/eggup-acquisition/Cargo.toml:23` — an empty `[dependencies]` section; the guard implements the semver comparison itself (`:136-183`) rather than adding a `semver` dev-dependency | `cargo tree -p eggup-acquisition --locked --offline --depth 1` → `eggup-acquisition v0.1.3` with **no dependency line** |
| 15 | The transport-neutrality boundary holds (§4.1) | `crates/eggup-curl/Cargo.toml:23-24` (unchanged); the guard reads the adapters' manifests and changelogs **as text** and never depends on them | `cargo tree -p eggup-curl --locked --offline --depth 1` → `eggup-curl v0.1.3` → `└── eggup-acquisition v0.1.3` (exactly one dependency; no `eggup-eggfetch`) |
| 16 | The M014 plan and closure carry a dated addendum recording the supersession (§12) | `plans/implementation/acquisition-transport/014-adapter-capability-parity-corrective.md` § "Addendum (2026-10-24)"; `plans/closure/acquisition-transport/014-status.md` § "Superseded addendum (2026-10-24, Acquisition Transport M016)" | Document review |
| 17 | The registry and roadmap rows are drafted for the parent and **not applied** (§15.9) | § "Roadmap disposition" and § "Registry updates" below; **this milestone** did not modify `plans/registry.md` or `plans/subsystems/**` | `git diff --stat -- plans/registry.md plans/subsystems/` shows only changes authored by **other** writers in this batch; the M016 rows are in the appendix and are **not** in either file |

## Production implementation evidence

| Change | Location | Notes |
|---|---|---|
| Duplicate `## Unreleased` removed | `crates/eggup-acquisition/CHANGELOG.md` (the stray heading and its duplicated boilerplate, formerly lines 96-99) | The stray section held **no unique content**: its two-line body is the boilerplate the top section already carries verbatim. Nothing was lost. |
| Next-release declaration added | `crates/eggup-acquisition/CHANGELOG.md:8` | `**Declared next release: `0.2.0`** — source-breaking.` Machine-readable, and it agrees with the M013 entry's prose below it. |
| Publication-coupling record added | `crates/eggup-acquisition/CHANGELOG.md:12-31` | Names all four dependents and the requirement each must carry, plus the publication-time action list and the "records, does not perform" statement. |
| M016 entry added to the curl adapter's changelog | `crates/eggup-curl/CHANGELOG.md:8-37` | The four corrected consequences and the widening marker. |
| M014 entry corrected **in place** | `crates/eggup-curl/CHANGELOG.md:113-117` | The false "no consumer migration is required … a future republication would be a non-breaking `0.1.3`" sentence now states that the consequence is **superseded** by the M016 entry. |
| M015 and M013 entries narrowed | `crates/eggup-curl/CHANGELOG.md:73-78`, `:160-163` | "No consumer migration is required **in this crate**" → "**by this crate's own API change**", with a pointer to the M016 entry for the consumer's obligation. |
| M016 entry added to the eggfetch adapter's changelog | `crates/eggup-eggfetch/CHANGELOG.md:45-74` | The same four corrected consequences and the widening marker. |
| M013 entry corrected **in place** | `crates/eggup-eggfetch/CHANGELOG.md:99-102` | Same narrowing as the curl adapter's M013 entry. |
| Stale capability-parity claim corrected **in place** | `crates/eggup-eggfetch/CHANGELOG.md:104-108` | The M013 entry still said "`eggup-curl` exposes none" of the `User-Agent` surface. M014 closed that asymmetry; the text now says so, and records that **placement** parity is deliberately not claimed. This is a pre-existing doc drift in a file this milestone owns, corrected in the same pass. |
| `User-Agent` grammar corrected | `crates/eggup-curl/src/lib.rs:222-223`, `:227-233` | The `SP` line now says a run of `SP` is permitted but a leading or trailing `SP` is not, and the RFC 7230 §3.2 justification is recorded. **Documentation only** — the implementation is untouched. |
| Regression guard added | `crates/eggup-acquisition/tests/publication_coupling.rs` | 3 tests; no dependency; no filesystem discovery, no network, no wall clock. |
| Guard wired into the existing assertion guard | `crates/eggup-acquisition/tests/assertion_guard.rs:23`, `:29`, `:130-144` | The new file joins the scanned sources, and M016's three behaviors join the named witness list. |

**Diff scope.** The change is confined to `crates/eggup-acquisition/**`,
`crates/eggup-curl/CHANGELOG.md`, `crates/eggup-curl/src/lib.rs`,
`crates/eggup-eggfetch/CHANGELOG.md`, and the four `plans/` records this
milestone owns. **No `Cargo.toml` was modified**, no version was bumped, no
public API was added, removed, or retyped, and no existing assertion was
removed or weakened.

## The red-first proof

The guard was written **before** any changelog correction, and its failure was
observed twice — first against the untouched tree, then against the tree with
the next-release declaration in place but the coupling still unrecorded. The
second failure is the one that proves the guard tests the *coupling*.

### Red #1 — the guard against the uncorrected tree

Command:

```sh
cargo test -p eggup-acquisition --locked --offline --test publication_coupling
```

Result: **exit 101**, `0 passed; 3 failed`:

```
test adapter_changelogs_record_the_widening_and_assert_no_contradicted_republication_version ... FAILED
test dependent_requirements_accept_the_next_release_or_record_the_widening ... FAILED
test seam_changelog_declares_exactly_one_unreleased_section_with_a_next_release ... FAILED

---- adapter_changelogs_record_the_widening_and_assert_no_contradicted_republication_version stdout ----
panicked at crates/eggup-acquisition/tests/publication_coupling.rs:428:5:
assertion `left == right` failed: the seam's changelog must carry exactly one `## Unreleased` section, found 2
  left: 2
 right: 1

---- dependent_requirements_accept_the_next_release_or_record_the_widening stdout ----
panicked at crates/eggup-acquisition/tests/publication_coupling.rs:389:5:
assertion `left == right` failed: the seam's changelog must carry exactly one `## Unreleased` section, found 2
  left: 2
 right: 1

---- seam_changelog_declares_exactly_one_unreleased_section_with_a_next_release stdout ----
panicked at crates/eggup-acquisition/tests/publication_coupling.rs:361:5:
assertion `left == right` failed: the seam's changelog must carry exactly one `## Unreleased` section, found 2
  left: 2
 right: 1

test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out
```

### Red #2 — the declaration in place, the coupling still unrecorded

After adding only the next-release declaration (`crates/eggup-acquisition/CHANGELOG.md:8`)
and removing the duplicate heading, the same command was run again. The
declaration test turned **green**, and the two coupling tests failed with the
**coupling** message:

```
test seam_changelog_declares_exactly_one_unreleased_section_with_a_next_release ... ok
test adapter_changelogs_record_the_widening_and_assert_no_contradicted_republication_version ... FAILED
test dependent_requirements_accept_the_next_release_or_record_the_widening ... FAILED

---- adapter_changelogs_record_the_widening_and_assert_no_contradicted_republication_version stdout ----
panicked at crates/eggup-acquisition/tests/publication_coupling.rs:457:9:
eggup-curl's changelog must record the required widening of its `eggup-acquisition` requirement, because `0.2.0` does not satisfy the requirement it declares today

---- dependent_requirements_accept_the_next_release_or_record_the_widening stdout ----
panicked at crates/eggup-acquisition/tests/publication_coupling.rs:413:9:
eggup-curl declares `eggup-acquisition` = `0.1.0`, which does not accept the seam's declared next release `0.2.0`; no changelog records the required widening for it

test result: FAILED. 1 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out
```

### Green — after the changelog corrections

Same command, after the coupling record and the two adapters' corrections:

```
running 3 tests
test seam_changelog_declares_exactly_one_unreleased_section_with_a_next_release ... ok
test dependent_requirements_accept_the_next_release_or_record_the_widening ... ok
test adapter_changelogs_record_the_widening_and_assert_no_contradicted_republication_version ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

**exit 0.**

## Exact commands run and results

All commands were run from `/Users/davidbowman/projects/eggup` with
`--offline` (no network). Platform: macOS (darwin 25.6.0, arm64), Rust
toolchain 1.89-compatible.

| # | Command | Result |
|---|---|---|
| 1 | `cargo fmt -p eggup-acquisition -p eggup-curl -p eggup-eggfetch -- --check` | **exit 0**, no output (after one `cargo fmt` pass applied to the new test file) |
| 2 | `cargo clippy -p eggup-acquisition -p eggup-curl -p eggup-eggfetch --all-targets --locked --offline -- -D warnings` | **exit 0** — `Finished \`dev\` profile … in 1.67s`; no warnings |
| 3 | `cargo test -p eggup-acquisition -p eggup-curl -p eggup-eggfetch --locked --offline` | **exit 0** — `eggup-acquisition`: 38 unit + 3 `assertion_guard` + 3 `publication_coupling` + 5 `typed_io_contract`; `eggup-curl`: 34 unit (33 passed, 1 ignored, pre-existing and reason-recorded) + 3 `assertion_guard`; `eggup-eggfetch`: 25 unit + 3 `assertion_guard` + 2 `typed_io_contract`; 0 doc-tests. **116 tests, 115 passed, 1 ignored, 0 failed.** |
| 4 | `cargo doc -p eggup-acquisition -p eggup-curl -p eggup-eggfetch --no-deps --locked --offline` | **exit 0** — `Generated …/target/doc/eggup_acquisition/index.html and 2 other files`; no rustdoc warnings |
| 5 | `cargo tree -p eggup-acquisition --locked --offline --depth 1` | `eggup-acquisition v0.1.3` with **no dependency line** — zero dependencies, as required |
| 6 | `cargo tree -p eggup-curl --locked --offline --depth 1` | `eggup-curl v0.1.3` → `└── eggup-acquisition v0.1.3` — exactly one dependency; no `eggup-eggfetch`, no HTTP/TLS stack |
| 7 | `CARGO_TARGET_DIR=/tmp/eggup-m016-target cargo package -p eggup-curl --locked --offline --allow-dirty` | **exit 101** — `error: failed to verify package tarball`; 11 compile errors (`E0425: cannot find function '__adapter_bound_detail' in crate 'eggup_acquisition'`, `E0599: no variant or associated item named '__adapter_io'`). The **pre-existing** publication blocker, recorded as evidence — **not** fixed by this milestone |
| 8 | `CARGO_TARGET_DIR=/tmp/eggup-m016-target cargo package -p eggup-eggfetch --locked --offline --allow-dirty` | **exit 101** — `error: failed to verify package tarball`; 4 compile errors of the same two kinds |
| 9 | `CARGO_TARGET_DIR=/tmp/eggup-m016-target cargo package -p eggup-eggpack --locked --offline --allow-dirty` | **exit 0** — `Packaged 20 files`; the leaf adapter's `=0.1.3` pin still resolves to the published `0.1.3`, so its coupling is **latent**, not yet loud |
| 10 | `grep -n "non-breaking" crates/eggup-curl/CHANGELOG.md crates/eggup-eggfetch/CHANGELOG.md` | **no output** (exit 1) — the false claim is gone from both changelogs |
| 11 | `git status --short` | Only this milestone's owned paths are modified; **no `Cargo.toml`** appears |

The `cargo package` runs used `CARGO_TARGET_DIR=/tmp/eggup-m016-target` and
`--allow-dirty`; the packaged artifacts land outside the repository. The
workspace-wide gate (`./scripts/check-local.sh`) is the **parent session's**
responsibility and was **not run** here.

### Test counts

| Target | Tests | Result |
|---|---|---|
| `eggup-acquisition` `unittests src/lib.rs` | 38 | 38 passed, 0 failed |
| `eggup-acquisition` `tests/assertion_guard.rs` | 3 | 3 passed, 0 failed |
| `eggup-acquisition` `tests/publication_coupling.rs` | **3 (added by M016)** | 3 passed, 0 failed |
| `eggup-acquisition` `tests/typed_io_contract.rs` | 5 | 5 passed, 0 failed |
| `eggup-curl` `unittests src/lib.rs` | 34 | 33 passed, 1 ignored (pre-existing, reason-recorded), 0 failed |
| `eggup-curl` `tests/assertion_guard.rs` | 3 | 3 passed, 0 failed |
| `eggup-eggfetch` `unittests src/lib.rs` | 25 | 25 passed, 0 failed |
| `eggup-eggfetch` `tests/assertion_guard.rs` | 3 | 3 passed, 0 failed |
| `eggup-eggfetch` `tests/typed_io_contract.rs` | 2 | 2 passed, 0 failed |
| Doc-tests (all three crates) | 0 | 0 passed, 0 failed |
| **Total** | **116** | **115 passed, 1 ignored, 0 failed** |

### The three added tests

| Test | Line | Proves |
|---|---|---|
| `seam_changelog_declares_exactly_one_unreleased_section_with_a_next_release` | 359 | exactly one `## Unreleased` section; it is the **first** section; it declares a parseable next release |
| `dependent_requirements_accept_the_next_release_or_record_the_widening` | 387 | for each of the four dependents: the declared requirement accepts the declared next release, **or** a changelog records a requirement that does |
| `adapter_changelogs_record_the_widening_and_assert_no_contradicted_republication_version` | 426 | both published adapters' `Unreleased` sections record the widening, and neither claims a republication version whose dependency range excludes the seam's next release |

## Per-platform results

Reported separately, as required. **No platform is inferred from another.**

| Platform | `cargo fmt --check` | `cargo clippy --all-targets -D warnings` | `cargo test` | `cargo doc` | Status |
|---|---|---|---|---|---|
| **macOS** (darwin 25.6.0, arm64) | pass (exit 0) | pass (exit 0, no warnings) | pass (115 passed, 1 ignored, 0 failed) | pass (exit 0) | **Run — green.** |
| **Linux** | not run | not run | not run | not run | **Not run.** No Linux host or container was available in this session. Nothing is inferred from the macOS lane. The new guard is platform-neutral — it reads text embedded at compile time and spawns nothing — so it has no platform-dependent code path, but that is a **design** statement, not a Linux result. |
| **Windows** | not run | not run | not run | not run | **Not run.** No Windows host was available in this session. Nothing is inferred from the macOS lane. |

**Known platform difference carried forward, not introduced here.** The
`#[cfg(not(windows))]` gate on the curl crate's live-loopback tests is
unchanged; this milestone adds no test to either adapter and changes no
platform-gated code path.

## Invariant review

| Invariant (plan §4) | Result | Evidence |
|---|---|---|
| 1. Transport neutrality | **Held** | The guard reads the adapters' manifests and changelogs **as text** and depends on neither crate. `cargo tree -p eggup-curl --depth 1` shows one dependency; `eggup-eggfetch` was not touched. |
| 2. The seam stays zero-dependency | **Held** | `crates/eggup-acquisition/Cargo.toml` is unmodified; `cargo tree -p eggup-acquisition --depth 1` prints no dependency line. The semver comparison is implemented in the guard rather than added as a `semver` dev-dependency. |
| 3. No manifest is changed | **Held** | `git status --short` lists no `Cargo.toml`. Every requirement is recorded in prose and in a machine-readable changelog marker. |
| 4. No publication act | **Held** | No `cargo publish`, no tag movement, no crates.io write. The `cargo package` runs were read-only verifications whose artifacts were written to `/tmp`. |
| 5. The seam's public API is unchanged | **Held** | `crates/eggup-acquisition/src/**` is not modified; the guard lives in `tests/` and adds no public item. |
| 6. The adapter's public API and behavior are unchanged | **Held** | The only `eggup-curl` source change is inside a `///` rustdoc block; `validate_user_agent` and `build_curl_args` are byte-identical. The curl suite passes 33/33 with the pre-existing single ignore. |
| 7. No authenticity claim | **Held** | The new prose is about version coupling and compatibility only. No statement anywhere implies publisher trust, signature validity, or provenance. |
| 8. Existing tests are not weakened | **Held** | No existing assertion was removed or relaxed; the three `assertion_guard` suites and the five `typed_io_contract` tests pass unchanged. The pre-existing ignored curl test keeps its recorded reason. |
| 9. The published artifacts are unchanged | **Held** | Only `Unreleased` sections and one rustdoc sentence changed. `eggup-acquisition 0.1.3`, `eggup-eggfetch 0.1.3`, `eggup-eggpack 0.1.3`, and `eggup-curl 0.1.2` keep their published shape. |

## Failure, rollback, and recovery review

- **The guard fails loudly; it never fails silently.** A missing declaration,
  an unrecognized requirement form, a dependent whose requirement cannot be
  parsed, or a dependent with neither an accepting requirement nor a recorded
  widening is a test failure with a message naming the crate, the requirement,
  and the declared next release. An unrecognized requirement form is an
  **error**, never an assumption of acceptance (`Requirement::parse`).
- **The guard mutates nothing.** It reads text embedded at compile time: no
  file, no network, no clock, no process. It cannot leave state behind and
  cannot race with a concurrent writer.
- **A wrong correction is loud.** Removing the declaration fails the guard;
  removing the coupling record fails the guard; re-introducing a republication
  claim at a version the seam's next release contradicts fails the guard.
- **The `User-Agent` correction cannot change behavior.** It is a rustdoc-only
  edit; the validator's rules are untouched, so no request can change.
- **Cancellation / restart / retry / contention.** Not applicable: no runtime
  state exists in this milestone.
- **Rollback.** Reverting the change is a file-level revert. The guard is
  additive and the changelog entries are additive, so reverting cannot break
  a consumer that never adopted them — nothing published changed.

## Compatibility and migration review

### Exactly what is source-breaking for a consumer of the published artifacts

**Nothing.** Every change is to unpublished content:

- `eggup-acquisition 0.1.3`, `eggup-eggfetch 0.1.3`, `eggup-eggpack 0.1.3`,
  and `eggup-curl 0.1.2` keep their current shape and behavior.
- The changelog `Unreleased` sections and one rustdoc sentence are the only
  content changes; a rustdoc sentence is not an API change.
- The new test file adds no public item to any crate.

### What a consumer inherits once the coupling is published

- A consumer that depends on the seam **directly** (`eggup-acquisition =
  "0.1.3"`) keeps resolving `0.1.3` — a caret does not cross a major boundary
  — and is therefore **unaffected** until it opts into `0.2`.
- A consumer that opts into `eggup-acquisition 0.2` **inherits the seam's
  source break**: `AcquisitionError::Io(String)` became
  `Io { operation, source }`, and `Clone`/`PartialEq`/`Eq` were dropped.
- A consumer of `eggup-curl 0.1.2` or `eggup-eggfetch 0.1.3` is unaffected
  today, for the same caret reason. Once the adapters are republished with the
  widened `0.2` requirement, **their** consumers inherit the seam's source
  break too, because the seam's `AcquisitionError` is part of the adapters'
  public surface (`AcquisitionTransport::fetch` returns it).

### Publication status

**No republication was performed by this milestone.** No version was bumped,
no tag was moved, no crates.io write of any kind occurred, and **no manifest
was changed**. The in-tree manifests still inherit the workspace version
`0.1.3`, which for `eggup-curl` was never published — that is the pre-existing
repository state, unchanged.

### Migration

None is possible or required now. The migration that the **future** publication
forces is the seam's own (`AcquisitionError::Io` matchers, and the dropped
derives), inherited by the adapters' consumers as described above.

## Security review

| Question | Finding |
|---|---|
| Does the correction change any accepted or rejected `User-Agent` value? | **No.** The `User-Agent` change is a rustdoc edit. `validate_user_agent` still refuses every C0 control (including `CR`/`LF`/`NUL`), `DEL`, every C1 control, and a leading or trailing `SP`; it still enforces the 512-byte bound; and every rejection is still the terminal `InvalidInput` variant. |
| Does permitting a run of interior `SP` weaken the injection defense? | **No.** Repeated interior `SP` cannot inject a header (the `CR LF` refusal is what does that), cannot become an option (the value is one literal argv element, and `curl` consumes the element after `--user-agent` unconditionally), and cannot escape the field value. RFC 7230 §3.2's `field-content` permits the run (`1*( SP / HTAB )`), so the accepted set remains a subset of a well-formed HTTP field value. |
| Does the guard execute anything untrusted? | **No.** It parses text embedded at compile time. It spawns no process, opens no socket, reads no file at runtime, and writes nothing. |
| Can the guard be defeated by an unparseable requirement? | **No.** An unrecognized requirement form is an **error** that fails the test; it is never treated as acceptance. |
| Does the guard leak anything? | **No.** It reads manifests and changelogs — no URL, credential, or path material is involved, and the failure messages quote only crate names, version requirements, and version numbers. |
| Does any new prose imply authenticity, publisher trust, or provenance? | **No.** The new text is about version coupling, requirement widening, and consumer migration obligations. SHA-256 remains integrity evidence only. |

## Documentation / operations evidence

| Document | What changed |
|---|---|
| `crates/eggup-acquisition/CHANGELOG.md` | The stray second `## Unreleased` heading and its duplicated boilerplate removed (the file now has exactly one, at the top, carrying the M013 entry). Added: the explicit next-release declaration (`:8`), the publication-coupling record with all four dependents (`:12-31`), and the "records the coupling, does not perform it" statement (`:24-31`). |
| `crates/eggup-curl/CHANGELOG.md` | New M016 entry (`:8-37`) carrying the four corrected consequences and the widening marker. The M014 entry's false "non-breaking `0.1.3`" sentence corrected in place (`:113-117`). The M015 and M013 entries' "no consumer migration is required in this crate" narrowed to "by this crate's own API change" (`:73-78`, `:160-163`). |
| `crates/eggup-eggfetch/CHANGELOG.md` | New M016 entry (`:45-74`) carrying the same four consequences and the marker. The M013 entry's consumer-migration sentence narrowed (`:99-102`). The stale "`eggup-curl` exposes none" capability claim corrected in place (`:104-108`) — a pre-existing drift in an owned file. |
| `crates/eggup-curl/src/lib.rs` | The `User-Agent` grammar block's `SP` line reworded to match the implementation, with the RFC 7230 §3.2 justification (`:222-233`). **Documentation only.** |
| `crates/eggup-acquisition/tests/publication_coupling.rs` | New: the three-test publication-coupling guard. |
| `crates/eggup-acquisition/tests/assertion_guard.rs` | The new test file joins the scanned sources (`:23`, `:29`); M016's three behaviors join the named witness list (`:130-144`). |
| `plans/implementation/acquisition-transport/016-adapter-publication-coupling-corrective.md` | New plan (16 sections). |
| `plans/closure/acquisition-transport/016-status.md` | This record. |
| `plans/implementation/acquisition-transport/014-adapter-capability-parity-corrective.md` | Dated addendum (2026-10-24) superseding §6.1, §9.3, and §6.3's grammar line. |
| `plans/closure/acquisition-transport/014-status.md` | Dated addendum (2026-10-24) in the existing addendum style, with a superseded-statement → corrected-statement table. |

**Documents deliberately NOT changed** (owned by the parent session or another
writer): `plans/registry.md`, `plans/subsystems/**`, the root `CHANGELOG.md`,
`architecture/**` (including `architecture/curl-adapter.md`, which repeats the
corrected `User-Agent` grammar), `docs/**`, `AGENTS.md`, `scripts/**`,
`.github/**`, `.opencode/skills/**`, `crates/eggup-curl/README.md` (which
repeats the corrected `User-Agent` grammar), and every other crate.

**Working-tree note.** The working tree was clean at this milestone's first
inspection (`bf439f5`) and, by the time this record was written, carried
changes owned by **other writers** in the same batch — `AGENTS.md`,
`architecture/**`, `plans/registry.md`, `plans/subsystems/**`, the root
`CHANGELOG.md`, `crates/eggup-core/**`, `crates/eggup-service/CHANGELOG.md`,
`docs/**`, `scripts/**`, and `.opencode/skills/**`. **None of those files was
touched by this milestone**, and none was read for content or depended upon.
The `eggup-acquisition`, `eggup-curl`, and `eggup-eggfetch` results above are
unaffected by them: `eggup-curl` depends only on `eggup-acquisition`, which
depends on nothing. In particular, `plans/registry.md` and
`plans/subsystems/acquisition-transport-roadmap.md` show changes that are
**not** M016's; the M016 rows for both are drafted in the appendix and are
**not** applied.

## Unresolved findings

The repository's rule is that **"medium-or-higher" is the blocking-severity
threshold**: any such issue still open forces a new corrective before the
milestone closes.

| ID | Finding | Severity | Status |
|---|---|---|---|
| M016-UF-1 | The whole-tree gate (`./scripts/check-local.sh`) and the Linux and Windows lanes were **not run**; the parent session scoped this session to the three acquisition crates. | **low** | **Not run, recorded as not run.** The three crates' own suites, clippy, fmt, doc, and the dependency-boundary review were run. The new guard is platform-neutral by construction (no subprocess, no network, no clock). |
| M016-UF-2 | `crates/eggup-curl/README.md` and `architecture/curl-adapter.md` repeat the **corrected** `User-Agent` grammar — "SP permitted only between two non-SP characters" — which was wrong in the same way the rustdoc was. | **low** | **Open, and owned by another writer.** The authoritative rustdoc (`crates/eggup-curl/src/lib.rs:222-233`) is now correct, and the M016 closure and plan record the drift. The README ships with the crate, so it **must** be corrected before the adapter's next publication — it is listed in the plan's §9.3 action list context and named in the plan's §16 handoff notes. **Not corrected here**: both files are outside this milestone's owned paths. |
| M016-UF-3 | `crates/eggup-eggfetch/CHANGELOG.md`'s `## Unreleased` section sits **below** the published `0.1.3` section, whereas `eggup-acquisition`'s and `eggup-curl`'s sit at the top. | **low** | **Open, deliberately not changed.** Moving it is a section relocation with no semantic effect, and it is outside the reported defect set. It is recorded here so it is not re-raised as a new finding. |
| M016-UF-4 | The adapters **cannot be published** today: `cargo package -p eggup-curl` and `cargo package -p eggup-eggfetch` both fail their verification build against the published `eggup-acquisition 0.1.3`. | **medium** | **Recorded, not closed — and correctly so.** This milestone is explicitly forbidden from changing a manifest, and the fix belongs to the publication milestone (plan §9.3: bump the workspace version, widen the four dependents' requirements, publish seam → adapters → leaf). The coupling that made this a *known* obligation rather than a late surprise is exactly what this milestone records, and the guard now fails if the widening is never recorded. The **publication milestone is the named follow-on**; it is a separate, explicitly-governed act and is not performed here. |

**No finding is open at medium or higher that this milestone was authorized to
close.** M016-UF-4 is medium and is **deliberately deferred to the
publication milestone**, whose plan must be authored before the adapters can
be published. The plan's §16 names it.

## Roadmap disposition

**The parent session owns `plans/subsystems/acquisition-transport-roadmap.md`.**
This milestone does not edit it. The intended row is drafted below for the
parent to apply, in the roadmap's own status-table format.

Intended roadmap status-table row:

> | M016 | conditionally closed 2026-10-24 | `plans/implementation/acquisition-transport/016-adapter-publication-coupling-corrective.md` | `plans/closure/acquisition-transport/016-status.md` | Records the seam's `0.2.0` publication coupling and corrects the false "non-breaking `0.1.3`" consequence in the adapters' changelogs. The seam's next release is source-breaking, so both adapters must widen `eggup-acquisition` to `0.2`, `eggup-eggpack` must re-pin to `=0.2.0`, and `eggup-transport-footprint` must widen to `0.2`; the workspace is version-lockstep, so the adapters' next versions are `0.2.0`, not `0.1.3`, and a consumer of the seam inherits the source break. A guard reads the declared next release and every dependent's requirement and fails when a dependent neither accepts it nor records the widening. No manifest, version, or publication; the adapters still cannot be packaged, and the publication milestone owns that |

Intended subsystem narrative addition (to the acquisition-transport section):

> **M016 (2026-10-24)** recorded the publication coupling that M013's
> `0.2.0` decision creates and corrected the two documents that asserted the
> opposite. The seam's next release is source-breaking and does not satisfy the
> `^0.1.0` requirement the adapters declare, so both adapters must widen to
> `0.2`, the leaf adapter must re-pin to `=0.2.0`, and the workspace's
> version-lockstep means the adapters' next versions are `0.2.0` rather than
> `0.1.3` — with a consumer of the seam inheriting the source break. A new
> guard reads the seam's declared next release and every dependent's
> requirement, and fails when a dependent neither accepts it nor has the
> widening recorded. The coupling is now visible before a publication attempt
> rather than after one.

## Registry updates

**The parent session owns `plans/registry.md`, `plans/subsystems/**`, and the
root `CHANGELOG.md`.** None of them was edited by this milestone. The intended
prose is drafted below for the parent to apply.

### `plans/registry.md` — dependency-ready implementation work table

Append the following row to the table (the table's rows stay ≤ 481 bytes; this
one is 402 bytes):

> | Acquisition transport | M016 adapter publication-coupling corrective | conditionally closed 2026-10-24 | `plans/closure/acquisition-transport/016-status.md` | Records the seam's `0.2.0` coupling: both adapters must widen `eggup-acquisition` to `0.2`, `eggup-eggpack` re-pins to `=0.2.0`, footprint widens to `0.2`; publish seam then adapters then leaf. Guarded; no manifest, version, or publication |

### `plans/registry.md` — acquisition-transport subsystem summary row

The subsystem row (currently "Acquisition transport | M001-M011 closed; …")
should gain M016 to its closed list:

> Acquisition transport | M001-M011 closed; M009 published `eggup-curl 0.1.2` (2026-10-04), M010 published `eggup-acquisition 0.1.3` (2026-10-06), M011 published `eggup-eggfetch 0.1.3` (2026-10-06); M012 registered, `ready for handoff`; **M014 implemented 2026-10-23 (adapter capability parity corrective — M013's UF-1 closed; capability parity only; no republication)**; **M016 conditionally closed 2026-10-24 (adapter publication-coupling corrective — the seam's `0.2.0` coupling is recorded and guarded; the adapters' `0.1.3` republication consequence was false and is corrected; no manifest, version, or publication)** | `plans/closure/acquisition-transport/` |

### Root `CHANGELOG.md`

The root changelog aggregates but does not replace the per-crate changelogs.
Intended entry:

> ### Acquisition transport — M016 adapter publication-coupling corrective
>
> The seam's next release (`eggup-acquisition 0.2.0`) is source-breaking and
> does not satisfy the `^0.1.0` requirement the two adapters declare, so both
> adapters must widen to `0.2`, `eggup-eggpack` must re-pin to `=0.2.0`, and
> the workspace's version-lockstep means the adapters' next versions are
> `0.2.0`, not `0.1.3`. The coupling is now recorded in the seam's changelog
> and guarded by a test that fails when a dependent neither accepts the
> declared next release nor records the required widening. The adapters'
> changelogs no longer claim a non-breaking `0.1.3` republication, and the
> curl adapter's `User-Agent` grammar now matches what it accepts. **No
> manifest, version, or publication was changed.** See
> `crates/eggup-acquisition/CHANGELOG.md` (Unreleased),
> `crates/eggup-curl/CHANGELOG.md` (Unreleased),
> `crates/eggup-eggfetch/CHANGELOG.md` (Unreleased), and
> `plans/closure/acquisition-transport/016-status.md`.

### `architecture/curl-adapter.md` and `crates/eggup-curl/README.md`

Both repeat the `User-Agent` grammar that this milestone corrected. The
intended replacement line is the one now in
`crates/eggup-curl/src/lib.rs:222-223`:

> ```text
> SP         = %x20            ; permitted inside the value; a run of SP is
>                              ; permitted, but a leading or trailing SP is not
> ```

Both files are owned by another writer and were **not** edited by this
milestone.

## Implementation status

The plan's `Status:` header was updated from `ready for handoff` to
`implemented` once this record existed, per `plans/implementation/README.md`
and the workflow in `architecture/tooling-governance.md` §6.

No "Parent-session verification addendum" section appears in this record: that
section is added by the parent session after the hosted CI qualification.

## Appendix — ready-to-paste rows

Both rows are formatted for their target tables and are **not** applied by this
milestone.

**`plans/registry.md`** — dependency-ready implementation work table (402
bytes, limit 481):

```markdown
| Acquisition transport | M016 adapter publication-coupling corrective | conditionally closed 2026-10-24 | `plans/closure/acquisition-transport/016-status.md` | Records the seam's `0.2.0` coupling: both adapters must widen `eggup-acquisition` to `0.2`, `eggup-eggpack` re-pins to `=0.2.0`, footprint widens to `0.2`; publish seam then adapters then leaf. Guarded; no manifest, version, or publication |
```

**`plans/subsystems/acquisition-transport-roadmap.md`** — status table:

```markdown
| M016 | conditionally closed 2026-10-24 | `plans/implementation/acquisition-transport/016-adapter-publication-coupling-corrective.md` | `plans/closure/acquisition-transport/016-status.md` | Records the seam's `0.2.0` publication coupling and corrects the false "non-breaking `0.1.3`" consequence in the adapters' changelogs. The seam's next release is source-breaking, so both adapters must widen `eggup-acquisition` to `0.2`, `eggup-eggpack` must re-pin to `=0.2.0`, and `eggup-transport-footprint` must widen to `0.2`; the workspace is version-lockstep, so the adapters' next versions are `0.2.0`, not `0.1.3`, and a consumer of the seam inherits the source break. A guard reads the declared next release and every dependent's requirement and fails when a dependent neither accepts it nor records the widening. No manifest, version, or publication; the adapters still cannot be packaged, and the publication milestone owns that |
```

## Parent-session verification addendum (2026-10-24, hosted qualification)

This milestone is hosted-qualified. CI run `40067964510` (commit `a431aaa`) is
**green on all five lanes**. The per-platform results are reported separately,
and a lane that runs no test says so rather than inferring a result:

| Lane | Runner | Result |
|---|---|---|
| `stable` | ubuntu | **pass** — 528 passed, 0 failed |
| `msrv` | ubuntu, pinned `1.89.0` | **pass** — `cargo check` only; the lane runs no test by design |
| `service-systemd` | ubuntu, privileged | **pass** — 9 passed, 0 failed |
| `macos` | macos-latest | **pass** — 528 passed, 0 failed |
| `windows-check` | windows-latest | **pass** — 650 passed, 0 failed |
| local gate | this machine (macOS, arm64) | **pass** — `./scripts/check-local.sh` exit 0; 528 passed, 0 failed |

**The earlier hosted run of this branch was red, and the evidence is recorded
here rather than dropped.** Run `40067739294` (commit `3d91ab1`) failed the
`stable` lane: the core crate's pre-existing unit test
`bounded_candidates_are_exact_and_environment_is_cleared` failed with
`Io { operation: "spawning bounded candidate", source: Os { code: 26, kind:
ExecutableFileBusy, message: "Text file busy" } }`. That test existed at
baseline `c480541` and this branch did not modify it; the failure is an
`ETXTBSY` on the script the fixture had just written, which the kernel raises
while the file is momentarily open for writing. It did not reproduce in 25
consecutive local runs of `cargo test -p eggup-core --lib`, and the very next
hosted run was green on all five lanes. It is recorded as a re-observation of
registry finding **1** — the load-sensitive-test class — which is low severity
and blocks nothing.

**Windows and Linux evidence for the new fixtures in this milestone is native,
not compile-only**: the `windows-check` lane executes the workspace test
targets directly, and the `stable` and `macos` lanes run the same suite.
