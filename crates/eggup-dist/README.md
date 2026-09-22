# eggup-dist

Versioned distribution-contract schema for release-time tooling.

A `DistributionContract` describes one product's release layout once —
target triples, platform aliases, asset file names, checksum sidecar names,
archive members, and install names — so runtime updaters, CI, and
shell/PowerShell bootstrap installers cannot silently diverge.

It does **not**:

- download release assets or query GitHub/crates.io;
- select release versions (`version` is opaque input to name expansion);
- compute or verify checksums (sidecars are integrity metadata, never
  authenticity);
- extract archives (it validates the member contract later extraction obeys);
- generate installers, run shells, publish releases, or manage services.

## Schema v1

Every document starts with explicit `schema_version = 1`. Unknown future
major versions fail with a typed `UnsupportedVersion` error. Within v1,
unknown fields are rejected (typo safety) and optional fields have
deterministic defaults.

```toml
schema_version = 1

[product]
id = "eggsact"

[[targets]]
triple = "x86_64-unknown-linux-gnu"
aliases = ["linux-x64"]

[targets.asset]
kind = "direct"
asset = "{product}-{version}-{target}"
install = "{product}"

[targets.checksum]
sidecar = "{asset}.sha256"
```

### Asset forms

A small tagged enum, not a generic package language:

- `direct`: one asset file → one member (`asset` + `install`).
- `bundle`: multiple asset files form one logical release (`entries[]`
  of `asset` + `install`; CodeGG-like three-runfile layout).
- `archive`: one archive file contains multiple required members
  (`asset` + `members[]` of literal `source` + templated `install`;
  Egress-like layout).

### Name templates

Small placeholder vocabulary only:

- `{product}`, `{version}`, `{target}`, `{alias}`;
- `{asset}` additionally for checksum sidecars (the expanded asset file).

Unknown placeholders fail; `{alias}` requires lookup via that alias;
`{asset}` requires a checksum context. No shell, conditionals, formatting
code, escaping, nested/double braces, or environment expansion. Template
literals use only ASCII letters, digits, `-`, `_`, and `.`. Malformed braces
and placeholders fail while parsing the contract.

After expansion, release assets and checksum sidecars must have unique names
within one target/version; install names must also be unique. Exact and
ASCII-case-only collisions fail with `DistError::NameCollision`, including
cross-collisions between an asset and a sidecar. ASCII case folding is
portable and locale-independent; Unicode case folding is not performed.
Expansion never returns a partially valid or ambiguous target.
Asset/install/sidecar names are flat file names (no `/` or `\`). Versions
remain opaque but filesystem-safe (non-empty, no separators/controls).

### Target resolution

Triples are canonical Rust triples; aliases are globally unambiguous.
Duplicate triples, duplicate aliases to different targets, and aliases
colliding with any triple are rejected. Unknown targets fail with
`UnknownTarget` instead of guessing.

### Archive safety

Member `source` paths are literal (no templates), relative, normalized, and
traversal-free. Rejected: absolute paths, `..`, `.`, empty components,
drive prefixes, backslashes, and duplicate normalized paths. M001 validates
the contract; extraction belongs to later milestones.

### Checksum contract

Sidecar naming is explicit/derivable (for example `{asset}.sha256`).
The model calls this integrity metadata, never signature/authenticity.

## Layout

- `src/lib.rs`: typed model, parser, validation, expansion.
- `tests/fixtures/`: `simple-direct.toml` (eggsact/stegoeggo-like),
  `codegg-bundle.toml`, `egress-archive.toml`.
- Product-specific fixtures live under `tests/`, never as generic constants.

## Non-goals

Version ordering, hosting/URLs, install directories, privilege policy,
service definitions, and signing standards remain consumer-owned.
No existing installer is replaced in M001.
