# Eggpack Manifest Interoperability M001 Closure

Status: closed

Starting repository SHA: `6e1ea4fc7e2995c4ab26c256afdc93ad300bd4c0`

Implementation commit: `5fbb66853bdad59aaf2bd3c7bb43a43492d0b6ef`

External baseline: Eggpack `eggpack-manifest` pinned to immutable revision `678bbf04f5a02827003a1d9ab83ba4f0e6360e41`, with corrected interoperability M001a closure at that baseline.

## Requirement-to-evidence matrix

| Requirement | Evidence |
|---|---|
| Optional leaf adapter; lower crates remain Eggpack-independent | `eggup-eggpack` depends on `eggup-core`, `eggup-acquisition`, and only `eggpack-manifest`. `cargo tree` shows no Eggpack edge under core, acquisition, or service. |
| Exact target and direct/bundle relationship preservation | `project` validates and selects exact targets. Tests cover direct projection, exact target rejection, and the corrected three-artifact CodeGG bundle. |
| Archive remains extraction-required | Archive projections preserve archive/member facts. `materialize_artifact_set` rejects archive form; archive fixture projection is tested. |
| Exact request map and tightened byte limit | `bind_requests` requires exact keys, retains the caller request, rejects a smaller caller cap, and tightens the cap to the manifest size. |
| Exact acquired files, permissions, size, digest | Materialization requires exact maps, absolute regular non-symlink files, exact sizes, and caller permission intent. Manifest SHA-256 bytes are attached to `IntegrityRequirement::Sha256`. |
| Fixture provenance | Eight upstream fixtures are copied byte-for-byte and individually hashed in `crates/eggup-eggpack/tests/fixtures/README.md`; corrected bundle projection SHA-256 is `f0b227442e2a2a28dc4e2100301233f2698703d15251b6f2459e6387df6c53ce`. |
| Ownership and diagnostic boundaries | Projection is pure. No discovery, origin construction, extraction, install-root selection, ownership, service, or authenticity authority was added. Adapter errors omit raw URLs and file contents. |

## Verification evidence

Passed on a clean worktree at the implementation SHA:

- `cargo fmt --all -- --check`
- `cargo test -p eggup-eggpack --all-targets --all-features --locked` — 4 passed
- `cargo check --workspace --all-targets --locked`
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
- `cargo doc --workspace --no-deps --locked`
- `cargo +1.89.0 check --workspace --all-targets --locked`
- `cargo +1.89.0 test -p eggup-eggpack --all-targets --locked`
- `scripts/check-local.sh`
- `git diff --check`
- `cargo tree -p eggup-eggpack`, `-p eggup-core`, `-p eggup-acquisition`, and `-p eggup-service`

Hosted CI run [36004261228](https://github.com/eggstack/eggup/actions/runs/36004261228), attempt 2, passed all four lanes on implementation SHA `5fbb66853bdad59aaf2bd3c7bb43a43492d0b6ef`: stable fmt/clippy/workspace tests/docs, Rust 1.89 check, macOS workspace tests, and Windows workspace check.

`cargo package -p eggup-eggpack --allow-dirty --no-verify --locked` could not resolve `eggpack-manifest` from crates.io because that crate is unpublished. The adapter is explicitly `publish = false`; publication qualification is not claimed.

## Invariant, recovery, and compatibility review

The adapter reads metadata only before creating an `ArtifactSet`; it does not copy, chmod, rename, or delete acquired files. Later source mutation is handled by Core staging and integrity verification. SHA-256 remains integrity evidence, not authenticity. Bundles preserve paired relationships and order; partial/extra maps fail without returning an `ArtifactSet`.

This is additive optional functionality. Existing lower crates remain Eggpack-independent. No unresolved correctness or security finding remains.

## Downstream disposition

- M002 remains blocked on the long-term Phase 10 safe extraction contract.
- M003 is ready for plan authoring, which must select a real consumer currently owning duplicated manifest-to-update mapping. Eggsact's direct fixture is a candidate, not yet sufficient adoption evidence.
- M004 remains planned pending real consumer adoption and a publishable upstream `eggpack-manifest` version.


## Post-closure corrective registration

Subsequent review identified two qualification/closure issues that do not invalidate the current adapter architecture but do require a separate corrective before real-consumer adoption:

1. M001 closed with four adapter tests, while the source plan required a broader positive/negative compatibility matrix covering exact request/acquired/permissions maps, filesystem rejection cases, unknown schema behavior, corrected CodeGG bundle relationship fidelity, and dependency/authority checks.
2. Implementation commit `5fbb66853bdad59aaf2bd3c7bb43a43492d0b6ef` also introduced an unrelated path-only `eggup-service -> eggup-core` dependency. Service M005 later repaired the dependency with `version = "0.1.0"` at `84076a066e498f2e4ec4ed5472af4359c0b93af7` and qualified service package construction. Current main is repaired; the historical scope interaction remains part of the evidence record.

Corrective plan: `plans/implementation/eggpack-manifest-interoperability/001a-adapter-qualification-and-closure-hardening-corrective.md`.

The prior transition marking M003 real-consumer adoption ready is withdrawn until M001a closes. Historical M001 implementation/CI evidence above remains accurate for what was actually exercised at the time and is not rewritten. Corrective closure will be recorded separately at `plans/closure/eggpack-manifest-interoperability/001a-status.md`.

## M001a closure pointer

M001a closed at implementation `19935ec3610a5238af33a9d4f05a14925ceac25c` with 4 unit + 28 interoperability tests, no production-code change, and current-main service packageability re-verified. Closure: `plans/closure/eggpack-manifest-interoperability/001a-status.md`. M003 real-consumer adoption is returned to ready for plan authoring.
