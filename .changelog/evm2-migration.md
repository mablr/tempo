---
tempo-chainspec: minor
tempo-evm: minor
tempo-node: patch
tempo-payload-builder: patch
tempo-precompiles: minor
tempo-primitives: minor
tempo-transaction-pool: patch
---

Moved EVM execution from revm and alloy-evm to evm2 and removed the `tempo-revm` crate. Reth now owns the bundle state types, so `tempo-evm`, `tempo-precompiles`, `tempo-primitives` and `tempo-chainspec` expose evm2 account, bytecode, block-environment and precompile result types in place of their revm equivalents.
