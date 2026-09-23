# Distribution and Bootstrap M004 — Closure and Verification Record

Status: closed

Source plan: `plans/implementation/distribution-bootstrap/004-retire-eggup-dist-authority.md`

Source roadmap: `plans/subsystems/distribution-bootstrap-roadmap.md`

Reviewed Eggup baseline: `dbfefd958dddd3ed833bf2e30459ca5e6b43d344`.

Eggpack Contract M002 closure: `82f799f3d971b2999ac14c2d8fc1b965370e0f58` (`eggstack/eggpack`, commit `docs(planning): close contract conformance m002`).

Eggup M003 predecessor implementation/closure: `9941c58d7039410c728860f9e4e382881d4ccf54` / `4169c8021b447fe73c8ee3ea71a80a535c940f54`.

Implementation commit: `bc25885bd41b86bfdf2f32d1e42856e00829cd7a` (`refactor(dist): retire eggup producer authority`).

## Executive finding

M004 is complete. Eggpack Contract M002 independently qualified the Eggup M003 schema-v1 and conformance surface before deletion. Eggup then removed the unpublished `eggup-dist` crate and its fixtures, removed it from the workspace and lockfile, and updated active planning to record the authority transfer. Historical M001-M003 implementation and closure records remain intact. Eggup now has no active producer-side distribution/bootstrap implementation or roadmap.

## Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Eggpack M002 closed with equivalent schema/conformance behavior | Eggpack closure `plans/closure/contract-conformance/002-status.md` at `82f799f3d971b2999ac14c2d8fc1b965370e0f58`; explicit API/behavior matrix and direct, CodeGG bundle, and Egress archive fixtures | passed |
| No schema-v1 meaning or M003 behavior lost | Eggpack M002 closure compares expected files, release/member inventories and bounds, extras policy, typed observations, alias/canonical mapping, findings/report bounds, TOML fixtures, and direct/bundle/archive behavior; it reports schema-v1 unchanged and adds a crossed bundle-entry regression check | passed |
| No active consumer depends on `eggup-dist` | Search of Eggup plus known consumers eggsact, stegoeggo, and eggsearch found no active dependency/import; `Cargo.toml` no longer lists the crate | passed |
| Crate is absent from the active workspace | `crates/eggup-dist` source, manifest, README, integration tests, and fixtures deleted; `Cargo.toml` membership and `Cargo.lock` package/dependency entries removed | passed |
| Runtime crates remain independent of Eggpack | Workspace manifests/dependency tree show no Eggpack dependency; no runtime implementation changed | passed |
| Historical provenance remains traceable | M001-M003 plans and closure records retained; M004 closure links both Eggpack M002 and Eggup M003 records | passed |
| Active planning no longer authorizes Eggup producer work | ADR-0004, long-term specification/roadmap, planning process, registry, distribution roadmap, architecture overview, and M004 plan updated; subsystem marked archived/transferred | passed |

### Predecessor-to-Eggpack behavior comparison

| Eggup M003 surface | Eggpack M002 evidence | Result |
|---|---|---|
| Schema-v1 parse/expand and target/alias semantics | M002 is additive to M001; closure states schema-v1 behavior is unchanged | passed |
| Expected direct, bundle, and archive release files | Complete expected-file sets and positive/negative completeness tests for all layouts | passed |
| Bounded release and archive-member inventories; duplicate, case-collision, traversal, and extras rules | Constructor boundary tests, path safety cases, and AllowExtras/Exact cases | passed |
| Typed direct/bundle/archive observations and exact mapping comparisons | Copied TOML goldens, round-trip tests, alias resolution, and mapping drift cases | passed |
| Structured deterministic reports and finding bounds | Stable ordering, typed finding assertions, 512-finding cap and bounded fields | passed |
| Direct, CodeGG bundle, and Egress archive fixtures | `golden_observation_fixtures_conform_for_all_layouts` plus conformance matrices | passed |

The Eggpack closure additionally verifies that bundle asset/sidecar/install relationships are compared per entry; this closes a false-acceptance gap not covered by Eggup's predecessor tests.

## Production implementation evidence

- Removed `crates/eggup-dist/{Cargo.toml,README.md,src/lib.rs,tests/**}`.
- Removed the workspace member from root `Cargo.toml` and all now-unused lockfile packages from `Cargo.lock`.
- `scripts/check-local.sh` required no special-case edit: it qualifies the workspace generically and contained no `eggup-dist` reference.
- No replacement Eggup distribution crate, Eggpack runtime dependency, manifest adapter, installer generator, publication automation, or runtime semantic change was introduced.
- No published consumer crate depended on `eggup-dist`; its crate was unpublished.

## Exact verification commands and results

Environment: Darwin x86_64; `rustc 1.98.1`, Cargo 1.98.1, Rust 1.89.0 installed.

Passed:

```text
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked  # 159 passed, 9 suites
cargo doc --workspace --no-deps --locked
cargo +1.89.0 check --workspace --all-targets --locked
./scripts/check-local.sh  # passed; includes fmt, clippy, workspace tests, docs, cargo tree
cargo package -p eggup-core --locked --allow-dirty  # passed, 19 files
cargo package -p eggup-acquisition --locked --allow-dirty  # passed, 6 files
cargo package -p eggup-service --locked --allow-dirty  # passed, 7 files
cargo package -p eggup-eggfetch --locked --allow-dirty --no-verify  # archive packaging passed, 6 files
git diff --check
```

The ordinary verifier form `cargo package -p eggup-eggfetch --locked --allow-dirty` was attempted and failed while verifying the packaged archive: it resolves the already-published `eggup-acquisition 0.1.0`, which lacks the newer `FetchLimits::{effective,validate}` and owner-private promotion helpers required by eggfetch. This is the known publication-order limitation recorded in acquisition M003/M004, not a workspace source failure. The tarball packages successfully with `--no-verify`; the local workspace check/test/doc and package qualification of the dependency crate all pass. No release or publication was authorized or performed.

Static dependency searches:

- `git grep -n -E 'eggup-dist[[:space:]]*=' -- ':!plans/**' ':!CHANGELOG.md'` returned no active code/manifest references.
- `rg -n "eggup-dist"` across eggsact, stegoeggo, and eggsearch returned no matches.
- `git grep -n "eggup-dist"` returns historical provenance/documentation references only; no deleted source or active dependency remains.

Hosted Eggup CI was not run for this local change. Native test evidence is Darwin x86_64; no Windows runner behavior is inferred from it.

## Invariant, failure, and recovery review

- No qualified behavior was removed before Eggpack M002 closure; its exact closure SHA is recorded above.
- Eggup core/acquisition/service gain no Eggpack dependency, and explicit non-Eggpack artifact deployment remains supported.
- Producer-side target naming, release/member inventories, mapping validation, packaging, and publication now have one active authority in Eggpack.
- Checksums remain integrity evidence; authenticity and release selection policy remain outside this migration.
- No runtime mutation, rollback, service, acquisition, or transaction behavior changed. Failure/recovery semantics are not applicable to this source-only authority retirement.

## Compatibility, security, and documentation review

`eggup-dist` was unpublished, and searches found no Git/path or published consumer adopting it. Its removal therefore requires no crates.io compatibility migration. Historical schema-v1 plans, M003 source description, and closure evidence remain available for provenance. The detailed behavior is now linked to Eggpack Contract M002. No security-sensitive runtime code changed and no new execution, network, archive, or trust capability was added.

## Unresolved findings

- **Low, accepted / publication ordering:** isolated `eggup-eggfetch` package verification still resolves the already-published older `eggup-acquisition 0.1.0`. This pre-existing ordering limitation is documented in acquisition M003/M004; resolving it requires the separately directed lockstep crate publication, which is outside M004. Local workspace qualification and archive packaging passed.
- No M004-specific medium-or-higher finding remains open.

## Roadmap disposition and dependency transitions

- Distribution/bootstrap M001-M004 are complete historical work; the subsystem is archived/transferred to Eggpack. No further Eggup producer-distribution plan is authorized.
- Eggpack Contract M002's external gate is satisfied. That gate unblocked this M004 retirement; this retirement does not make another Eggup implementation newly ready.
- Service lifecycle M005 and Consumer Adoption CodeGG M005 were already ready for plan authoring and remain so.
- A future optional Eggup manifest-consumer adapter remains gated on a stable Eggpack ReleaseManifest contract. Egress remains blocked on archive update/extraction transaction semantics; Gregg remains blocked on footprint evidence. Those states are unchanged.
- `plans/registry.md`, `plans/subsystems/distribution-bootstrap-roadmap.md`, ADR-0004, the long-term documents, and this plan now record the completed transfer and remaining dependency state.

M004 is closed: Eggpack is the sole active producer authority, Eggup retains consumer-side deployment responsibilities, and predecessor evidence is preserved.
