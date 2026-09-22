# Closure and Verification Records

Closure records determine whether an Eggup milestone is actually complete.

## Layout

```text
closure/<subsystem>/NNN-status.md
```

Use the same milestone number as the source implementation plan.

## Required structure

A closure record MUST include:

- status: closed, conditionally closed, corrective pass required, or blocked;
- source plan and roadmap;
- reviewed repository baseline;
- implementation commits/PRs;
- executive finding;
- requirement-to-evidence matrix;
- production implementation evidence;
- exact commands run and results;
- invariant review;
- failure/rollback/recovery review;
- compatibility and migration review;
- security review;
- documentation/operations evidence;
- unresolved findings with severity;
- roadmap disposition;
- registry updates.

A milestone MUST NOT be marked closed on compilation or happy-path tests alone.

For platform-sensitive work, report native evidence separately by Linux, macOS, and Windows. Do not infer one platform from another.
