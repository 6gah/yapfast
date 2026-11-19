# Theme port parity ledger

Last audited against the owner Slint implementation on 2026-11-18. A row is
**complete** only when the owner behavior reaches an iced consumer, its UI is
in the corresponding settings workflow, and a focused test covers it. Parsing
a token in `yapfast-core` does not make a row complete.

## Complete and tested

- Owner package discovery and `Package::resolve_with`; persisted selection,
  options, content permission, Performance and Reduce motion in `layout.json`.
- Base colours through the kit `Theme` conversion boundary, with Wired colours
  as fallbacks. Package surface gradients and extra paints are retained in the
  companion registry; main panes, Settings, menus, pickers, embeds, code,
  labels, actions and selected rows consume them.
- Shapes, radii, solid/dashed/dotted borders, shadows, all eight ordinary
  spacing tokens and the covered pane/list geometry.
- Popup/hover/press motion under Performance and Reduce motion.
- The separate Transitions between channels choice drives the resolved owner
  fade or dissolve transition over the retained previous chat surface.
- Ordinary layout modes, the covered non-rotating list styling, top/tree rails,
  grid and collapsed members, and content opt-in.
- Moving surface/list art, particles, fake glass, procedural overlays, icon and
  sound replacement, media recolouring and dithering on the surfaces named
  below. This does not claim the still-missing art subsections or surfaces.
- A dedicated Settings → Themes destination; Appearance keeps quick choices
  and routes Browse/New/Edit into theme management.
- Settings → Themes exposes the owner Installed, Marketplace, Add from URL
  and Create tabs. Catalog search covers names, authors and descriptions;
  URL/catalog fetches enter the same protected review with permissions,
  notes and cost before installation.
- Installed packages support Apply/Turn off, 30-second Try/Keep/Revert,
  rollback and uninstall. Trials do not persist or install reviewed bytes
  until Keep, and invalid/deleted Back targets safely fall to stock.
- Package notification sounds, interface sounds and interface volume are
  separate per-theme choices and survive install/rollback refreshes.
- Auto dual-variant packages follow live iced system light/dark changes while
  explicit mode choices and the persisted package id remain unchanged.
- The owner emergency turn-off shortcut (`Ctrl+Shift+T`) works even while a
  settings layer or another overlay has keyboard focus.
- Existing owner custom themes preserve `creator` and `folder` data across an
  iced read/write round trip.
- The owner's non-rotating bubble gel and IRC rule draw around and beside
  virtual timeline content, with focused pixel tests.
- The owner guided creator has Start, Colours, Shape, Fonts and Advanced,
  newest-Pictures suggestions, picture blur/dim, built-in and imported fonts,
  exact asset-aware undo/start-over, package save/export and guarded deletion.
  Its checked draft resolves through a temporary companion, so colours,
  shapes, fonts, picture art and glass preview live and unregister on Cancel.

## Active parity work

- Typography: checked package fonts now load into iced and the Theme boundary
  carries all seven roles, size, weight, case and retained letter spacing.
  Message bodies, names, conversation headers, channel rows and Settings
  titles/labels consume it. Remaining ordinary text surfaces still need the
  same trace and focused coverage.
- Appearance → Readable text is persisted and restores stock reading faces;
  remaining reading surfaces are being audited.
- Installed previews, remix/cost capability detail, list/gallery views and
  empty-state actions are still being expanded; ordinary metadata, refused
  errors and lifecycle actions are present.
- Surface art/glass expansion: Window, Rail, Sidebar, Chat, Members, Account,
  Composer, ProfilePane, Header, Call, Aside, Settings, Menu, Actions, Embed,
  Code, Label, Picker, Toast, RowSelected, Card, Dialog, CardOver, Footer,
  Popout and Lightbox are wired. Tooltip art/glass remains; its ordinary
  shape and colour tokens already apply.
- Extra colours are retained; focus-ring, selection, toggle/knob-off,
  scrollbar/track and the wired surface grounds now consume them. The
  remaining control-specific extras are still being traced.
- Text shadow, glow, outline and emboss draw behind selectable message
  bodies, author/time pieces, headings, header text and member labels. Click
  ripple, hover gloss and selected highlighter reach channel rows and cards;
  foil reaches profile cards and member-grid tiles.

## Missing owner UI/behavior

- Whole ordinary renderer subsections still missing: reaction stamp, the
  separate row highlighter and name-hover decrypt event slots, non-rotating
  animation properties and `motion.unfocused`.
- Remaining ordinary list/chrome/content fields recorded in the audit include
  row/header fonts and focus detail, profile following, initials,
  embed/activity following and hover/press sound cues.

## Deliberately deferred by owner instruction

- Theme cursors.
- Rotation, card tilt/tape, magnification, reflected rails and ransom names.
- LCD previous-frame ghosting or any work requiring an iced fork.
