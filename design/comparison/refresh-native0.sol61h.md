---
type: ImplementationAccount
title: Explicit retained executable effects for native seeded refresh
description: Completes the accepted provider effect seam without strengthening the stat heuristic or introducing a second walker.
resource: /design/comparison/refresh-native0.sol61h.md
status: verified-provider-checkpoint
generated: { by: "model:openai/gpt-6.1-sol#high", at: 2026-10-30 }
sources:
  - resource: "opencode://ses_e74387c29ffep4tyaHAb72eQ86"
    title: Implements the parent-selected opt-in retained executable correction, provider API migration and native consumer cutover
  - resource: "opencode://ses_e74f4db73ffebUUrvE3MbpCHWY"
    title: Commissions completing default refresh and accepts in-place executable correction using prepared metadata with separate successful-effect accounting
  - resource: /design/comparison/refresh0.sol61h.md
    title: Predecessor provider0.8 refresh and postcheck checkpoint and accepted syscall receipts
  - resource: /transfer/sync/src/listed/runner.rs
    title: Successful retained acceptance owns the opt-in effect and advances coverage only after success
  - resource: /transfer/sync/src/listed/selection.rs
    title: Pure lowering and live binding include the retained-attribute execution interpretation
  - resource: "file:///home/rektide/archive/jj-vcs/jj/lib/src/local_working_copy.rs"
    title: ExecBit::new_from_disk uses any Unix 0o111 bit; stock setter is not the preservation behavior selected here
  - resource: "file:///home/rektide/src/jj-waltz/design/coordinated-refill/refresh0.sol61h.md"
    title: Consumer default, native lifecycle, public schema and receiving verification account
---

# Selected provider addition

Provider **0.9.0** adds `RetainedAttributes::{Preserve,ManifestExecutable}` on
`ListedTarget`. Preserve is the explicit non-native setting. Native JW opts into
manifest executable conformance; generic `RefreshOwned` alone never enables it.
No comparator behavior or default check set changes from provider0.8.

The existing successful `KeepLeaf` acceptance stage uses the already prepared
regular target metadata and bound exact executable expectation. If any `0o111`
bit already matches, nothing is written. Otherwise one chmod sets/clears those
bits in place, preserving the other permission bits. The kernel performs no
stat/read-back, content read, inode installation or privacy audit for this effect.
Mutation is sync-owned, not a guard/check/acquisition callback. Context binding
includes the opt-in interpretation; source expectations and lowering remain pure.

`ListedOutcome.retained_permissions_set` counts successful actual metadata writes,
on success or failure. Ordinary `Progress`, its compact cold `Failure`, and saved
`WorkTotals` are unchanged. Retained desired coverage advances only after its correction
succeeds. Failed correction prevents complete coverage and keeps earlier progress.
The existing caller-owned stable-tree assumption still applies; no hostile-race
or alias-isolation guarantee is added. Unix source expectations still reject
unsupported platforms/kinds during preparation.

SyncParts and the explicit reconcile example transport keep **version4** from
provider0.8: postchecks remain bound per lane; listed-only effects do not enter
the ordinary cursor. Provider products and exact pins advance together, including
the three independent example receiving locks. Comparison-only parts are unchanged.

## Checks so far

- Workspace all-target check passed after the API/pin migration.
- Listed integration: **24 passed**, Nextest
  `0a161e67-37bc-49ef-a480-002675e490a2`, including opt-in/no-op/inode/mtime/mode
  preservation and real ENOENT correction failure with a successful metadata prefix.
- Sync library all-features: **24 passed**,
  `c88e528f-6d31-4b90-84a2-c8c6b0c8d423`, including binding and cursor barriers.
- Reconcile process suite first run: **19 passed, 1 failed**,
  `ebf63edf-d9bc-4181-a6d5-e37eb15edb2e`. The sole failure was its stale
  `parts.version == 4` assertion during an intermediate version5 design. Its focused
  correction passed (`80ded2a8-df15-433b-a048-24c15c091c94`). The final narrower
  accounting restores version4, avoiding that unnecessary transport change.

An intermediate generic `Progress` counter made `Failure` grow from 120 to 128
bytes and tripped strict Clippy's error-size threshold. Rather than suppress the
budget or box every ordinary failure, the final seam keeps its actual count on
`ListedOutcome`. This is why no generic work/transport schema was enlarged.

The predecessor's whole-workspace gates and syscall witness are accepted evidence,
not rerun by rote. This addition does not change the installer/postcheck kernel.
No deployment, publication, representative workload or timing campaign is claimed.

### Final narrow-seam receipts

- Listed all-features integration: **24 passed**,
  `ec91ed1f-4c24-4ab2-965c-76c9b82b4ff3`. This is the final listed-only counter
  shape, with all modes, native opt-in and actual failure-prefix evidence.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: passed
  after narrowing the counter, with no large-error suppression or boxing change.
- `cargo test -p diff-dirs-transfer-sync --doc --all-features`: **11 passed**.
- Both walk and cursor examples passed all-target receiving checks and refreshed
  their exact locks; reconcile refreshed its receiving lock during the process
  checks above. Its final ordinary transport remains version4.
- `cargo fmt --all -- --check`: passed.
