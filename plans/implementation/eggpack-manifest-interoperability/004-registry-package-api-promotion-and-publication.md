# Eggpack Manifest Interoperability Milestone 004 — Registry Package/API Promotion and Publication

Status: active

Eggup plan-authoring baseline: `3f4e99e381b233bfd4be1a676218e9ba2cdce2d4`

Primary preflight: `plans/implementation/eggpack-manifest-interoperability/004a-package-api-promotion-readiness-preflight.md`; closure: `plans/closure/eggpack-manifest-interoperability/004a-status.md`

Producer prerequisite (now satisfied): Eggpack Release Manifest M003 — `eggpack-manifest 0.1.0` published to crates.io 2026-10-02 from `eggstack/eggpack@8d661e4eb9da1806e5d7c7606939d24e9aceb2c0`, checksum `2a08f24b05e9652878dd49145cdc3cbd38c7a76032d7b01a5fe1535d9446b629`, non-yanked, tag `eggpack-manifest-v0.1.0`; closure `eggstack/eggpack: plans/closure/release-manifest/003-status.md`.

Primary class: packaging / registry promotion / manual publication

## 1. Why this milestone is authorized now

M004a closed on 2026-10-01 and proved the minimum registry-resolvable publication set mechanically. It left M004 blocked on exactly two things:

1. the Eggpack-owned prerequisite `eggpack-manifest 0.1.0`, and
2. Eggup's own `eggup-acquisition 0.1.2` → `eggup-eggfetch 0.1.2` → `eggup-eggpack 0.1.2` publications.

Item 1 is satisfied: `eggpack-manifest 0.1.0` is published, and its `src/lib.rs` is byte-identical to the consumer-qualified Git source Eggup pinned (`678bbf04f5a02827003a1d9ab83ba4f0e6360e41`). The M004a compatibility-incident branch is therefore not triggered and the adapter needs no semantic requalification.

Item 2 is this milestone's work. A maintainer has explicitly authorized the release process. M004a required that a separately authorized implementation plan exist before any of it ran; this is that plan.

## 2. What is authorized

Exactly three first publications, in this load-bearing order:

```text
eggup-acquisition 0.1.2   (leaf; no Eggup deps)
        |
        v
eggup-eggfetch 0.1.2      (requires eggup-acquisition ^0.1.0; carries the 0.1.2 FetchLimits migration)
        |
        v
eggup-eggpack 0.1.2       (pins eggup-core/eggup-archive/eggup-acquisition =0.1.2 + eggpack-manifest =0.1.0)
```

`eggup-core 0.1.2` and `eggup-archive 0.1.2` are already published and are **not** republished: `git diff v0.1.2..HEAD -- crates/eggup-core crates/eggup-archive` is empty, so their registry bytes already match this source.

`eggup-curl`, `eggup-service`, and `eggup-transport-footprint` are **out of scope**. M004a's graph proof showed no edge from them into the published candidate, and `eggup-transport-footprint` is `publish = false` by design. They are not published by this milestone.

No Eggsact source is changed. Eggsact's Git→registry migration stays separately authorized in `eggstack/eggsact`.

## 3. Production changes

The only production change is the mechanical promotion M004a identified, plus the metadata truthfulness it flagged:

- `crates/eggup-eggpack/Cargo.toml`: remove `publish = false`; replace the immutable-Git `eggpack-manifest` dependency with the registry dependency `eggpack-manifest = "=0.1.0"`; backfill `homepage`, `documentation`, `authors`, `keywords`, and `categories` so the published metadata matches the other published Eggup crates.
- `crates/eggup-eggpack/README.md`: the final paragraph currently states the crate "is unpublished and pins the currently unpublished `eggpack-manifest` crate to an immutable Eggpack revision". That is rendered on crates.io and becomes false at publication, so it is corrected to the published registry state.
- `Cargo.lock`: refresh the yanked `yoke-derive 0.8.3` entry to `0.8.4` (M004a info finding). `yoke-derive` is a transitive dependency of `eggfetch-core` and so only affects the `eggup-eggfetch` graph.
- root `CHANGELOG.md`: the `## 0.1.2` section currently states that acquisition/eggfetch/curl/service/eggpack "were **not** published as 0.1.2" and that `eggup-eggpack` "remains `publish = false` by design". Completing a partially-shipped workspace version makes those lines false, so the section is updated to record the completed 0.1.2 publication set with dates and checksums. Nothing is erased and no earlier statement is silently dropped; the correction is explicit.

No `src/` change, no public API change, no version change. The workspace is already at `0.1.2`, and `eggup-eggpack`'s `=0.1.2` pins to `eggup-core`/`eggup-archive` align with what is already on the registry, so no version bump is performed or needed.

## 4. Invariants

- Publish exactly `eggup-acquisition`, `eggup-eggfetch`, and `eggup-eggpack` at `0.1.2`. Do not publish any other workspace crate.
- Do not move, retag, or delete the existing `v0.1.1` or `v0.1.2` tag. Tags are immutable.
- No `--allow-dirty` on any package or publish command. The tree must be clean at every upload.
- No weakenable exact-pin: `eggup-eggpack` keeps `=0.1.2` on core/archive/acquisition and `=0.1.0` on `eggpack-manifest`.
- Do not vendor, duplicate, or reimplement the manifest parser to avoid the registry dependency.
- No API, schema, or semantic change to satisfy packaging. Integrity remains SHA-256 checksum evidence only; no authenticity or signature claim.
- Do not change `eggup-core`, `eggup-archive`, `eggup-curl`, or `eggup-service` source.
- Publication is a manual maintainer action. Add no CI publication workflow and no release automation.
- Never print, log, commit, or record the crates.io credential.
- Each published version is immutable. Never plan an overwrite; a post-publication defect takes a new version through the normal process.
- Keep Eggup's producer boundary: this milestone publishes Eggup's own consumer crates. It does not change what Eggpack owns.

## 5. Tag and GitHub release policy for this milestone

The workspace `v0.1.2` tag already exists at `e8e07eb538d0eef18ea4cb4ace3bb905c316da72` and a GitHub Release `0.1.2` already documents the core+archive pair. A second `v0.1.2` tag is impossible and the existing one is not moved.

Verified source identity behind that tag: `git diff v0.1.2..HEAD -- crates/eggup-acquisition crates/eggup-eggfetch` is empty, so the published `eggup-acquisition` and `eggup-eggfetch` bytes are exactly the `v0.1.2` tag content. `eggup-eggpack` has changed since that tag by the M003a caller-bound destination seam, so it is **not** reproducible from `v0.1.2`.

Decision: append the three crates to the existing GitHub Release `0.1.2` notes, recording the completed publication set with checksums, and state plainly that `eggup-eggpack 0.1.2` is not reproducible from the `v0.1.2` tag. crates.io's `.cargo_vcs_info.json` carries the exact publication commit for each of the three versions, so per-crate auditability does not depend on the tag.

## 6. Work packages

### WP1 — Register this plan

Register M004 in the interoperability roadmap and the active planning registry before implementation, so the milestone has a status before its irreversible steps.

### WP2 — Manifest promotion and metadata truth

Apply §3. Confirm `cargo package -p eggup-eggpack` now succeeds where it failed under M004a with "no matching package named `eggpack-manifest`", and that the packaged manifest contains no `git`/`path` source.

### WP3 — Local and hosted qualification

Run `scripts/check-local.sh`, MSRV, `cargo doc`, and for each of the three crates `cargo package --locked` and `cargo publish --locked --dry-run`. Record exact outputs, packaged file lists, and `.crate` checksums. Require hosted CI green on the release-prep commit before any upload.

### WP4 — First-publication gate

Immediately before each upload, prove the exact version is absent, confirm publisher authority is the intended Eggstack account, and reconfirm a clean tree on the same commit. Verify the previous crate in the order is already visible before starting the next.

### WP5 — Publish in order

`cargo publish -p eggup-acquisition --locked`, verify; then `cargo publish -p eggup-eggfetch --locked`, verify; then `cargo publish -p eggup-eggpack --locked`, verify. After each, read the crates.io API for exact version, non-yanked state, checksum, and `published_by`.

### WP6 — Registry-only consumer proof

From temporary external projects outside this repository, with only registry dependencies and no `[patch]`, path, or git override:

- adapter-only: `eggup-eggpack = "=0.1.2"` resolves, compiles, and passes a bounded `project_json` / default-destination / caller-bound-destination smoke;
- Eggsact-shaped: `eggup-core =0.1.2`, `eggup-acquisition =0.1.2`, `eggup-eggfetch =0.1.2`, `eggup-eggpack =0.1.2` resolves with one registry source per package.

Confirm no `git+` or `path =` entry in either lockfile. Remove both projects after.

### WP7 — Release notes and closure

Append the completed set to GitHub Release `0.1.2`. Write `plans/closure/eggpack-manifest-interoperability/004-status.md` and reconcile roadmap/registry status.

## 7. Stop conditions

Stop and write a corrective before publishing further if any of these holds:

- `cargo package` or `publish --dry-run` for any of the three crates fails for a reason requiring a code, API, or exact-pin change;
- any exact `0.1.2` version already exists with different bytes, or a package name is owned by a different account;
- the hosted matrix is not green on the release-prep commit;
- the packaged `eggup-eggpack` manifest still shows a `git` or local `path` source for any Eggup or Eggpack edge;
- the registry-only smoke cannot resolve the exact published graph;
- publishing would require weakening unknown-value handling, bounded limits, or integrity-only framing;
- publication credentials or ownership cannot be established safely;
- a later crate's preconditions are not yet visible after the previous upload.

Publication is not transactional. A partial state is acceptable only if each published version is individually correct and the remainder is completed in order; it is not acceptable to publish a crate whose exact-pinned dependencies are not yet registry-visible.

## 8. Acceptance criteria

| Requirement | Acceptance evidence |
|---|---|
| Eggpack prerequisite satisfied and identity preserved | `eggpack-manifest 0.1.0` published; published `src/lib.rs` byte-identical to the `678bbf04` consumer-qualified source; no adapter requalification needed |
| Exactly three crates published, in order | three `cargo publish` invocations with registry verification after each; no other package uploaded |
| No republish of core/archive | `git diff v0.1.2..HEAD -- crates/eggup-core crates/eggup-archive` empty; no publish command for either |
| Adapter manifest fully registry-based | packaged `eggup-eggpack` manifest resolves all edges from crates.io; no `git`/`path` source; `=0.1.2` and `=0.1.0` pins intact |
| Package metadata complete and truthful | `homepage`/`documentation`/`authors`/`keywords`/`categories` backfilled; README and CHANGELOG no longer claim the crate is unpublished |
| Clean qualification | `scripts/check-local.sh` green, MSRV green, `cargo doc` clean, package + publish dry-run green for all three, no `--allow-dirty` |
| Hosted qualification | CI green on the release-prep commit across Stable, MSRV, macOS, and Windows lanes |
| Registry-only usability | external adapter-only and Eggsact-shaped projects resolve exact `=0.1.2` and pass smoke with no git/path source |
| Tags untouched | `v0.1.1` and `v0.1.2` unmoved and unchanged; GitHub Release `0.1.2` notes extended, not replaced |
| Bounded scope | no `eggup-curl`/`eggup-service`/`eggup-transport-footprint` publication; no Eggsact change; no publication automation added |
| Closure traceability | closure records per-crate commit, checksum, size, publisher, timestamp, commands, CI run, and the `eggup-eggpack`-not-in-tag caveat |

## 9. Handoff after closure

M004 closes the Eggpack-manifest interoperability seam's packaging half. The remaining consumer-side step is Eggsact's own Git→registry migration, which is separately authorized in `eggstack/eggsact` and is not claimed here.

Future work in this subsystem needs new evidence and a new plan. If a published `0.1.2` defect is discovered, do not attempt to overwrite: preserve the published record, and take a new version through the normal plan/closure process.
