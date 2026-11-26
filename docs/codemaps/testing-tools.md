# Testing tools

Written from commit `4682fe5`.

## What it is

Everything that runs the app without a desktop or a real account: the
private display, the control file, the fake Discord, the measurement
harnesses, and the headless UI tests.

| Tool | What it is |
|---|---|
| `scripts/ui-session` | Starts Yapfast on a private Weston display and drives it (`scripts/ui-session:1`) |
| `testing/ui_session.py` | The driver that display runs: starts Yapfast and the fake, reads `STATE/commands`, answers in `STATE/results` (`testing/ui_session.py:1`) |
| `crates/yapfast-iced/src/control_file.rs` | The window's side: lines the app carries out (`control_file.rs:38`, `:46`, `:210`) |
| `crates/yapfast-iced/src/dump.rs` | `dump [PATH]`: the window as text (`dump.rs:42`) |
| `crates/yapfast-iced/src/goto.rs` | `goto TARGET`, `menu N`: places in the window through its own actions (`goto.rs:24`, `:41`) |
| `testing/fake_discord.ts` | A fake Discord: HTTP, WebSocket, `/_control/*` (`testing/fake_discord.ts:1`) |
| `testing/fake_discord_env.ts` | The `--env PATH` world: people, DMs, servers, history, voice (`fake_discord_env.ts:1`) |
| `scripts/test-fake-discord` | The Discord service's tests against the fake |
| `testing/measure_*.py` | Memory and CPU harnesses (`testing/measure_connected.py:1`, `measure_scroll.py:1`, `measure_tray.py:1`, `measure_background.py:1`, `measure_cameras.py:1`, `measure_discord_cameras.py:1`, `measure_networks.py:1`) |
| `testing/record_host.rs`, `tray_host.rs` | Stands in for the display's recorder and the tray (`testing/record_host.rs:1`, `testing/tray_host.rs:1`) |
| `crates/yapfast-iced/src/tests/` | The window's headless tests (`docs/client/tests.md:9`) |

## How a command gets to the window

1. `scripts/ui-session do 'click 10 20' 'dump'` hands each command to
   `testing/ui_session.py`, which writes one `ID<TAB>COMMAND` line to
   `.local/ui-session/commands` (`scripts/ui-session:1`).
2. `testing/ui_session.py` acts: input is synthesised on the display
   (`scripts/framework-input`), a screenshot is Weston's, and a window
   command (`theme`, `stage`, `pick`, `window`, `goto`, `dump`, `menu`) is
   appended to the control file `YAPFAST_UI_TEST_CONTROL` names.
3. The window reads that file every 500 ms
   (`control_file::commands`, `control_file.rs:46`) and carries each new
   line out (`control_file.rs:210`).
4. `dump`, `goto` and `menu` answer in `answer.txt` beside it, `ok` or
   `error WHY` (`control_file.rs:200`), which the driver waits for and
   reports as the command's result.

## The two tools that save the most

- **`dump`** instead of a screenshot: one line per element in view, its
  part or kind, its text, where it is and the theme's colours, indented by
  what holds it, a long list showing its first few and then a count
  (`dump.rs:49`). It reads the window through iced's headless simulator
  over `App::view` and takes the parts from the frame it draws
  (`crates/yapfast-iced-kit/src/inspect.rs:1`).
- **`goto`** instead of a click: `home`, `server NAME|#N`,
  `channel NAME`, `settings/PAGE`, `profile USER`, through the core's own
  actions (`goto.rs:24`). `menu N` opens the Nth message's context menu
  (`goto.rs:41`).

## Gotchas

- Nothing here reaches the desktop or a real account: the display is
  private and Discord is the fake (`scripts/ui-session:1`).
- The window reads the control file every 500 ms, so a command lands
  within half a second; the driver's `answer.txt` wait is 20 s
  (`testing/ui_session.py:1`).
- The app's `stdout` goes to `.local/ui-session/yapfast.log`, never to the
  agent: a result must come back as a file (that's why `dump` writes one).
- `.local/ui-session/` is emptied at each `start`, except the copied
  binaries, the build log and the videos (`scripts/ui-session:1`).
- A killed session: `scripts/ui-session stop`; a wedged one:
  `scripts/ui-session status`, then `stop`.
- The UI tests and `ui-session` are different harnesses: a test builds the
  window in-process with `iced_test` (`crates/yapfast-iced/src/tests/mod.rs:24`),
  while `ui-session` runs the real binary on a display.
- Memory is what most harnesses measure; `--profile measure` is the
  profile they use, never `fast` (`docs/measurements/README.md:1`).

## Tests

- `scripts/check --fast` runs fmt, doc links, clippy, the tests and a
  smoke build (`scripts/check:1`).
- `crates/yapfast-iced/src/tests/mod.rs:24` — `preview`, `start`,
  `interact`, `shows`, `present`, `last`, `right_click`.
- `crates/yapfast-iced-kit/src/testing.rs:1` — the kit's own tests, with no
  core.
- `scripts/test-stack` — Matrix and LiveKit's stack, when a test needs
  them (`scripts/test-stack:1`).
