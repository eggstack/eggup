# Distribution and Bootstrap Roadmap

Status: M001-M002 closed; M003 validators ready for handoff

Long-term references:

- `plans/000-long-term-specification.md#14-bootstrap-installers`
- `plans/000-long-term-specification.md#42-eggup-dist`

Related ADR:

- `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md`

## 1. Purpose and ownership boundary

This subsystem reduces drift among release workflows, runtime updaters, shell/PowerShell bootstrap installers, supported target lists, and asset/checksum naming.

It owns release-time schema/tooling, not release cadence or hosting.

## 2. Work classification

### Invariants

- bootstrap and runtime target/asset contracts cannot silently diverge;
- installer origins remain consumer-controlled;
- unsupported targets fail rather than guess;
- checksum validation precedes candidate execution/commit;
- generated/validated scripts never introduce implicit privilege escalation.

### Capabilities

- describe one product's release layout once;
- validate a release asset set;
- generate or validate POSIX/PowerShell installers;
- test installers against local fixtures.

### Infrastructure

- DistributionContract schema;
- target mappings;
- asset/member layouts;
- checksum-manifest rules;
- generator/conformance CLI/library.

### Polish

- templates;
- release diagnostics;
- docs snippets.

## 3. Non-goals

- automatic GitHub release creation;
- package manager publication;
- signing standard selection;
- forced identical installer UX across products.

## 4. Current state evidence

The roadmap's first-consumer evidence gate is satisfied: eggsact and stegoeggo are live on the same Eggup 0.1.0 core/acquisition contracts without generic API changes.

M001 established TOML schema v1. M002 then closed the post-closure ambiguity findings by enforcing strict template grammar and portable expanded-name uniqueness. The corrected v1 contract is now stable enough for M003 conformance validators.

Gregg already uses a machine-readable target table checked against runtime constants. Eggsearch, eggsact, stegoeggo, Egress, and CodeGG duplicate shell/PowerShell target mapping and checksum logic. CodeGG demonstrates a multi-runfile bundle installer; Egress demonstrates archive-based pair installation.

## 5. Target architecture

`eggup-dist` consumes a checked-in product-owned distribution contract and produces validation/generation outputs. Runtime consumers can generate or test their target/asset tables against the same source without depending on eggup-dist at runtime.

## 6. Dependency graph

```text
core + acquisition contracts proven by consumers
              |
              v
M001 DistributionContract schema [closed]
              |
              v
M002 schema uniqueness/template grammar corrective
              |
              v
M003 installer/release validators
              |
              v
M004 generator/templates + two consumer adoptions
```

## 7. Milestones

### M001 — Versioned DistributionContract schema

Plan: `plans/implementation/distribution-bootstrap/001-distribution-contract-schema.md`.

Define schema v1 from real simple, CodeGG bundle, and Egress archive/pair layouts. Parse and structurally validate target/alias, asset, checksum, and archive-member mappings. No network, extraction, or installer generation.

### M002 — Schema uniqueness and template grammar corrective

Plan: `plans/implementation/distribution-bootstrap/002-schema-uniqueness-template-corrective.md`.

Reject malformed template grammar and any expanded release/install namespace collision before v1 becomes a downstream validation contract.

### M003 — Release and installer conformance validators

Plan: `plans/implementation/distribution-bootstrap/003-release-installer-conformance-validators.md`.

Validate target mapping, release asset completeness, checksum names, caller-supplied archive member inventories, and runtime/bootstrap mapping observations against corrected v1 without network, extraction, or language parsing.

### M004 — Generator/templates and two consumer adoptions

Optionally generate installer bodies or checked fragments once validation proves the schema is sufficiently expressive.

## 8. Cross-cutting requirements

Schema evolution requires compatibility/versioning. Generated scripts must remain readable and testable. Product-specific fallback/source-install messages remain product policy.

## 9. Verification strategy

Golden manifests, local fake releases, shellcheck/PSScriptAnalyzer where available, installer idempotency, checksum mismatch, interrupted replacement, unsupported target, and bundle rollback.

## 10. Risks and decision points

Generating complete installers too early may create an opaque template framework. Prefer conformance checking first if it captures most duplication with less machinery.

## 11. Completion definition

At least two consumers derive or validate runtime and bootstrap asset policy from one machine-readable contract.

## 12. Milestone status

| Milestone | Status | Implementation plan | Closure record | Blockers |
|---|---|---|---|---|
| M001 | closed; post-closure findings feed M002 | `plans/implementation/distribution-bootstrap/001-distribution-contract-schema.md` | `plans/closure/distribution-bootstrap/001-status.md` | — |
| M002 | closed | `plans/implementation/distribution-bootstrap/002-schema-uniqueness-template-corrective.md` | `plans/closure/distribution-bootstrap/002-status.md` | — |
| M003 | **ready for handoff** | `plans/implementation/distribution-bootstrap/003-release-installer-conformance-validators.md` | — | corrected schema M002 closed |
| M004 | planned / blocked | — | — | dist M003 |
