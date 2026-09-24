# Planning and Closure Hygiene Corrective C001 — Closure

Status: closed

Source plan: `plans/implementation/planning-closure-hygiene-corrective/001-post-batch-status-and-evidence-reconciliation.md`

Source governance: `plans/003-planning-process.md#11-corrective-passes`, `#12-closure-records`, `#13-registry`, `#14-baseline-handling`.

Eggup starting SHA: `8937ad01b18268fd72c3917b0391f0c7c5b30f1c` (HEAD before corrective work).

C001 landing commit: `47bd68255534d3be5798968a4c290c6f202b3cb7` (the commit that landed the C001 reconciliation; parent is the starting SHA above).

CodeGG before (pre-adoption) SHA: `220d3638fe043b24e65a7817241543612f9e82af`
(`retrieval-architecture C001: explicit sweep provenance, remove git subprocess`).

CodeGG after (implementation) SHA: `23d84422c71bc1c251ba916a1a6c81e35c341fc9`
(`feat(upgrade): adopt eggup managed runfile updates`).

CodeGG closure/planning SHA: `e508de52083a30a1b234f5f9b619910cad6792d4`.

## Result

Evidence/documentation-only corrective. No Eggup or CodeGG production source
changed. The four post-batch findings are closed:

1. `plans/registry.md` now records the Core M007 production implementation
   baseline instead of the retired distribution baseline.
2. The consumer-adoption roadmap no longer describes supported CodeGG
   self-update as check-only and no longer labels M005 `[ready]`.
3. The M005 "deleted duplicated helpers/LOC" requirement has an explicit
   factual disposition: zero superseded generic updater helpers were deleted
   because the baseline had already retired its unsafe in-place execution
   path; the adoption adds CodeGG-specific policy/adapter/extraction code
   that consumes Eggup.
4. The M005 release-binary-size delta is now measured under equivalent
   conditions: +221,272 bytes (+0.26%) revision-to-revision on
   x86_64-apple-darwin release builds.

## Requirement-to-evidence matrix

| # | Plan requirement | Evidence | Result |
|---|---|---|---|
| 1 | Registry production baseline reflects Core M007, not the retired distribution implementation | `plans/registry.md`: `Last implementation baseline reviewed` is now `8d5fc12f7224145285f22d0975a7bb91e1e363ea`; stale `bc25885bd41b86bfdf2f32d1e42856e00829cd7a` control metadata removed; latest closure/planning baseline identifies the M007 closure state at `2cab1f97ef30fa347c2030da321462459672c521` | closed |
| 2 | Active consumer-adoption planning describes CodeGG M005 as closed, not check-only/ready | `plans/subsystems/consumer-adoption-roadmap.md` matrix row now reads `verified in-place managed-runfile update on supported Linux/macOS; manual fresh-install guidance elsewhere`; dependency graph now reads `M005 CodeGG [closed]`; sweep confirms remaining `check-only` occurrences are intentional historical descriptions | closed |
| 3 | CodeGG baseline-to-implementation updater diff quantified; duplicated-helper/LOC disposition explicit | `git diff --stat` / `--numstat` between the exact before/after SHAs plus targeted source inspection (see Helper/LOC evidence) | closed, zero removed generic helpers |
| 4 | Release binary-size delta measured equivalently or demonstrated unavailable | Controlled before/after release builds, same host/target/toolchain/profile/features (see Size evidence) | closed, +0.26% measured |
| 5 | M005 historical closure points to the supplement without falsifying its original statement | `plans/closure/consumer-adoption/005-status.md` gained a post-closure supplement pointer; the original "no release-profile before/after binary-size measurement was made" statement is preserved verbatim | closed |
| 6 | Service Lifecycle M005 disposition | Registry unblocks Service Lifecycle M005 for plan authoring; C001 closed | ready for plan authoring |

## Planning files changed

- `plans/registry.md` — implementation baseline `bc25885…` → `8d5fc12…`;
  added latest closure/planning baseline (`2cab1f9…` M007 closure state);
  C001 `ready` → `closed`; Service Lifecycle M005 held → ready for plan
  authoring; next-handoff and execution-graph wording updated.
- `plans/subsystems/consumer-adoption-roadmap.md` — CodeGG matrix row and
  `M005 CodeGG [ready]` → `[closed]` corrected.
- `plans/closure/consumer-adoption/005-status.md` — supplemental pointer only;
  original historical claims untouched.
- `plans/implementation/planning-closure-hygiene-corrective/001-post-batch-status-and-evidence-reconciliation.md` —
  status `ready for handoff` → `closed` with closure pointer.
- `plans/closure/planning-closure-hygiene-corrective/001-status.md` — this record (new).

Exact stale strings removed/corrected:

- `Last implementation baseline reviewed: `bc25885bd41b86bfdf2f32d1e42856e00829cd7a``
- `| CodeGG | check-only update; verified bundle installer | resolve generic updater blocker | 3-runfile bundle |`
- `M005 CodeGG [ready]` (dependency graph)

Remaining `check-only` occurrences in `plans/` were classified and are
intentional: `plans/002-long-term-roadmap.md` (phase deliverable wording),
the closed M005 implementation plan (pre-adoption state description,
Windows-manual posture), and the roadmap M005 milestone section
(`check-only updater blocker is closed` — states the closed historical fact).

## Helper/LOC evidence

Exact revisions measured: before `220d3638…`, after `23d84422…`.

Full-range diff: 20 files changed, 1437 insertions, 377 deletions (includes
CodeGG docs/plans and small 1.89-compatibility fixes noted below).

Updater/adoption-relevant paths
(`src/upgrade`, `tests/upgrade.rs`, `Cargo.toml`, `Cargo.lock`):

```text
15	0	Cargo.lock
2	0	Cargo.toml
919	0	src/upgrade/managed.rs
32	31	src/upgrade/mod.rs
13	38	tests/upgrade.rs
```

Named ownership disposition:

- Removed superseded generic updater helpers: none (zero). The baseline
  `src/upgrade/mod.rs` (149 lines) was already check-only after the M005
  hardening: it queried release metadata, acquired no candidate bytes, and
  attempted no replacement. There was no active generic replacement engine
  left to delete, so the "deleted duplicated helpers/LOC" result is
  legitimately zero.
- Repurposed (not a generic engine): `describe_upgrade` (pure fail-closed
  check-only disposition) became `describe_manual_fresh_install`, now used
  only for manual fresh-install guidance on unsupported in-place targets.
- Retained: `check_for_updates`, `current_version`, `installer_invocation`,
  `VersionInfo`, installer URL/version-pin constants, and the existing
  Eggfetch trust/timeout/redirect policy.
- New CodeGG-specific adapter/policy/extraction code: `src/upgrade/managed.rs`
  (919 lines, new file) — CodeGG-owned release/target/archive policy,
  checksum-manifest verification, strict bundle extraction, per-runfile
  identity/version validation, and Eggup multi-artifact transaction wiring;
  `upgrade()` dispatches to it on supported targets. `src/main.rs`
  `cmd_upgrade` now calls the managed path (1 insertion, 16 deletions of
  manual-guidance printing).
- Dependency delta: `+eggup-core`, `+eggup-acquisition` (both pinned to
  Eggup `66813b3b94de3a9b2f270e0000dc339ef6f0b478`), plus one transitive
  crate (`sha2` via `eggup-core`). No dependency removed.
- Incidental non-adoption edits inside the revision range (not attributed to
  Eggup): Rust 1.89 compatibility fixes in `src/agent/tool_batch.rs`
  (stable char-boundary truncation), `src/tui/commands/shell.rs` (module
  import), and `src/security/workflow/mod.rs` (path comparisons).

This satisfies the ownership goal: CodeGG had no active generic replacement
engine at baseline, and the adoption consumes Eggup rather than introducing
another one. No still-active duplicated generic updater engine was found, so
no consumer corrective was opened.

## Size evidence

Controlled comparison, identical conditions for both revisions:

- Host/target: macOS x86_64 build host, `--target x86_64-apple-darwin`.
- Toolchain: stable Rust 1.98.1 (`rustc 1.98.1 (48a229cea 2026-09-01)`) for
  both revisions. Rust 1.89 was attempted first per the workspace MSRV, but
  the before revision does not compile under 1.89 (pre-existing
  incompatibilities: `error[E0432]` on `use super::super as app;` in
  `src/tui/commands/shell.rs`, `error[E0658]` on `floor_char_boundary` in
  `src/agent/tool_batch.rs`, `error[E0282]` in `src/tui/commands/shell.rs`);
  the after revision contains the small compatibility fixes. Using one newer
  toolchain for both revisions keeps the comparison equivalent.
- Profile: release (`lto = "thin"`, `strip = true`, `codegen-units = 1`,
  identical in both revisions' manifests).
- Features: default feature selection; `--locked`; `--bin codegg`.
- Measurement: `wc -c target/x86_64-apple-darwin/release/codegg`
  (both binaries verified Mach-O 64-bit x86_64 executables reporting
  `codegg 0.1.0`).

Results:

- Before (`220d3638…`): 84,008,880 bytes.
- After (`23d84422…`): 84,230,152 bytes.
- Absolute delta: +221,272 bytes.
- Percentage delta: +0.2634% (≈ +0.26%).

This is a revision-to-revision adoption delta. It is not attributed causally
to Eggup alone: the implementation revision also contains the 1.89
compatibility fixes and CLI wiring noted above.

## Verification commands actually run

Eggup (docs-only change; HEAD `8937ad01…` before the corrective commit):

```text
git rev-parse HEAD                                            # 8937ad01…
git grep -n "M005 CodeGG \[ready\]" -- plans                   # only the C001 plan text + roadmap:106 (fixed)
git grep -n "check-only" -- plans/subsystems/consumer-adoption-roadmap.md plans/registry.md  # roadmap:69 (fixed), roadmap:153 historical (kept)
git grep -n "bc25885bd41b86bfdf2f32d1e42856e00829cd7a" -- plans/registry.md  # registry:5 (fixed)
cargo fmt --all -- --check                                    # passed
git diff --check                                              # passed
```

CodeGG historical evidence (fresh clone of `dbowm91/codegg`; exact SHAs
verified present via `git cat-file -t`):

```text
git diff --stat 220d3638… 23d84422…                           # 20 files, 1437+/377-
git diff --numstat 220d3638… 23d84422… -- src/upgrade tests/upgrade.rs Cargo.toml Cargo.lock
rustup run 1.89 cargo build --release --locked --target x86_64-apple-darwin --bin codegg   # before rev: FAILED (E0432/E0658/E0282, recorded above)
cargo build --release --locked --target x86_64-apple-darwin --bin codegg                   # before rev: finished release profile, 0 errors
cargo build --release --locked --target x86_64-apple-darwin --bin codegg                   # after rev: finished release profile, 0 errors
wc -c <before-target>/x86_64-apple-darwin/release/codegg      # 84008880
wc -c <after-target>/x86_64-apple-darwin/release/codegg       # 84230152
file <both binaries>                                          # Mach-O 64-bit executable x86_64
<both binaries> --version                                     # codegg 0.1.0
```

Full Eggup test suite (`scripts/check-local.sh`): not run — not required for
a Markdown-only change and the script couples nothing docs-specific; no
production source changed, so fmt plus diff-check is the proportionate
verification. Status per verification honesty: not run (not applicable to
docs-only change).

## Invariant review

- Historical implementation/closure SHAs unchanged and treated as immutable
  facts; prior closure text not rewritten.
- Supplemental evidence names exact revisions measured.
- No number manufactured: the size delta is measured, the helper deletion
  count is zero by inspection of the exact diff.
- Same target/profile/features/toolchain for both size builds; no
  debug-vs-release or cross-feature comparison.
- Current CodeGG HEAD not used as the "after" revision; the implementation
  point `23d84422…` is used throughout.
- No production source changes in Eggup or CodeGG; no publication, tag,
  release, or dependency update performed.
- ADR-0004 ownership unchanged.

## Failure/recovery review

This pass changes no runtime state machine. Planning failure rules held:

- Exact historical revisions resolved (`git cat-file -t` confirms all three
  CodeGG SHAs); no approximate SHA substituted.
- Equivalent binary builds were produced; no reproducibility blocker.
- No still-active duplicated generic updater engine found; no consumer
  corrective opened.
- No stale status masked an unclosed dependency; M005/Core M007 closures
  verified closed before relabeling.
- No production file modified to ease measurement (builds ran in external
  worktrees outside the Eggup repository).

## Compatibility/migration review

No API, CLI, filesystem, service, release, dependency, or package
compatibility change. No consumer migration performed. Historical closure
records remain valid; C001 supplements rather than rewrites them.

## Security review

No security-sensitive code touched. The CodeGG diff inspection confirms the
adoption preserves CodeGG-owned Eggfetch trust profile, archive checksum
verification before extraction, and strict extraction allowlisting, with no
reintroduced shell/curl execution path. No new threat surface in Eggup
planning files.

## Documentation/operations evidence

Roadmap, registry, M005 closure pointer, plan status, and this record are
the complete documentation surface. No canonical specification or ADR change.
No CodeGG documentation change required; CodeGG's own control surface was
not found stale by this pass.

## Unresolved findings

None. No medium-or-higher planning/evidence defect remains open. The four
C001 findings are closed above; no new finding was opened.

## Disposition

C001 is closed. Service Lifecycle M005 is unblocked and ready for plan
authoring against the qualified Core M007
`ValidatedTransaction::commit_with_post_commit` API. Do not combine Service
M005 implementation planning into this cleanup pass.
