# Transport Footprint — `crates/eggup-transport-footprint` Deep Dive

Source: `crates/eggup-transport-footprint/` — `Cargo.toml` plus
`src/bin/{curl_only,eggfetch_only,dual}.rs` (45 bin source lines total, 3
binaries) and `tests/cross_link_guard.rs` (a 2-test source-level guard). The
crate-level contract lives in
[`Cargo.toml`](../crates/eggup-transport-footprint/Cargo.toml) and the three
bin sources; there is no `README.md`, no `CHANGELOG.md`, and no `src/lib.rs`.

## Purpose and ownership boundary

This crate exists to make **one decision measurable**: which HTTP transport stack
a consumer of `eggup-acquisition` links into its binary.

It is a build-time and link-time fixture, not a runtime component. It owns no
policy, performs no fetching, and exposes no library API. The three binaries do
not run a transaction — they construct a transport, construct a request, and
print what the configuration resolves to.

What it deliberately does not do:

- It does not choose a transport for a consumer. The consumer picks the feature.
- It does not measure download throughput, latency, or reliability. Those are
  not properties of a link graph.
- It does not add release or source fallback. Composition is expressed once, in
  `dual`, purely to show that the seam owns it.
- It is never published. `publish = false` in the manifest is deliberate, and
  `plans/registry.md` records it as such: "`eggup-transport-footprint` is
  `publish = false` by design."

## Position in the workspace

The crate is a **consumer of the seam**, not part of the core mechanism. It sits
in the same dependency layer as `eggup-eggfetch` and `eggup-curl`, and it is the
only place in the workspace where the two adapters appear in the *same* link,
by design and only in the `dual` binary.

Its dependency edges are declared as optional, so a consumer enabling one
feature does not compile the other:

```text
eggup-transport-footprint
  ├── eggup-acquisition            (always)
  ├── eggup-curl        (optional, feature "curl")
  └── eggup-eggfetch    (optional, feature "eggfetch")
```

The optional-dependency features are declared in the conventional form
(`curl = ["dep:eggup-curl"]`), which means the implicit `dep:` features are not
separately exposed — the crate's *only* public feature surface is `curl`,
`eggfetch`, and `default`.

| Crate | Manifest requirement | Resolved version | Note |
|---|---|---|---|
| `eggup-acquisition` | `"0.2"` (path) | `0.2.0` | The seam, and the only unconditional dependency |
| `eggup-curl` | `"0.2"` (path, optional) | `0.2.0` | External-process adapter |
| `eggup-eggfetch` | `"0.2"` (path, optional) | `0.2.0` | Native in-process HTTP/TLS adapter |

The requirement literals above are the ones in
[`crates/eggup-transport-footprint/Cargo.toml`](../crates/eggup-transport-footprint/Cargo.toml);
the resolved versions are the current workspace version (`0.2.0`). The manifest
is authoritative and is not changed by this document.

It inherits the 5 shared workspace keys (`version`, `edition`, `rust-version`,
`license`, `repository`) and sets `[lints] workspace = true`. Being binary-only,
it has no `#![forbid(unsafe_code)]` / `#![deny(missing_docs)]` crate attributes —
those live on the seven library crates. The workspace `unsafe_code = "deny"` lint
still applies to the bins.

## Public surface

There is no library target. The "public surface" is three binaries, each gated
by `required-features` so that Cargo refuses to build one whose adapter is not
linked.

| Binary | Path | `required-features` | Constructs | Prints |
|---|---|---|---|---|
| `curl_only` | `src/bin/curl_only.rs` | `curl` | `CurlTransport::with_executable("/usr/bin/curl", CurlConfig::strict())` | executable path, redacted request, default limits |
| `eggfetch_only` | `src/bin/eggfetch_only.rs` | `eggfetch` | `EggfetchTransport::strict(EggfetchConfig::strict())` | `max_redirects`, redacted request, default limits |
| `dual` | `src/bin/dual.rs` | `curl`, `eggfetch` | both transports wrapped in `ComposedTransport::new(&curl, &eggfetch, CompositionPolicy::UnavailableOnly)` | the resolved composition policy |

Each binary's module doc states its intent in one line, and that line is the
fixture's actual assertion:

```rust
//! Curl-only footprint binary: links seam + curl, never Eggfetch.
//! Eggfetch-only footprint binary: links seam + Eggfetch, never curl.
//! Dual-transport footprint binary: links seam + curl + Eggfetch + composition.
```

## The decision this crate exists to expose

`eggup-curl` and `eggup-eggfetch` are interchangeable at the type level — both
implement `AcquisitionTransport`, and neither knows the other exists. Because
the seam hides that choice, the *cost* of the choice is also easy to overlook
until it lands in a consumer's dependency graph and binary size. These binaries
are the fixture that makes the cost visible.

Measured on this workspace at commit `ae9b47a` (macOS, `debug` profile, exact
commands in [Verification](#verification)). The workspace version has since
advanced to `0.2.0`; the counts below are the dated capture and are
not re-derived — the commands in [Verification](#verification) reproduce them at
the current baseline.

| Configuration | Unique crates in the resolved tree | `curl_only` / `eggfetch_only` / `dual` binary size |
|---|---|---|
| `--no-default-features --features curl` | **3** | 575,640 B (≈ 562 KiB) |
| `--no-default-features --features eggfetch` | **92** | 6,280,608 B (≈ 5.99 MiB) |
| `--all-features` | **93** | 18,975,288 B (≈ 18.1 MiB) |

The `curl`-only tree is three crates deep in total — the fixture, the seam, and
`eggup-curl` — and `eggup-curl` contributes **no new external dependency**,
because its only dependency is the seam:

```text
eggup-transport-footprint v0.2.0
├── eggup-acquisition v0.2.0
└── eggup-curl v0.2.0
```

The `eggfetch` tree is 92 crates, because a native HTTP + TLS stack is linked
into the process. The top level looks identical, which is precisely the point —
the difference is entirely transitive and invisible at the source level:

```text
eggup-transport-footprint v0.2.0
├── eggup-acquisition v0.2.0
└── eggup-eggfetch v0.2.0
    ├── eggfetch-core v0.2.0
    │   └── hyper-rustls ─ rustls ─ tokio-rustls ─ rustls-webpki
    │       rustls-pki-types ─ rustls-native-certs ─ webpki_roots
    └── tokio, futures-util
```

The `dual` figure (93 unique crates, ≈ 18.1 MiB) is worth reading carefully
rather than summing the other two: it is substantially larger than either
adapter alone because both stacks, and both TLS implementations, are present in
one image. Composition is cheap at the *type* level and expensive at the *link*
level.

What this does and does not license as a conclusion:

- It **does** establish that a consumer can build a fetching updater that adds
  zero third-party crates to its dependency graph, by choosing `eggup-curl`.
- It **does not** establish that `eggup-curl` is preferable. Delegating to an
  external binary trades a static dependency graph for a runtime dependency on
  the presence and version of a system program, plus a different TLS trust
  posture. `curl` is resolved through the system package manager, and its
  version is not pinned by this workspace. The two adapters differ on more than
  size, and neither is a free choice.
- It says nothing about throughput, protocol support, redirect handling, or
  proxy behavior. Those are covered in
  [`curl-adapter.md`](curl-adapter.md) and [`eggfetch-adapter.md`](eggfetch-adapter.md).

## Feature and target gating

The gating is the crate's actual mechanism, and it has a consequence a reviewer
should know:

- `default = []`. With no features, **no binary target matches**, and Cargo
  reports: ``warning: target filter `all-targets` specified, but no targets
  matched; this is a no-op``.
- Enabling `curl` alone builds only `curl_only`; enabling `eggfetch` alone builds
  only `eggfetch_only`; enabling both builds all three.

This means a build of this crate proves nothing unless a feature is named
explicitly. A green `cargo build -p eggup-transport-footprint` with default
features compiles **zero** of the three fixtures.

## Invariants and what this crate enforces

This crate enforces no runtime invariant. What it *demonstrates* is that the
seam's boundary holds:

| Property | How the fixture demonstrates it |
|---|---|
| Adapters are interchangeable | The same two seam types are constructed and driven identically in `curl_only` and `eggfetch_only` |
| Adapters do not know each other | `curl_only` links no Eggfetch code, and vice versa; `required-features` stops a binary from *building* when its adapter's feature is off. It does **not** stop a cross-link when both features are on, because an enabled optional dependency becomes available to every target in the package — so "links no Eggfetch code" is a meaningful claim only in the single-adapter configurations, and those are the configurations CI builds as their own steps (see [Testing approach](#testing-approach)). The source-level guard `tests/cross_link_guard.rs` asserts the same claim in **every** configuration, including `default = []` |
| Composition belongs to the seam | Only `dual` references `ComposedTransport`, and it gets it from `eggup-acquisition` |
| URLs are redacted by the seam, not the adapter | Both single-adapter binaries print `request.redacted()`, exercising the seam's redaction rather than an adapter's |
| Bounds are resolved by the seam | Both single-adapter binaries print `FetchLimits::default()`, locating the bound in the seam |
| Policy is explicit at construction | `CurlConfig::strict()` / `EggfetchConfig::strict()` are passed at the call site, never defaulted implicitly |

Note that `curl_only` hardcodes `/usr/bin/curl` as the executable path. That is
a deliberate fixture value, not a discovery default: it keeps the binary from
performing PATH discovery it does not need, which is the same discipline
`eggup-curl`'s explicit-executable policy requires of real consumers.

## Testing approach

The crate has **two** tests, and they are not tests of the binaries. They live in
`tests/cross_link_guard.rs` and are **source-level**, not behavioral:

- **T1** — `single_adapter_fixtures_never_reference_the_sibling_adapter`: the whole
  file text of each single-adapter fixture is rejected if it names the sibling
  adapter, and the composition fixture is rejected if it stops naming either one.
  The check is over the whole text rather than over the lines that survive a `cfg`
  evaluation, because a `#[cfg]`-gated sibling reference compiles cleanly in the
  isolated configurations.
- **T2** — `manifest_keeps_the_shape_that_produces_the_recorded_counts`: the manifest
  shape that produces the 3 / 92 / 93 split is pinned — `publish = false`,
  `eggup-acquisition` as the only unconditional dependency, both adapters optional,
  `default = []`, and each binary's `required-features`.

The test target carries no `required-features`, so the guard is built and run in
**every** feature configuration, including `default = []`.

The three binaries themselves are still never executed, by design. The evidence the
crate produces is compile-time and link-time: a successful build proves the feature
combination resolves, the adapter links, and the seam's types compose. So "the
fixtures pass" is not a meaningful statement, and a reviewer should not look for
binary test names here. The verification that matters is that the build matrix below
stays green, and that the three binaries continue to compile against the seam.

### CI coverage

The isolated single-adapter configurations — the crate's entire purpose — are built
by CI. Each is its own step, so a failure names itself instead of hiding in a loop.

| Where | Command | Isolated single-adapter configurations built? |
|---|---|---|
| `scripts/check-local.sh` | `cargo clippy --workspace --all-targets --all-features --locked` | No — `--all-features` puts both adapters in the package |
| CI `stable` | three `cargo build -p eggup-transport-footprint` steps (`ci.yml:26-28`) | **Yes** — `curl` (`ci.yml:26`) and `eggfetch` (`ci.yml:27`), plus the dual configuration (`ci.yml:28`) |
| CI `msrv` | three `cargo check -p eggup-transport-footprint` steps (`ci.yml:52-54`) | **Yes** — `curl` (`ci.yml:52`) and `eggfetch` (`ci.yml:53`), plus the dual configuration (`ci.yml:54`) |
| CI `macos` | two `cargo build -p eggup-transport-footprint` steps (`ci.yml:97-98`) | **Yes** — `curl` (`ci.yml:97`) and `eggfetch` (`ci.yml:98`); the dual configuration is covered by the lane's workspace `--all-features` test step (`ci.yml:93`) |
| CI `windows-check` | `cargo check --workspace --all-targets --locked` | No — the lane passes no `--all-features` and names no footprint step |

**The gap this crate exists to close is closed.** The claim that `curl_only` links no
Eggfetch code and `eggfetch_only` links no curl code is now checked by a machine in
the `stable`, `msrv`, and `macos` lanes, instead of being asserted here.

Two residual asymmetries remain, and both are recorded rather than hidden:

- The Windows lane still passes no `--all-features` on purpose (see
  [`tooling-governance.md`](tooling-governance.md) §5.3), so the three feature-gated
  binaries are not built there. The bins are thin and platform-neutral by
  construction, so this is low risk — but a future change that made the fixtures
  platform-sensitive would not be noticed on Windows.
- The `macos` lane builds the two isolated configurations explicitly; it does not add
  a third `curl,eggfetch` step, because the lane's workspace `--all-features` step
  already compiles that configuration.

This crate's subsystem roadmap is
[`plans/subsystems/transport-footprint-roadmap.md`](../plans/subsystems/transport-footprint-roadmap.md).
Its M001 milestone closed the coverage gap recorded here.

## Verification

Exact commands, reproducible from the repository root:

```sh
# curl-only fixture — seam + curl, no Eggfetch
cargo build -p eggup-transport-footprint --no-default-features --features curl --locked

# eggfetch-only fixture — seam + Eggfetch, no curl
cargo build -p eggup-transport-footprint --no-default-features --features eggfetch --locked

# dual fixture — seam + both adapters + composition
cargo build -p eggup-transport-footprint --all-features --locked

# dependency-shape comparison
cargo tree -p eggup-transport-footprint --no-default-features --features curl --locked
cargo tree -p eggup-transport-footprint --no-default-features --features eggfetch --locked

# clippy over the bins
cargo clippy -p eggup-transport-footprint --all-targets --all-features --locked -- -D warnings
```

The `--locked` flag matters here: without it the dependency counts in the table
above are not reproducible, and a silent `Cargo.lock` update could change them.

## Cross-references

- [`overview.md`](overview.md) — module map, dependency graph, and the
  cross-cutting invariant that an adapter must not coordinate with its sibling.
- [`acquisition.md`](acquisition.md) — the seam these binaries drive, including
  `AcquisitionRequest::redacted()`, `FetchLimits`, and `ComposedTransport`.
- [`curl-adapter.md`](curl-adapter.md) — the external-process adapter, its
  explicit-executable policy, and what delegating to a system binary means.
- [`eggfetch-adapter.md`](eggfetch-adapter.md) — the native adapter, and the TLS
  and proxy posture that the 89 transitive crates exist to provide.
- [`tooling-governance.md`](tooling-governance.md) — the local gate and the CI
  matrix summarized in the coverage table above.
- [`../plans/registry.md`](../plans/registry.md) — records the `publish = false`
  decision and the registry state of every crate.
