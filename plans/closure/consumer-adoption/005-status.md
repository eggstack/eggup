# Consumer Adoption M005 — CodeGG Managed-Runfile Bundle Closure

Status: closed

Source plan: `plans/implementation/consumer-adoption/005-codegg-managed-runfile-bundle-adoption.md`

Source roadmap: `plans/subsystems/consumer-adoption-roadmap.md`

Eggup API source: `66813b3b94de3a9b2f270e0000dc339ef6f0b478`.
CodeGG implementation: `dbowm91/codegg@23d84422c71bc1c251ba916a1a6c81e35c341fc9`.
CodeGG closure/planning commit: `dbowm91/codegg@e508de52083a30a1b234f5f9b619910cad6792d4`.
Hosted CodeGG CI: [run 35876819155](https://github.com/dbowm91/codegg/actions/runs/35876819155), passed.

## Result and ownership

CodeGG now performs verified in-place updates on the already-supported
Linux/macOS x86_64/aarch64 prebuilt targets. It downloads and checks the
release archive using CodeGG's existing Eggfetch/Rustls/WebPKI profile, applies
strict product-owned archive extraction, validates each runfile, and commits
the three-runfile generation using Eggup's multi-artifact transaction.
Optional third-party notices remain data. Unsupported targets, including
Windows, retain explicit manual fresh-install guidance and no in-place update
claim.

CodeGG owns release selection, target/archive policy, CLI behavior, helper and
eggsearch identity, and extraction allowlist. Eggup owns acquisition contracts,
integrity continuity through staging, locking, commit/rollback/recovery, and
receipts. Eggpack remains the archive producer authority. Normal self-update
does not execute `install.sh` or external `curl`.

Historical dependency-security M005's blocked closure remains unchanged; the
follow-up resolution is recorded in [CodeGG's closure
record](https://github.com/dbowm91/codegg/blob/e508de52083a30a1b234f5f9b619910cad6792d4/plans/closure/dependency-security-workspace-consolidation/007-codegg-eggup-adoption.md).

## Evidence

- `eggup-core` and `eggup-acquisition` are pinned to the same immutable Eggup
  git revision in CodeGG `Cargo.toml` and `Cargo.lock`. Both reverse trees show
  CodeGG as the only direct consumer; the duplicate tree was reviewed.
- CodeGG adapts Eggup acquisition over its ordinary Eggfetch client, retaining
  the existing trust profile and avoiding `eggup-eggfetch` feature unification.
- The updater selects the exact archive basename and requires exactly one
  matching checksum entry. It verifies the archive before extraction and uses
  strict top-level regular-file allowlisting, fixed archive/member bounds,
  required-member checks, and negative traversal/link/extra/duplicate tests.
- Per-runfile SHA-256 requirements bind the verified extracted snapshot to
  Eggup staging and locked revalidation. CodeGG's version probe checks CodeGG
  identity/version, eggsearch's independent pinned version, and the helper's
  safe refusal identity. Existing destinations require the consumer ownership
  verifier; only genuinely absent historical siblings may be created.
- Receipt handling preserves committed, rolled-back, and recovery-required
  outcomes. CodeGG has no service/post-install health check requirement and
  does not use the M007 post-commit API.
- No generic Eggup corrective was required. Eggup production changes are not
  part of this milestone.

## Verification

Local qualification ran on macOS x86_64.

Passed:

```text
CodeGG scripts/verify.sh full
  - repository guards, formatting, check, and Clippy passed
  - default workspace test run passed: 4,891 tests
  - feature-enabled CodeGG run passed: 4,941 tests
CodeGG managed-upgrade unit fixtures                            # 9 passed
CodeGG cargo test --test upgrade --locked -- --test-threads=1   # 10 passed
CodeGG cargo clippy --workspace --all-targets --locked -- -D warnings
CodeGG cargo +1.89.0 check --workspace --all-targets --locked
CodeGG scripts/verify.sh quick
CodeGG scripts/release/test-release-tools.sh
CodeGG scripts/release/test-installer.sh
CodeGG scripts/release/test-clean-host.sh                       # 26 passed
CodeGG cargo tree -i eggup-core --locked
CodeGG cargo tree -i eggup-acquisition --locked
CodeGG cargo tree -d --locked
CodeGG git diff --check
```

The first full run exposed a missing explicit 404 fixture route. The fixture
was corrected, the focused wrapper test passed, and the repeat full run passed.
The Rust 1.89 check exposed pre-existing source compatibility issues; small
compatibility fixes were included with the implementation and the final check,
Clippy, quick checks, and focused upgrade tests all pass.

Hosted CodeGG Linux CI passed formatting, all repository guards, workspace
Clippy, and serialized workspace tests. Native macOS local qualification
passed. Hosted macOS and Windows jobs were not available; Windows in-place
replacement remains unsupported. No release-profile before/after binary-size
measurement was made, so no binary-size delta is claimed.

## Disposition

Consumer Adoption M005 is closed. The earlier concern that CodeGG would need
to copy Gregg's updater is resolved by Eggup's immutable-pinned public APIs.
The requested sequential batch proceeds to Verified Update Core M007; this
closure makes no claim about Windows replacement, archive authenticity, or
release binary-size impact.

## Post-closure supplement

Post-closure measurement and status reconciliation for this milestone lives
in `plans/closure/planning-closure-hygiene-corrective/001-status.md`
(planning/closure hygiene C001). That record quantifies the
baseline-to-implementation updater diff and the release binary-size delta
that were not measured at original M005 closure time. The statement above
that no release-profile before/after binary-size measurement was made
remains the accurate historical record of this closure; the C001 supplement
does not retroactively claim otherwise.
