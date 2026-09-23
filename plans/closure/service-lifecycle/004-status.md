# Service Lifecycle M004 — Closure and Verification Record

Status: closed

Source plan: `plans/implementation/service-lifecycle/004-windows-scm-adapter.md`

Source roadmap: `plans/subsystems/service-lifecycle-roadmap.md`

Reviewed repository baseline: `51baa10717c0faf761175f8c4563f1a2b181565c`

## Implementation commits/PRs

- Implementation: `e653544168c3fc0f532f22145ec015bbc6d7b7eb` (`feat: add Windows SCM service adapter`), `cf48f2055d30da374b70493b5b7299132bcd723f` (`feat: support Windows service dependencies`), `8830d63f87a71bc5bdf89fceb805ee47b3e46b64` (`fix: validate SCM dependencies by namespace`), and `51baa10717c0faf761175f8c4563f1a2b181565c` (`test: cover Windows SCM permission diagnostics`).
- No PR was required. Hosted CI run [35808039362](https://github.com/eggstack/eggup/actions/runs/35808039362) was triggered by the final implementation push. No publication or consumer migration was performed.

## Executive finding

M004 is complete. `WindowsScmManager` implements inspection, owned create and
refresh, start/stop/restart, and uninstall through the safe
`windows-service 0.8.1` API. Service name alone never proves ownership. Queried
SCM command lines are parsed with `windows-args 0.2.0`; malformed and
ambiguous commands are `Unknown`, while exact executable, argv, and optional
config identity are required for `Owned`. Mutations fail closed, transitions
share one monotonic deadline, and uninstall reports completion only after SCM
confirms absence. No medium-or-higher SCM finding remains open.

## Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Safe typed Windows SCM API | `windows-service = 0.8.1`, target-specific; no Win32 calls in first-party code | passed |
| Cross-platform backend seam | Private backend trait and scripted fake run in `eggup-service` tests | passed |
| Service key validation | Empty, control/NUL, path-like, and overlong names rejected; display name validated separately | passed |
| Exact ownership | Exact executable and argv; config identity exact-once; executable/argv drift is Foreign; malformed/ambiguous command is Unknown | passed |
| Windows command parsing | Quoted paths, spaces, quotes/backslashes, Unicode, relative and malformed command cases use `windows-args`; unquoted ambiguous executable paths fail closed | passed |
| SCM state mapping | Stopped/Running, all pending states, Paused, and unsupported/Unknown mapping covered | passed |
| Install and refresh | Absent create; Owned refresh after reinspection; race to Foreign denies refresh; unmanaged type/dependencies/account/load-order/tag preserved | passed |
| Start/stop/restart | Already-terminal states; pending confirmation; one deadline; restart never starts before confirmed stop; timeout remains incomplete/error | passed |
| Delete truthfulness | Owned stop/reinspection before delete; marked-for-delete remains incomplete until observed absent | passed |
| Access/no elevation | `access_denied_diagnostic_is_bounded_and_does_not_escalate`; Win32 access-denied maps to bounded remediation; no `sc.exe`, shell, ambient PATH, PowerShell, or UAC path | passed |
| No first-party unsafe code | Crate retains `#![forbid(unsafe_code)]`; dependency encapsulates Windows API calls | passed |
| Unix regression | Full workspace tests and `scripts/check-local.sh` pass | passed |

## Production implementation evidence

- `WindowsScmInstall` carries a caller-selected SCM service key, display name,
  start type, error control, optional create-time account name, and bounded
  timeout. Executable and arguments come from `ServiceSpec`; creation does not
  start the service.
- The descriptor supports caller-supplied service/load-order-group
  dependencies and omits custom passwords, descriptions, recovery actions,
  and entrypoint policy. Refresh preserves existing service type, unspecified
  dependencies, account, load-order group, and tag; the adapter updates only
  its declared fields.
- The SCM query returns an opaque command line in `ServiceConfig`. The adapter
  bounds its length, rejects control characters and unbalanced quotes, parses
  with `windows-args`, requires an absolute executable, and reconciles argv and
  optional config through the shared M003 identity rule.
- SCM StartPending/StopPending/ContinuePending/PausePending map to
  `Transitioning`; Paused and unsupported observations map to `Unknown`.
  Wait-hint sleeps are capped and share the original operation deadline.
- Uninstall stops a Running or Paused owned service, rechecks Owned+Stopped,
  requests deletion, and polls until Absent. Marked-for-delete or still-present
  results cannot report completion.
- Access denied returns bounded manager diagnostics that state an authorized
  account is needed. No operation starts an elevation flow or emits the
  underlying environment/configuration.

## Dependency impact

- Exact direct versions: `windows-service = 0.8.1` under
  `cfg(windows)`; `windows-args = 0.2.0` as a platform-neutral command parser.
- Windows `eggup-service` normal dependency tree adds `windows-service 0.8.1`
  (`bitflags 2.13.2`, `widestring 1.2.1`, `windows-sys 0.61.2`,
  `windows-link 0.2.1`), plus `windows-args 0.2.0` (`wtf8 0.0.3`).
- Non-Windows normal dependency tree adds only `windows-args 0.2.0` and
  `wtf8 0.0.3`; no Windows SCM/runtime crate is selected.
- No features were enabled on `windows-service`. Eggup code remains safe Rust.

## Exact commands and results

Environment: macOS 15 / Darwin 25.6.0 arm64; stable Rust 1.98.1; Rust 1.89.0.

Passed locally:

```text
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p eggup-service --all-targets --all-features --locked       # 70 passed
cargo test --workspace --all-targets --all-features --locked           # 197 passed
cargo doc --workspace --no-deps --locked
cargo package -p eggup-service --locked --allow-dirty                  # 7 files, 216.9 KiB unpacked
cargo +1.89.0 check --workspace --all-targets --locked
cargo +1.89.0 test -p eggup-service --all-targets --locked             # 70 passed
cargo check -p eggup-service --all-targets --locked --target x86_64-pc-windows-msvc
cargo clippy -p eggup-service --all-targets --all-features --locked --target x86_64-pc-windows-msvc -- -D warnings
cargo tree -p eggup-service --target x86_64-apple-darwin --edges normal
cargo tree -p eggup-service --target x86_64-pc-windows-msvc --edges normal
./scripts/check-local.sh
```

Hosted CI run `35808039362` passed stable formatting, clippy, workspace tests
and docs; Rust 1.89 workspace check; macOS workspace tests; and the full
Windows workspace check, including the real SCM backend. The local full
workspace Windows cross-check could not build the unrelated `ring` C target
because this macOS environment lacks the Windows C headers (`assert.h`); the
service crate itself compiled and passed clippy for `x86_64-pc-windows-msvc`,
and hosted Windows CI passed the workspace check.

No native Windows SCM instance was mutated. The Windows evidence is compile
and hosted CI evidence plus deterministic fake-backend tests; no privileged
runtime service integration was executed. Unix behavior is covered by the
macOS suite and existing local regressions, not inferred from Windows.

## Invariant, recovery, and compatibility review

- `Absent` permits explicit install creation. `Owned` is rechecked before
  refresh, lifecycle changes, and delete. `Foreign` and `Unknown` never reach
  a destructive backend operation.
- A present service with an unrepresentable/non-UTF-8 command, malformed
  quoting, non-absolute executable, or ambiguous unquoted path is never
  treated as Owned. Query errors are returned as errors, never converted to
  Absent.
- Restart has one deadline through ownership queries, stop confirmation,
  reinspection, start, and running confirmation. An incomplete stop prevents
  a start request.
- Create and refresh do not auto-start. Refresh carries forward the fields
  outside adapter ownership. A lost/raced registration returns conflict or
  ownership denial; it is not recreated by a refresh.
- Delete API acceptance alone is not success. Polling reports incomplete while
  the registration is marked for deletion or still present and confirms success
  only after an Absent query.
- No Unix APIs, product names, descriptions, recovery policy, network,
  transport, shell, `sc.exe`, remote manager, or service entrypoint were added.
  Existing Unix APIs remain source-compatible.
- No ADR, consumer migration, or publication was required.

## Unresolved findings

- Informational: no native Windows SCM runtime test ran because this host is
  macOS; the hosted Windows lane compiled the workspace but did not mutate
  SCM state.
- Deferred by scope: custom passwords, service descriptions, failure/recovery
  actions, service entrypoints, and consumer migration remain caller-specific
  or later evidence-driven work.
- None: no medium-or-higher SCM correctness/security finding remains open.

## Disposition and roadmap transition

Service M004 is closed. Service M005 prepared-transaction lifecycle
integration is ready for plan authoring because service M004 and
verified-update-core M005 are closed. Distribution M004 generator/adoption and
CodeGG M005 remain ready for plan authoring after distribution M003. Gregg
M004 remains blocked on corrected-path footprint evidence; Egress M006 remains
blocked on an archive update/extraction transaction contract. No plan in the
authorized sequence was blocked by an unforeseen issue.

## Registry updates

- Service M004 Windows SCM → closed.
- Service M005 prepared-transaction lifecycle integration → ready for plan
  authoring.
- Distribution M004 and CodeGG M005 remain ready for plan authoring.
- Gregg M004 remains footprint-gated; Egress M006 remains transaction-contract
  gated.
