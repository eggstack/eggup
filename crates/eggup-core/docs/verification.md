# Integrity and candidate validation contract

`hash_file` streams local bytes through native SHA-256. The sidecar parser
accepts exactly one digest entry, rejects malformed or ambiguous input, and
binds an optional exact filename. A digest is integrity evidence only; it is
not an authenticity or signature claim. There is no authenticity verifier in
this crate, and no public API promises one: every commit requires a verified
SHA-256 digest for every member, and `IntegrityStatus::NotRequired` can never
reach candidate execution or commit.

The public transaction phase is deliberately ordered:

```text
PreparedTransaction
        | verify_integrity()
        v
VerifiedTransaction
        | validate(&CandidateValidator)
        v
ValidatedTransaction -- commit(ownership) ---------------------> TransactionReceipt
        `-- commit_with_post_commit(ownership, policy, check) --> TransactionReceipt
```

`validate` rejects any member without `Verified` integrity evidence.
Both commit paths re-hash every staged member under lock against the
digest recorded at verification time; bytes altered by candidate execution or
by a concurrent writer fail with `StageRevalidation` before live mutation.
The post-commit path then holds the lock and backup through one caller-owned
check and resolves a failed check using explicit `KeepInstalled` or
`RollBack` policy.

Candidate commands use literal argv vectors, clear inherited environment, a controlled working
directory, an unconditional wall-clock deadline, bounded stdout and stderr, and kill/reap on
timeout or output overflow. Stdin is null unless the caller attaches a bounded payload with
`CommandSpec::stdin_data`; an attached payload is bounded by `MAX_STDIN_BYTES` inclusively, a
larger payload is rejected with `Error::InvalidInput` before any child is spawned, and the
payload is written by a dedicated writer thread that the deadline bounds exactly as it bounds
the output readers. The deadline bounds the whole bounded execution, so a bounded call always
returns. The deadline has priority over the stdin writer's verdict, and the verdict is
classified by the instant at which the writer observed the write, not by the instant at which
the runner polled for it: a write failure observed before the deadline is an `Err`, a write
failure observed at or after it is the deadline outcome (`Ok` with `timed_out` set), and a
writer still in flight at the deadline is abandoned. An undelivered payload is never reported
as a success. The normative statement of that contract, of the deadline semantics, of what a
descendant holding the output pipe open means for the result, of the bounded stdin capability,
and of the stable `Error::Io` `operation` strings a downstream adapter may branch on, lives in
the `run_bounded` rustdoc in `src/candidate.rs` and in `architecture/core-transaction.md`.
`ExactIdentityValidator` and
`CrossMemberAgreementValidator` are consumer-policy-neutral helpers; a consumer can
supply a custom `CandidateValidator` for a different identity contract. No fallback or
release ordering is selected by core.
