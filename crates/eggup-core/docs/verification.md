# Integrity and candidate validation contract

`hash_file` streams local bytes through native SHA-256. The sidecar parser
accepts exactly one digest entry, rejects malformed or ambiguous input, and
binds an optional exact filename. A digest is integrity evidence only; it is
not an authenticity or signature claim.

The public transaction phase is deliberately ordered:

```text
PreparedTransaction
        | verify_integrity()
        v
VerifiedTransaction
        | validate(&CandidateValidator)
        v
ValidatedTransaction -- commit() --> TransactionReceipt
```

Candidate commands use literal argv vectors, clear inherited environment, null
stdin, a controlled working directory, a wall-clock deadline, bounded stdout
and stderr, and kill/reap on timeout or output overflow. `ExactIdentityValidator`
and `CrossMemberAgreementValidator` are consumer-policy-neutral helpers; a
consumer can supply a custom `CandidateValidator` for a different identity
contract. No fallback or release ordering is selected by core.

