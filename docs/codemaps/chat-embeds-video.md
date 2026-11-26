# Chat, embeds and video

Written from commit `4682fe5`.

## What it is

The open room's messages: a virtualized list of rows built from the core's
`ChatMessage`s, one message's content (author, text, attachments, embeds,
reactions, a thread's card), and the pictures and videos inside it.

- `timeline/` holds the list: `Timeline` (`timeline/mod.rs:132`) keeps
  `messages` (the core's `Arc<Vec<Arc<ChatMessage>>>`), each row's height
  and top, what each row shows (`looks`), where it's scrolled (`from_end`),
  the selection and the clocks. `timeline/view.rs:1` draws it,
  `timeline/rows.rs:1` measures rows, `timeline/scroll.rs:1` keeps the
  place, `timeline/keys.rs:1` the keyboard.
- `message/` is one message: `message/mod.rs:1` (gutter, author, text
  blocks, reactions), `message/attachments.rs:1` (files and pictures),
  `message/embed.rs:1` (an embed's card), `message/reaction_chips.rs:1`,
  `message/hover_bar.rs:1` (the buttons on the hovered message),
  `message/beside.rs:1` (the thread beside).
- Pictures: `pictures.rs:1` turns the core's buffers into iced handles
  without a copy; `animated.rs:1` plays their frames; `paint.rs:1` fits
  them. `media.rs:1` builds a message's gallery of pictures.
- Video in a message: `video.rs:1` (`video::frames` follows the frames the
  core decodes), `playback.rs:1` (the cell's play state), and the core's
  `clip.rs:1`, `decode.rs:1`, `video_player.rs:1`.

## Where state lives

| State | Home |
|---|---|
| Messages, read marks, unread | The core's `AppState` (`application.rs:482`) |
| Row heights, scroll, selection, revealed spoilers | `App::timeline` (`timeline/mod.rs:132`) |
| A message's text laid out | Kept with its row (`timeline/rows.rs:1`) |
| Picture buffers and conversions | `pictures.rs:1`, the core's `picture.rs:1` |
| What is playing, and how far | `App::playback` (`playback.rs:1`) |
| Video frames | `App::video`, `Arc<video::Shared>` (`app/mod.rs:153`) |

## How a message comes to the screen

1. The core's events arrive batched (`app/message.rs:181`), and `update`
   applies each (`app/mod.rs:469`).
2. `app.sync()` (`sync.rs:1`) replaces `timeline.messages` only when the
   core's `Arc` changed, and keeps the heights of rows that are the same
   `Arc` as before (`docs/client/structure.md:32`).
3. `timeline/view.rs:1` builds the rows near the viewport (`widgets::Rows`
   holds those, keyed by item), and each row asks `message::view` for its
   element.
4. Pictures the built rows want are asked for in batches
   (`sync.rs:1`), and the core answers as its events.

A message's menu is built from the row and the core's permissions:
`menus::open_menu` (`menus.rs:360`), reached by a right click
(`Message::Timeline(Event::RowMenu)`, `timeline/mod.rs:362`) or by the test
display's `menu N` (`goto.rs:41`).

## Gotchas

- Row indices are not message indices. `crate::timeline::row_of`
  (`timeline/mod.rs:318`) maps an item to its row; the thread beside lives
  at rows at or above `timeline::ASIDE` (`timeline/mod.rs:119`).
- The timeline is virtualized by hand: a widget not in `Rows` is not
  built, so nothing outside the viewport exists to find, click or read.
  That is why the test display's `dump` writes only what's in view.
- A row's height must be measured before it can be scrolled to:
  `Event::Measured` (`timeline/mod.rs:336`) feeds the heights back.
- Text is laid out once, when the message comes, and kept with the row. A
  change that doesn't replace the message's `Arc` won't re-lay it.
- Video frames come from a worker or a decoder on the core's runtime and
  are drawn by the renderer's hook (`video.rs:1`); a frame never travels
  as a message.
- Emoji draw from the Twemoji font packed by the app's `build.rs:1`, so a
  missing glyph is a font problem, not a drawing one.

## Tests

- `crates/yapfast-iced/src/tests/timeline.rs:1` — a channel's messages,
  scrolling, folding, the hover bar, reactions.
- `tests/media.rs:1` — pictures, GIFs, videos, the image viewer.
- `tests/writing.rs:1` — the composer beside them.
- `tests/menus.rs:1` — a message's menu and its submenus.
- Beside the code: `timeline/rows.rs` measures (`docs/client/tests.md:46`),
  `markdown.rs:1` parses Discord's Markdown.
