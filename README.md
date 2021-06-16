# iota-voting

Simple example how a voting with tokens could happen when the data is included in the transaction payload. For this example the data needs to be `build` or `burn`.

Someone with the full Tangle history has to download a snapshot file, uncomment `get_messages_for_outputs` to create `snapshot_messages.bin` and share it with others.

Then everyone can download a snapshot for the same milestone index from any node and needs to get `snapshot_messages.bin`, then just run it with `cargo run` to see the voting result.

Output will look like

```bash
Network ID:                     6530425480034647824
Ledger index:                   51
VotingResult {
    iotas_voted_for_build: 4000000,
    iotas_voted_for_burn: 2000000,
    iotas_not_voted: 2779529277277761,
    amount_votes_for_build: 4,
    amount_votes_for_burn: 2,
    amount_not_voted: 3,
}
```
