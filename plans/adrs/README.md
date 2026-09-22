# Architecture Decision Records

Use ADRs for durable Eggup decisions that affect multiple milestones or consumers.

## Naming

```text
ADR-NNNN-short-title.md
```

## Status lifecycle

```text
proposed -> accepted -> deprecated or superseded
         `-> rejected
```

Accepted ADRs are historical. Supersede rather than rewrite.

## Required content

- status and date;
- decision owners;
- context;
- decision drivers;
- considered options;
- precise decision;
- consequences;
- compatibility/migration effects;
- security/reliability implications;
- verification;
- supersession.

An ADR is normally required for a new public compatibility contract, trust model, transaction guarantee, service ownership rule, persistent on-disk format, or durable dependency/layer decision.
