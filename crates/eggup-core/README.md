# eggup-core

`eggup-core` is the policy-neutral local substrate for verified, multi-artifact
updates. It is intentionally transport-neutral: callers acquire artifacts and
choose release, authenticity, and service policies; the core provides bounded
local validation, staging, and transaction mechanics.

The crate is foundation-stage software. The initial release establishes the
workspace boundary and test contract; it does not yet implement a production
updater.

