# Voice, calls and streams

Written from commit `4682fe5`.

## What it is

One persistent voice connection at a time, owned by a native thread, which
either a Discord call or a Matrix call runs
(`crates/yapfast-core/src/voice.rs:1`); its phases are `Phase`
(`voice.rs:19`: disconnected, joining, connected, reconnecting, leaving).
Around it: the window's call UI, audio in and out, noise suppression, the
soundboard, screen sharing (Go Live) in a worker process, and the frames
that come back for the tiles.

- Core: `voice.rs:1` (the connection), `discord_voice.rs:1` and
  `matrix_voice.rs:1` (each network's), `calls.rs:1` (call state),
  `call_details.rs:1` (the signal's numbers), `audio_io.rs:1` with
  `audio_io/pulse.rs:1`, `wasapi.rs:1`, `lasting.rs:1`,
  `silent.rs:1`/`unsupported.rs:1`, `noise.rs:1` (DeepFilterNet),
  `dereverb.rs:1`, `declick.rs:1`, `leveling.rs:1`, `meter.rs:1`,
  `push_to_talk.rs:1`, `soundboard.rs:1`, `sounds.rs:1`, `volume.rs:1`,
  `ring.rs:1`, `voice_message.rs:1`, `voice_roster.rs:1`.
- Go Live: `stream_worker.rs:1` — a separate process (`yapfast
  --stream-worker`) holding the capture, encoders and connection, so they
  cost memory only while sharing; `Job`/`Control`/`Report` lines on its
  pipes (`stream_worker.rs:7`). `stream_audio.rs:1` mixes the stream's
  sound; `encode.rs:1`, `clip.rs:1`, `decode.rs:1` are the codecs;
  `native/video.c:1` and the other `.c`/`.cc` files beside it are the C.
- Discord's side: `yapfast-discord/src/voice_connection.rs:1`,
  `call_connection.rs:1`, `rtp.rs:1`, `dave.rs:1` (the E2EE protocol),
  `call_video.rs:1`, `voice.rs:1`, `stream_connection.rs:1`,
  `service/voice_join.rs:1`, `service/stream_openings.rs:1`.
- The window: `crates/yapfast-iced/src/call/mod.rs:1` — `stage/` (the
  tiles and controls), `dock.rs` (the dock over the account card),
  `popups/` (devices, the share menu, the soundboard, someone's volumes),
  `details.rs` (the connection's details), `dm.rs` (a DM's call),
  `corner.rs` (the corner tile and full screen). `video.rs:1` draws the
  frames; `playback.rs:1` plays messages' voice messages and videos;
  `members.rs:1` shows who's in a channel.

## Where state lives

| State | Home |
|---|---|
| The connection, its phase, devices | The core's `voice::State`, a native thread (`voice.rs:1`) |
| Who is in which call | `ChatRoom::people` (`chat.rs:116`), the core's |
| The call's UI: what's large, what's open | `App::call` (`app/mod.rs:155`) |
| Frames, by participant and source | `App::video`, `Arc<video::Shared>` (`app/mod.rs:153`) |
| What a message is playing | `App::playback` (`app/mod.rs:122`) |
| The worker's process and pipes | `stream_worker.rs:1` |

## Flows

- **Join a voice channel**: a row's `Event::Select` →
  `Action::SelectRoom(id)`; the core's `SelectRoom` joins the call first
  and shows the room once it's in it (`sidebar/mod.rs:353`), which is why
  joining is a room action and not a call action.
- **A frame reaches a tile**: the core decodes into buffers in the
  renderer's format (`picture::set_frame_buffers`, `docs/client/structure.md:119`),
  the view's tile draws it through `video.rs:1`, and a new frame only
  redraws: no message, no view rebuild.
- **Go Live**: the core starts the worker with a `Job`; once Discord has
  started the stream, `Control::Send` hands it the server
  (`stream_worker.rs:7`); frames come back through `frames.rs:1` (core) and
  `video::frames` (window).

## Gotchas

- Only one call at a time: a second join asks first (`call/dm.rs:1`
  draws the question). A test that joins twice sees the question, not the
  call.
- Audio can't run in the test display: the core silences it in tests
  (`app/mod.rs:252`) and the private display runs with `--audio` and ALSA
  plugins chosen by `scripts/run-framework-isolated:1`, but nothing
  reaches real speakers (`scripts/ui-session:1`).
- The stream worker is a separate process: a crash there is a `Report`,
  not a panic in the window, and its memory leaves with it.
- DeepFilterNet's tract model needs `opt-level = 3` even in dev builds
  (`Cargo.toml:175`); unoptimized it can't keep up and its debug
  assertions reject the model's graph.
- Discord's E2EE (`dave.rs:1`) changes what a packet means; a "lost"
  packet that DAVE dropped plays as nothing, not as loss
  (`origin/dave-passthrough`).

## Tests

- `crates/yapfast-iced/src/tests/calls.rs:1` — the call's UI.
- `crates/yapfast-core/src/application/tests/calls.rs:1` — calls in the
  core, as the fake server runs them (`yapfast-discord/tests/fake_server/calls.rs:1`).
- `scripts/test-voice` and `scripts/test-voice`'s harness through
  `testing/test_voice_reliability.py:1`, `testing/join_voice.py:1`,
  `testing/voice_harness.py:1` — real audio paths between two clients.
- `docs/voice.md:1`, `docs/discord/voice.md:1` and
  `docs/discord/go-live.md:1` describe how each part works.
