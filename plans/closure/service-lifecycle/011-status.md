# Service Lifecycle M011 — Closure and Verification Record

Status: closed

Source plan: `plans/implementation/service-lifecycle/011-verified-service-patch-publication.md`

Source roadmap: `plans/subsystems/service-lifecycle-roadmap.md#M011--conditional-published-service-patch-for-wg-basic`

Reviewed planning baseline: `0bde3fefbda07019529ad7566c02e6e4ec141fd6` (M010 strict closure)

Pre-publication qualification head: `f65496fbd8950cb50d46620f57da336b2da232fb`; hosted run `37876276212` green.

Publication source: `feb6ae5aea4c9b61c4051957f3662ca49d845f9e` (`crates/eggup-service`)

Post-publication qualification head: `2fbb3c6f75ae536b53a7d30087c0b0edef7c74ec`; hosted run [`37883703334`](https://github.com/eggstack/eggup/actions/runs/37883703334) green on all five jobs.

## Executive finding

Service M011 published the M010 owned-failed-systemd quiescence correction as
`eggup-service 0.1.3` on 2026-10-09. Only `eggup-service` was uploaded. The
registry package is 74,405 bytes (72.7 KiB compressed; 396.0 KiB unpacked),
with SHA-256
`9f7f7ea854577158e66aa202709ab1c97a3aedcf00b06c1ab914d25b132124dc`. Its
embedded `.cargo_vcs_info.json` identifies source commit
`feb6ae5aea4c9b61c4051957f3662ca49d845f9e` and path `crates/eggup-service`.
Cargo downloaded and verified the registry archive against this checksum.

A new external fixture depends on exactly `eggup-service = "=0.1.3"`; its lock
file resolves both `eggup-service 0.1.3` and `eggup-core 0.1.3` from the
crates.io registry, with no path or Git edge. Hosted systemd 255 evidence
confirms the published adapter completes stop for the exact-owned failed unit
only after quiescence evidence agrees. The same run retains the 0.1.2 negative
control and the Foreign/malformed inspection controls.

The existing shared `v0.1.3` tag was not moved. GitHub Release `0.1.3` was
extended with the package-specific source, checksum, downstream pin guidance,
and the exact registry `.crate` as a release asset. No other crate was
published or changed in the registry.

## Publication record

| Field | Value |
|---|---|
| Registry package | `eggup-service 0.1.3` |
| Published | 2026-10-09; explicit maintainer authorization received in this task |
| Registry checksum | `9f7f7ea854577158e66aa202709ab1c97a3aedcf00b06c1ab914d25b132124dc` |
| Local registry cache SHA-256 | identical |
| Package size | 74,405 bytes; 10 files; 396.0 KiB unpacked / 72.7 KiB compressed |
| Source revision | `feb6ae5aea4c9b61c4051957f3662ca49d845f9e` |
| `.cargo_vcs_info.json` path | `crates/eggup-service` |
| `cargo info` | Version 0.1.3; Rust 1.89; MIT; registry resolves |
| GitHub release | [0.1.3](https://github.com/eggstack/eggup/releases/tag/v0.1.3), extended with this package and `.crate` asset |
| GitHub asset verification | Downloaded `eggup-service-0.1.3.crate`; 74,405 bytes and SHA-256 identical to crates.io |
| Existing tag | `v0.1.3` object `414279c51fa274be9968030dacacb8b033090210`, peeled commit `bd4368369aaf0ae1de2ce213fc16689281bdd664`; unchanged |

## Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| M010 correction is required and source-compatible | M010 closure `plans/closure/service-lifecycle/010-status.md`; no public enum/signature change or added dependency | passed |
| `0.1.3` unused before publish | `cargo info eggup-service@0.1.2 --registry crates-io` resolved; `cargo info eggup-service@0.1.3 --registry crates-io` returned no version before upload | passed |
| Published package source is identified exactly | `.cargo_vcs_info.json` inside the downloaded `.crate` names `feb6ae5aea4c9b61c4051957f3662ca49d845f9e` and `crates/eggup-service` | passed |
| Registry checksum matches the downloaded artifact | `shasum -a 256` of Cargo's registry cache file equals the checksum stored in the 0.1.3 external fixture lockfile and crates.io resolution | passed |
| Registry-only graph has no path/Git edge | `crates/eggup-service/tests/published-api-0.1.3/Cargo.toml` declares only `eggup-service = "=0.1.3"`; lock/tree show registry `eggup-service 0.1.3` and `eggup-core 0.1.3` | passed |
| API remains compatible | External fixture wildcard-matches `LifecycleState`; Owned failed inspection remains `Ownership::Owned` + `LifecycleState::Unknown`; no signature/variant migration | passed |
| Published 0.1.2 remains a discriminating negative control | Existing privileged `published-api-0.1.2` fixture still observed `Owned/Unknown` with `MainPID=0`, `ControlPID=0`, empty `ControlGroup`, no job, yet stop did not complete | passed |
| Published 0.1.3 completes exact-owned failed-unit stop safely | New privileged registry-only fixture observed `ActiveState=failed`, zero PIDs, no job and empty ControlGroup; stop returned `completed=true` only after independent post-stop proof | passed |
| Foreign and malformed controls fail closed | New registry-only 0.1.3 inspection controls passed for changed `ExecStart` → Foreign and malformed `ExecStart` → Unknown | passed |
| Other packages and tags remain untouched | One crates.io upload (`eggup-service 0.1.3`); `v0.1.3` remote ref and peeled target unchanged | passed |
| GitHub release is complete | Existing Release `0.1.3` notes extended; downloaded release asset is 74,405 bytes and SHA-256 matches the registry artifact; tag not recreated or moved | passed |
| Downstream version/checksum handoff is explicit | Release note and this record name wg-basic Distribution M004 C001a, exact pin `=0.1.3`, source SHA, checksum, and no API migration | passed |
| No newly unblocked Eggup implementation plan was omitted | Service M011 is the only direct successor to M010. The independent Acquisition M012 readiness is unchanged; wg-basic M004 remains active on rootful qualification and production trust-root needs, so M005 remains blocked | passed |

## Registry-only fixture and hosted platform evidence

Fixture: `crates/eggup-service/tests/published-api-0.1.3/`. The standalone
manifest has no workspace inheritance, `[patch]`, path dependency, or Git
dependency. Its lockfile records:

```text
name = "eggup-core"    version = "0.1.3"
  source = "registry+https://github.com/rust-lang/crates.io-index"
  checksum = "161b244bc474fa24815ebd81db8cda46e0f752dd4b84983e454239f89fd789ad"
name = "eggup-service" version = "0.1.3"
  source = "registry+https://github.com/rust-lang/crates.io-index"
  checksum = "9f7f7ea854577158e66aa202709ab1c97a3aedcf00b06c1ab914d25b132124dc"
```

Hosted run `37883703334` passed:

| Lane | Evidence | Result |
|---|---|---|
| Stable/Linux | fmt, clippy, full workspace tests, docs, packaged service verification | passed |
| Rust 1.89 MSRV | workspace check and packaged service verification | passed |
| Linux systemd | 0.1.2 negative control, current workspace failed-stop suite, 0.1.3 registry-only API/check/systemd suite | passed |
| macOS | full workspace test suite and packaged service verification | passed |
| Windows | targeted native tests, full workspace tests, packaged service verification | passed |

The post-publication systemd fixture ran on Ubuntu 24.04.5 LTS, kernel
`6.17.0-1022-azure`, systemd `255 (255.4-1ubuntu8.17)`. It proved the published
0.1.3 stop against `/usr/bin/false` while systemd continued to report
`ActiveState=failed`, `MainPID=0`, `ControlPID=0`, `Job=`, and `ControlGroup=`;
`is-active` returned `failed`/3 and Eggup returned
`TransitionResult { operation: Stop, completed: true }`. The copied full
systemd integration suite also covers residual cgroup tasks, auto-restart,
start-limit behavior, and foreign service preservation. The external
inspection-control suite passed all three tests.

## Exact commands and results

Local verification on the final implementation/test tree:

```text
./scripts/check-local.sh                                             PASS
cargo fmt --manifest-path crates/eggup-service/tests/published-api-0.1.3/Cargo.toml -- --check  PASS
cargo check --manifest-path crates/eggup-service/tests/published-api-0.1.3/Cargo.toml --locked  PASS
cargo test --manifest-path crates/eggup-service/tests/published-api-0.1.3/Cargo.toml --locked -- --nocapture  4 passed; real systemd integration is skipped without EGGUP_SYSTEMD_INTEGRATION
cargo tree --manifest-path crates/eggup-service/tests/published-api-0.1.3/Cargo.toml --locked  registry-only graph
git diff --check                                                     PASS
```

Release qualification and publication:

```text
cargo package -p eggup-service --list --locked        PASS (10 files; pre-publication qualification)
cargo package -p eggup-service --locked               PASS (pre-publication qualification)
cargo publish -p eggup-service --dry-run --locked     PASS (pre-publication qualification)
cargo publish -p eggup-service --locked               Published 0.1.3 from feb6ae5
cargo info eggup-service@0.1.3 --registry crates-io   PASS; registry resolves 0.1.3
shasum -a 256 eggup-service-0.1.3.crate               9f7f7ea854577158e66aa202709ab1c97a3aedcf00b06c1ab914d25b132124dc
```

The Linux systemd integration was not run on the local macOS host; its native
result is provided by hosted Linux run `37883703334`. macOS and Windows
evidence is separately present in that same all-green run.

## Hosted rerun history

- Run `37883459560` failed in the existing systemd restart-limit fixture because
  it assumed `systemctl start` must return success for a `Type=exec`
  `/usr/bin/false` unit. The test now uses the changed `ExecStart` invocation
  record to prove the second launch and tests that the third is rate-limited;
  runtime code did not change for this harness correction.
- Run `37883703334` initially hit Linux `ETXTBSY` in the unrelated Core
  `bounded_candidates_are_exact_and_environment_is_cleared` test. The failed
  Stable job was rerun on the same source SHA and passed; all five jobs in the
  completed run are green.

## Invariants, compatibility, and security review

- Failed systemd status is still not a public `LifecycleState` variant.
- Stop completion requires exact ownership revalidation, stable manager
  observations, no active manager job, zero process IDs, consistent
  `is-active`, and validated cgroup quiescence when a control group exists.
- Foreign, Unknown, malformed, racing, and residual-process cases fail closed.
- No `reset-failed`, direct process kill, privilege escalation, new dependency,
  or automatic restart was introduced.
- The public API is unchanged; consumers can move a direct exact pin from
  `eggup-service = "=0.1.2"` to `=0.1.3` and refresh the lockfile without a
  source migration.
- Only `eggup-service` was published. SHA-256 remains integrity evidence only;
  this release makes no authenticity/signature claim.

## Documentation, known limitation, and roadmap disposition

Updated the service and root changelogs, release documentation, service and
workspace architecture, CI guidance, M011 plan, roadmap, and registry. GitHub
Release `0.1.3` now records the service source/checksum, downstream pin, and
registry `.crate` asset. No historical service M009 closure or shared tag was
rewritten.

One low-severity documentation limitation remains in immutable registry bytes:
the embedded `CHANGELOG.md` is the pre-upload source snapshot and labels 0.1.3 a
release candidate. The checked-in changelog and GitHub release note now state
the actual published status and document this snapshot. The source snapshot is
truthful as of its build time, the code/package checksum is verified, and no
runtime/API defect is involved. No medium-or-higher finding remains open.

The downstream handoff is now available in the GitHub release record and here:
wg-basic Distribution M004 C001a should use `eggup-service = "=0.1.3"` with
lockfile refresh; its public API needs no migration. This publication makes the
version prerequisite available but does not close wg-basic M004, whose rootful
qualification and production trust-root requirements remain, nor unblock M005.
No additional Eggup implementation plan became dependency-ready from this
publication. Acquisition M012 remains independently ready and unchanged.

## Registry updates

- Service Lifecycle M010: closed with required runtime correction; unchanged.
- Service Lifecycle M011: closed after registry verification, exact-version
  external/systemd proof, GitHub release extension, and downstream version
  handoff.
- Service Lifecycle roadmap and `plans/registry.md`: updated to published
  `eggup-service 0.1.3` and M011 closed.
- No other milestone status changed. wg-basic M004 remains active; M005 remains
  blocked on M004.
