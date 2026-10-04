# M009 — eggup-curl 0.1.2 Publication

Status: closed; see `plans/closure/acquisition-transport/009-status.md`

Repository baseline: `6a3e1e931b4976deebb0da5ca28f6462e899671e`

Source roadmap: `plans/subsystems/acquisition-transport-roadmap.md`

Primary class: package promotion / downstream-unblock

Hard dependencies:

- Acquisition M005 curl adapter + composition — closed.
- Acquisition M006 Windows portability/cross-closure corrective — closed.
- Acquisition M007 boundary safety hardening — conditionally closed with the Windows live-loopback limitation explicitly retained.
- Acquisition M008 sub-second deadline truthfulness — closed with hosted qualification.
- `eggup-acquisition 0.1.2` is already published and satisfies eggup-curl's existing compatible dependency requirement.

Downstream trigger: cargo-cleanme Phase 10 M010C needs the lightweight external-curl acquisition adapter from a registry-resolvable source and must not copy Gregg's local updater machinery.

## 1. Objective

Publish the already-implemented and qualified `eggup-curl` adapter as version 0.1.2 to crates.io without changing its acquisition semantics. Prove that an external consumer can resolve it from the registry with no Git/path dependency and preserve the existing bounded external-process contract.

This is a publication milestone, not a new transport feature milestone.

## 2. Current implementation evidence

`eggup-curl` already provides:

- exact caller-selected URLs only;
- external `curl` execution with no shell and no internal `sudo`;
- explicit connect/total ceilings and parent wall deadlines;
- exact HTTP 404 classification as `FetchOutcome::NotFound`;
- bounded streaming into Eggup-owned staging;
- cancellation/timeout child kill + reap;
- race-safe no-clobber promotion;
- credential-bearing diagnostic redaction;
- explicit redirect/protocol/proxy policy;
- no release/version/mirror policy.

M005/M006 qualified the adapter and composition model. M007 corrected diagnostic, finite-bound, and pathname-reopen defects. M008 corrected sub-second deadline truthfulness. The remaining M007 conditional note is specifically that hosted Windows loopback networking could not prove a live curl HTTP transfer; Windows portable process/error-path fixtures remain valid and no stronger Windows live-HTTP claim is allowed by this publication.

## 3. Invariants

- Do not alter release-selection or fallback policy; those remain caller-owned.
- Do not add Eggpack, release discovery, self-update, or service policy to `eggup-curl`.
- `NotFound` remains data for the exact URL, not a transport fallback decision.
- TLS/5xx/timeout/cancellation/size/staging failures remain hard acquisition failures.
- Keep core transport-neutral.
- SHA-256 verification/replacement remain above this adapter; the adapter itself does not claim authenticity.
- Publication must not imply that the historical Windows hosted live-loopback gap has been closed.

## 4. Package-readiness audit

Before publication:

1. inspect `crates/eggup-curl/Cargo.toml` metadata and package contents;
2. ensure README/license are included;
3. ensure path dependencies translate to registry-resolvable version requirements in the packaged manifest;
4. ensure the resulting package resolves against published `eggup-acquisition 0.1.2`;
5. confirm no unpublished workspace crate becomes a registry dependency;
6. inspect `cargo package -p eggup-curl --locked` output;
7. build/test from the packaged source where practical;
8. run `cargo publish -p eggup-curl --locked --dry-run`.

If Cargo requires a metadata-only adjustment for publication, keep it semver-compatible and behavior-neutral. Do not bundle unrelated workspace version changes.

## 5. Qualification

Re-run the current full workspace/package gates on the publication source revision:

~~~text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
cargo +1.89 check --locked --workspace --all-targets
cargo +1.89 test --locked --workspace --all-targets
cargo package -p eggup-curl --locked
cargo publish -p eggup-curl --locked --dry-run
~~~

Use the existing hosted Stable/MSRV/macOS/Windows matrix. Preserve the current platform qualification truth:

- Linux/macOS may carry live local HTTP/curl evidence.
- Windows must at minimum carry the already-qualified portable adapter/process/error-path fixtures.
- Do not fail publication solely because the hosted Windows environment still refuses the historical spawned-curl loopback case; do fail if current Windows portable coverage regresses.

## 6. Publication sequence

Publish only `eggup-curl 0.1.2`.

After registry visibility:

- create a fresh external fixture crate outside the Eggup workspace;
- depend on `eggup-curl = "=0.1.2"` from crates.io;
- resolve/build with no Git/path overrides;
- exercise a deterministic fixture path sufficient to prove the public API is usable;
- inspect `Cargo.lock` to ensure `eggup-curl` and `eggup-acquisition` resolve from the registry.

Tag/release naming should follow the existing per-package publication convention if used elsewhere in Eggup, e.g. `eggup-curl-v0.1.2`. Do not move existing `v0.1.2` tags or republish other packages.

## 7. Downstream compatibility

cargo-cleanme may consume the registry package after M009 closure, but cargo-cleanme owns:

- release/version selection;
- GitHub origin;
- exact URLs;
- fallback policy;
- install destination;
- self-update CLI semantics.

Gregg M004 remains separately unwritten; this publication does not authorize or imply a Gregg migration.

## 8. Documentation and registry updates

At implementation/closure:

- mark Acquisition M009 closed in the subsystem roadmap;
- add the publication/registry-only evidence to `plans/registry.md`;
- update README/package status only where it currently enumerates published crates;
- keep historical M005-M008 closure records unchanged.

## 9. Acceptance criteria

- `eggup-curl 0.1.2` is visible on crates.io;
- registry-only external resolution/build succeeds;
- package graph contains no Git/path source for Eggup dependencies;
- current Stable/MSRV/macOS/Windows qualification is green;
- no transport semantics changed except behavior-neutral package metadata if required;
- Windows live-loopback limitation remains stated truthfully;
- cargo-cleanme's lightweight updater dependency is unblocked.

## 10. Stop conditions

Stop before publication if:

- the packaged manifest requires an unpublished dependency;
- registry `eggup-acquisition` does not satisfy the packaged dependency graph;
- current-host tests reveal a semantic regression from M005-M008;
- package contents omit required source/license/readme material;
- publication would require widening transport semantics or claiming unsupported Windows live-network evidence.

A stop should produce a corrective/package-readiness plan rather than silently changing transport policy.

## 11. Closure evidence

Record:

- exact publication source revision;
- `cargo package` file list/checksum;
- publish dry-run result;
- hosted CI runs;
- crates.io checksum/version;
- registry-only external fixture `Cargo.lock` source evidence;
- any metadata-only delta;
- retained Windows qualification limitation;
- downstream cargo-cleanme handoff note.
