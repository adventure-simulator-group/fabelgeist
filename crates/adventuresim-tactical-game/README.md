# adventuresim-tactical-game

The tactical game on [bevy_netahoy](https://github.com/wtfdemon/bevy_netahoy):
hand-to-hand fighters stepped at 20 Hz, predicted on their owner's client,
judged where each attacker saw the others, and drawn on the fabelgeist rig. It
plays over WebTransport and falls back to WebSocket where UDP is blocked.

## Run it

```bash
just tactical-game-server   # 127.0.0.1:5000 (WebSocket), :5001 (WebTransport), :5002 (offer)
just tactical-game-client   # a native client; start as many as you like
```

The server runs two bots (`BOTS=<n>` for another count). In the client, WASD
moves, a click captures the mouse to look, left click strikes (a second click
during a strike chains the combo), right click or Shift guards, Space jumps,
and Esc releases the mouse. `TRANSPORT=ws|wt` pins a transport.

In a browser, the client reads its server from the page's address
(`ServerEndpoints::from_page`): over plain http the page's host on the default
ports; over https the `ws` and `wt` paths beside the page, behind one proxy.

## How it is built

- `fighter.rs`: `Fighter`, the game half of a player, and the one step every
  peer runs on each command: prediction, replay and the server.
- `moves/`: one module per move, timed in ticks. `strike.rs` reaches the
  nearest fighter in front of the attacker where the attacker saw them;
  `guard.rs` turns a strike from the front into a shove.
- `combat.rs`: the events a step reports (`CombatEvent`), the server's verdict
  (`StrikeOutcome`) and the injury it leaves (`Vitals`).
- `server/`: joins, strike resolution, getting up after being downed, bots.
- `client/`: prediction and visuals, input, the rig's animation, impacts, the
  camera and the injury rings.

The crate docs (`cargo doc -p adventuresim-tactical-game --open`) walk
through a tick and give the checklist for adding a move.

## Check it

```bash
cargo nextest run -p adventuresim-tactical-game   # moves, outcomes, a headless bot fight
cargo run -p adventuresim-tactical-game --example rig_check
```

`rig_check` plays every motion file on the rig without a GPU and prints where
the head and feet land: a motion file that doesn't fit the rig shows there as
a body off its feet.
