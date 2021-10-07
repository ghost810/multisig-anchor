## Instantiation

To create the multisig, you must pass in a set of `HumanAddr` with a weight
for each one, as well as a required weight to pass a proposal. To create
a 2 of 3 multisig, pass 3 voters with weight 1 and a `required_weight` of 2.

Note that 0 *is an allowed weight*. This doesn't give any voting rights, but
it does allow that key to submit proposals that can later be approved by the
voters. Any address not in the voter set cannot submit a proposal.

## Execution Process

First, a registered voter must submit a proposal. This also includes the
first "Yes" vote on the proposal by the proposer. The proposer can set
an expiration time for the voting process, or it defaults to the limit
provided when creating the contract (so proposals can be closed after several
days).

Before the proposal has expired, any voter with non-zero weight can add their
vote. Only "Yes" votes are tallied. If enough "Yes" votes were submitted before
the proposal expiration date, the status is set to "Passed".

Once a proposal is "Passed", anyone may submit an "Execute" message. This will
trigger the proposal to send all stored messages from the proposal and update
it's state to "Executed", so it cannot run again. (Note if the execution fails
for any reason - out of gas, insufficient funds, etc - the state update will
be reverted, and it will remain "Passed", so you can try again).

Once a proposal has expired without passing, anyone can submit a "Close"
message to mark it closed. This has no effect beyond cleaning up the UI/database.

## Running this contract

You will need Rust 1.44.1+ with `wasm32-unknown-unknown` target installed.

You can run unit tests on this via: 

`cargo test`

Once you are happy with the content, you can compile it to wasm via:

```
RUSTFLAGS='-C link-arg=-s' cargo wasm
cp ../../target/wasm32-unknown-unknown/release/contract-multisig.wasm .
ls -l contract-multisig.wasm
sha256sum contract-multisig.wasm
```