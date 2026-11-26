# App shell and navigation

Written from commit `4682fe5`.

## What it is

One `App` holding the window's whole state, one `Message` saying what
happened, one `update` that hands a message to its area or screen, and one
`view` that builds the window again from the top (`app/mod.rs:1`,
`app/message.rs:1`, `docs/client/structure.md:10`).

- `App` (`app/mod.rs:58`): `client` (the core), `shell` (the window and
  the tray), `win` (what lives only as long as the window), a struct per
  area (`rail`, `sidebar`, `timeline`, `composer`, `members`, `profile`
  state in `profiles`, `call`), a `Slot` per screen the core opens
  (`server_screen`, `channel_screen`, `win.settings`), `layout` (what
  `layout.json` keeps), `theme`/`themes`, and `requests` (what screens ask
  of the root).
- `Message` (`app/message.rs:28`) has one variant per area, plus
  `Control(String)` for the test display's control file.
- `App::update` (`app/mod.rs:399`): `handle_message` (`app/mod.rs:465`) is
  one `match` arm per message, then `settle` (`app/settle.rs:150`) does
  what the arm asked (closes, preferences, focus) and syncs if anything
  changed.

## Where state lives

| State | Home |
|---|---|
| The core's rooms, people, selection | `yapfast_core::application::AppState` (`application.rs:482`) |
| What place is open | `AppState::navigation` (`application.rs:137`, `application.rs:700`) |
| What screen is open | `Slot`s on `App` (`slot.rs:1`) |
| Window-only state (rail, sidebar, timeline…) | `App`'s fields (`app/mod.rs:58`) |
| Preferences, window size, theme chosen | `layout.json` through `crate::layout::Layout` (`layout.rs:1`) |
| The window itself, tray, zoom | `crate::shell::Shell` (`shell.rs:1`) |

## How a place opens

A click is never the model: the core's `Action` is (`application.rs:1644`).

1. A control sends `Message::Side(Event::Select(id))` or similar
   (`sidebar/mod.rs:310`).
2. That dispatches `Action::SelectRoom(id)`, or `Action::Open(id)` first
   when the room is in another place (`sidebar/mod.rs:350`).
3. `app.sync()` (`sync.rs:1`) compares the core's lists with what the
   window shows and rebuilds only what differs: the rail
   (`rail::rail`, `sync.rs:27`), the sidebar's rows (`sidebar::sidebar`,
   `sync.rs:28`), the timeline.
4. The next `view` draws from that state (`app/view.rs:28`), stacking the
   window's layers and then whatever is open over it (`app/layers.rs:1`).

Going somewhere by name, for a tool, is `crates/yapfast-iced/src/goto.rs:24`:
`home`, `server NAME|#N`, `channel NAME`, `settings/PAGE`, `profile USER`,
all through those same actions.

## How the window is drawn

- Layers, top last: the window, then a layer each for what's open
  (`app/view.rs:1`). Popups open with a motion (`app/layers.rs:1`).
- The window is built again after every message, so cost goes out of the
  view: manual list virtualization (`widgets::Rows`), pictures as the
  core's own buffers (`pictures.rs:1`), moving pictures inside their
  widget (`animated.rs:1`), meters and clocks read as they're drawn
  (`docs/client/structure.md:26`).
- `ctrl`-keybinds are matched in one place before any layer takes keys
  (`app/layers.rs:466`); Ctrl+Shift+T turns a dev theme off and
  Ctrl+Shift+I turns the parts inspector on (`app/layers.rs:476`).

## Subtasks, in order

- New message: a variant in `app/message.rs:28`, an arm in
  `app/mod.rs:465`, the work in its area's module.
- New screen the core opens: a `Slot` on `App`, a `screens.rs` glue
  function (`app/screens.rs:1`), a `Layer` if it draws over the window.
- New key: `app/subscription.rs:1` turns the iced event into a `Message`;
  `app/layers.rs:1` decides who has it.

## Gotchas

- `App` is borrowed apart by `ctx::cx!` (`ctx.rs:1`); a screen's update
  takes a `Cx`, not `&mut App`, so it can't reach the fields the root
  keeps. Fighting the borrow means the wrong layer.
- `settle` runs after every message, and its passes repeat until nothing
  is asked (`app/settle.rs:150`): a request that asks for another closes
  in a chain, a few passes deep.
- The view is rebuilt from scratch; work in it is per frame, per list and
  per message. Compute in `sync` or in the handler, not in the view.
- `Message::Control` is the only way the test display drives the window
  (`app/subscription.rs:226`); it runs on the window's thread, like any
  other message.

## Tests

- `crates/yapfast-iced/src/tests/navigation.rs:1` — the rail and the
  sidebar.
- `tests/shell.rs:1` — the frame, tray, notifications, zoom.
- `tests/tree.rs:1` — a theme's server tree.
- `tests/mod.rs:24` — `preview`, `start`, `interact`, `shows`, `present`
  (`docs/client/tests.md:9`).
