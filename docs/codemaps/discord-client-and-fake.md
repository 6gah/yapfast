# The Discord client, and the fake server

Written from commit `4682fe5`.

## What it is

`crates/yapfast-discord` speaks to Discord: the gateway over a WebSocket,
the REST API, the cache of what came, and the service that turns both into
the core's events. `testing/fake_discord.ts` impersonates it, so nothing
here needs a real account.

- `yapfast-discord/src/lib.rs:1` — the crate's modules.
- `client.rs:1` — how Yapfast presents itself: the gateway and HTTP
  headers, matching a current desktop Firefox (`client.rs:32`), and
  `test_server()` (`client.rs:18`), which reads
  `YAPFAST_DISCORD_TEST_SERVER` and points everything at the fake with
  `TEST_TOKEN` (`client.rs:30`).
- `gateway.rs:1` — the connection, heartbeats, resumes, the payloads.
- `api.rs:1` — every endpoint the app calls; `event.rs:1` — what the
  gateway's events mean; `ready.rs:1` — the first payload, which says the
  servers, channels and people; `cache.rs:1` (with `cache/sealed.rs:1`) —
  what is kept between events.
- `service.rs:1` — the long-running thing the core talks to, with
  `service/session.rs:1` (the connection), `service/tracker.rs:1` (what
  changed, for the core's lists), `service/live.rs:1`, `service/marks.rs:1`
  (reads), `service/member_lists.rs:1`, `service/voice_join.rs:1`,
  `service/stream_openings.rs:1` (Go Live's start).
- The core's side: `yapfast-core/src/discord.rs:1` (what the window asks
  of Discord) and `discord_text.rs:1` with `discord_text/*.rs` (turning
  Discord's payloads into the core's rooms, people, messages, settings).
- The fake: `testing/fake_discord.ts:1` — an HTTP and WebSocket server with
  `/_control/*` routes; `testing/fake_discord_env.ts:1` — the file
  `--env PATH` reads, a JSON file or a TypeScript module whose default
  export is an `Env` (`fake_discord_env.ts:19`: `me`, `people`, `dms`,
  `groups`, `servers`, `favoriteGifs`); `testing/envs/example.json:1` uses
  every part, with the pictures beside it.

## Where state lives

| State | Home |
|---|---|
| What Discord sent, kept | `cache.rs:1`, in the service's memory |
| The connection and session | `service/session.rs:1` |
| The core's rooms, people, messages | `yapfast_core`'s `AppState` (`application.rs:482`) |
| The fake's world | Built from its env, in that process (`fake_discord.ts:1`) |
| Ids in the fake | From the order things appear, so stable across runs of one file (`fake_discord_env.ts:10`) |

## Flows

- **A command from the window**: a `Message` → an `Action`
  (`application.rs:1644`) → the core → the service → `api.rs` or
  `gateway.rs` → Discord (or the fake) → events back → the core's
  `AppState` changes → `Message::Events` → `app::sync`.
- **A new endpoint**: look it up at https://docs.discord.food first
  (`AGENTS.md:1`), add it to `api.rs:1`, give the fake the same route and
  its shape (`testing/fake_discord.ts:1`, with a test in
  `yapfast-discord/tests/`), then the core's handler.
- **A new part of the world for tests**: add it to `Env`
  (`fake_discord_env.ts:19`) and to `testing/envs/example.json:1`, then a
  test in `yapfast-discord/tests/fake_env.rs:1`.

## Gotchas

- The fake is only reached through `YAPFAST_DISCORD_TEST_SERVER`
  (`client.rs:18`); without it, the app wants a real login. `ui-session`
  sets it when started with `--discord` (`scripts/ui-session:1`).
- Ids in the fake are derived from order, not random
  (`fake_discord_env.ts:10`): a test may name a channel by its id as long
  as it doesn't reorder the env.
- `scripts/test-fake-discord [FILTER]` runs the service's tests against
  the fake; a filter is a test's name (`forum`, `calls_in_dms`). The whole
  suite is about 30 s, before the last commit of a change to
  `crates/yapfast-discord`, `discord_text` or `testing/fake_discord.ts`
  (`AGENTS.md:1`).
- The fake's `/_control/*` routes both set the world up and let a test act:
  `scripts/ui-session do 'control <name>'` POSTs one, `'get <name>'` GETs
  one (`scripts/ui-session:1`).
- `outputHashes` in `nix/package.nix:30` pins each Git dependency by
  commit; moving one needs a new hash there (`docs/building.md:22`).

## Tests

- `yapfast-discord/tests/` — one file per area, on the fake:
  `fake_server/main.rs:1` builds it, `fake_commands.rs:1`,
  `fake_interactions.rs:1`, `fake_env.rs:1`, `fake_server/messages.rs:1`,
  `fake_server/servers.rs:1`, `fake_server/direct_messages.rs:1`,
  `fake_server/calls.rs:1`, `fake_server/voice_channels.rs:1`,
  `fake_server/profiles.rs:1`, `fake_server/expressions.rs:1`,
  `fake_server/threads.rs:1`, `fake_server/account.rs:1`,
  `fake_server/links.rs:1`, `server_parse.rs:1`.
- `crates/yapfast-core/src/discord_text/tests.rs:1` — Discord's payloads to
  the core's shapes.
- `crates/yapfast-iced/src/tests/signin.rs:1` — the sign-in screen.
