# Profiles and activity cards

Written from commit `4682fe5`.

## What it is

Someone's profile in its four shapes — the card (opened by a member's row,
a message's author, a friend), the same card docked beside a one-to-one DM,
the full profile over the window, and the editor — plus the cards for a
role, a server, a sticker, an emoji and a server tag, and the Active Now
cards.

- `profile/mod.rs:1` — the `Profiles` state and the card's three places;
  the card is 300 px beside a list (`profile/mod.rs:44`).
- `profile/look.rs:1` — works out what a profile shows, once each time the
  core's view of it changes.
- `profile/body.rs:1` — draws the card; `profile/full.rs:1` the full
  profile's Board, Activity and Wishlist tabs; `profile/activity.rs:1` an
  activity's card; `profile/cosmetic.rs:1` effects, decorations and
  nameplates; `profile/name_style.rs:1` the Display Name Style window.
- `profile/editor/mod.rs:1` — the profile editor, drawn with the card's
  parts (`editor/card.rs:1`, `editor/colour.rs:1`, `editor/menus.rs:1`,
  `editor/styles.rs:1`).
- `person_menu.rs:1` — someone's context menu and the person dialog;
  `info_cards.rs:1` roles, servers, stickers; `tag_card.rs:1` a server
  tag's card; `active_now.rs:1` the Active Now cards.
- The core's side: `application/profiles.rs:1` (the profile view and
  Discord's fetch), `application/profile_editor.rs:1` and
  `application/cosmetics.rs:1`; `yapfast-discord/src/profile.rs:1`,
  `activity.rs:1`, `collectibles.rs:1`, `wishlist.rs:1`,
  `profile_effect.rs:1`.

## Where state lives

| State | Home |
|---|---|
| What the core knows of each profile | `AppState` (`application.rs:482`), by user id |
| Which cards are open, and where | `App::profiles` (`app/mod.rs:126`) |
| The editor's draft | The editor's own state, kept while it's open |
| Cosmetics equipped | The core's; `application/cosmetics.rs:1` |
| The Activity panel's filter | `application/activity_panel.rs:1` |

## Flows

- **Open someone's card**: `App::open_card(user, at)`
  (`app/screens.rs:186`) → `Profiles::open_card` → the core fetches what it
  lacks (`Message::Events` brings it) → `profile/look.rs` works the card
  out → `profile/body.rs` draws it. The test display's `goto profile USER`
  does exactly this (`goto.rs:120`).
- **Add something to the card**: a field on the core's profile view, a line
  in `profile/look.rs`, a block in `profile/body.rs`; the card's parts are
  named and themed like any other (`docs/themes.md`).
- **Edit the profile**: Save dispatches the core's profile-edit action;
  the editor keeps its draft, and `App::editor_*` (`app/screens.rs:104`)
  routes its messages while it's open.

## Gotchas

- The card is opened where it was clicked: `open_card` takes the point,
  and the press it came from is `App::pressed` (`app/mod.rs:128`). A
  caller without a press (a tool) passes the window's middle.
- Profiles are `Arc`-shared by the core; `look` reworks one only when the
  core's version changed (`docs/client/structure.md:32`).
- Three places draw the same card (`body.rs`); a change to one shape must
  not be duplicated into the others — it takes a parameter.
- The full profile's tabs ask Discord for more on first open
  (`ProfileTab`, `application.rs:416`); without a fetch their lists are
  empty, which looks like a bug and isn't.
- The editor's preview drives the window's theme while it's open
  (`docs/client/settings.md:1`); closing it must put the theme back.

## Tests

- `crates/yapfast-iced/src/tests/people.rs:1` — cards, the full profile,
  friends, Active Now.
- `crates/yapfast-iced/src/tests/editor.rs:1` — the editor.
- `crates/yapfast-iced/src/tests/cards.rs:1` — role, server, sticker and
  tag cards.
- `crates/yapfast-core/src/application/tests/profiles.rs:1` and
  `profile_editor.rs:1` — the core's side of both.
