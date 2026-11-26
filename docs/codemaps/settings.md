# Settings

Written from commit `4682fe5`.

## What it is

A card over the dimmed window: the sections down its left under the account
and a search, and the chosen section's page on the right under its title
and the close button. It's a screen on the contract, so its state lives in
a `Slot` only while it's open (`crates/yapfast-iced/src/settings/mod.rs:1`,
`docs/client/settings.md:1`).

- `settings/mod.rs:78` — `Section`: Account, Voice, Messages,
  Notifications, Discord, Sounds, Advanced, Appearance, Themes, Games,
  Keybinds, About. Their order in the column is `GROUPS`
  (`settings/mod.rs:140`); each section's title and icon are
  `Section::title` / `Section::icon` (`settings/mod.rs:103`).
- Pages: `settings/pages/mod.rs:1` holds the switch-and-choice pages
  (Account, Messages, Notifications, Sounds, Advanced, About, Discord,
  Games) with what each preference does; `settings/appearance.rs:1` is
  Appearance and the theme editor's host; `settings/voice.rs:1` Voice &
  Video; `settings/keybinds.rs:1`; `settings/theme_creator.rs:1`,
  `settings/theme_page.rs:1`, `settings/theme_marketplace.rs:1` the theme
  screens; `settings/updates.rs:1` the update check.
- The preferences themselves are `layout.json`, read and written through
  `crates/yapfast-iced/src/layout.rs:1` and the core's
  `yapfast_core::themes::layout` (`themes/layout.rs:1`).

## Where state lives

| State | Home |
|---|---|
| Whether Settings is open, and on what | `App::win.settings`, a `Slot` (`slot.rs:1`); the section also in `App::win.settings_section` |
| The pages' preferences | `layout.json` (`layout.rs:1`), applied as they change |
| The theme editor's draft and preview | `settings/appearance.rs:1`; the preview drives `App::theme` while open |
| What Discord must be told | The core, through the action each change dispatches |

## Flows

- **Open a section**: `App::open_settings(section)`
  (`app/settle.rs:87`) opens the `Slot` and shows the section. The test
  display's `goto settings/appearance` is this call (`goto.rs:140`).
- **A switch is flipped**: the page sends `Event::Pref` →
  `settings/pages/mod.rs:1` runs the effect (writes `layout.json`, applies
  it, dispatches an action if the core must know) and asks
  `Request::Changed` so the window redraws what shows it.
- **A new page**: a `Section` arm (title, icon, `GROUPS`), a page module
  with its view and events, a row in `settings/mod.rs`'s switch. The test
  display's page names are `goto.rs:140`.

## Gotchas

- Settings is a `Slot`, not a field: reaching it from outside is
  `win.settings.get()`/`get_mut()`, and it's gone when closed. The pages'
  state goes with it — that's deliberate (`settings/mod.rs:1`).
- Escape, the close button and a click outside all close it; a page that
  opens another screen (the editor, a menu) must close that first
  (`app/sky.rs` no: see `app/screens.rs:114` for the editor's Escape
  order).
- `Ctrl+K` does nothing over Settings (`app/screens.rs:132`).
- The Voice page can start a camera to try it
  (`App::camera_test`, `app/settle.rs:138`); that's a real capture source
  and only runs while the page is open.
- Preferences are kept as they change, not on close (`settings/mod.rs:1`):
  a test that closes Settings and reads `layout.json` sees the change.
- The page names the test display takes are this branch's — there is no
  `plugins` page here (see [plugins.md](plugins.md)).

## Tests

- `crates/yapfast-iced/src/tests/settings.rs:1` — the card, its pages,
  Appearance, the theme editor, the inspector.
- `crates/yapfast-iced/src/tests/dialogs.rs:1` — the dialogs the pages
  open.
- `crates/yapfast-core/src/application/tests/preferences.rs` — not
  present on this branch; the core's own layout tests are in
  `themes/layout.rs:1`.
