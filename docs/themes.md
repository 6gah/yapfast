# Themes

The iced rewrite's owner-parity status is tracked field by field in
[Theme port parity ledger](theme-port-parity.md). A core token is not marked
complete there until an iced surface and focused test consume it.

Settings → Appearance offers the themes Yapfast comes with and the person's
own as quick choices. Settings → Themes is the dedicated theme-management
destination. It currently manages installed packages and custom themes, their
options, content permission, performance and reduced motion. Its Marketplace
and Add from URL tabs fetch public GitHub themes through the core's bounded,
verified package pipeline, show a stock-themed permission/cost review, and do
not persist anything until Install. An
installed theme shown in Appearance links
back to that page. Built-in and custom
themes are a full set of the colours of `Theme` in
`yapfast_core::theme` (`crates/yapfast-core/src/theme.rs`), plus whether it's
dark, which sets the scrollbars' shade and the colours worked out from the
others. The core holds the built-in ones as `BUILT_IN` and reads custom
themes and hex colours; `crates/yapfast-iced/src/themes.rs` makes the chosen
one the kit's `Theme` (`crates/yapfast-iced-kit/src/theme.rs`), which every
widget paints with. An installed package is read from the core theme store,
resolved with `Package::resolve_with`, and converted at that same boundary;
see [the client's theme](client/window.md#theme). The
choice is kept in `layout.json` as `theme`, the theme's id, and applied
before the window's first frame, and again to a window made anew. With none
kept, the window shows Bonfire. An id Yapfast doesn't know, or a deleted
theme's, shows Yap. The window-control command `theme ID` chooses and keeps
one as Appearance and Themes do.

## Installed themes

Settings → Themes reads installed packages from `<data>/themes` through
`yapfast_core::themes::install::Themes`. Their layout ids are
`installed:<package-id>`, keeping them separate from built-in and `custom:`
ids. Each installed row offers **Try**, **Apply**, **Roll back** when core has
retained an earlier revision, and **Remove**. Try shows the package for 30
seconds without changing `layout.json`; its banner offers **Keep** and
**Revert**, and expiry reverts automatically. A reviewed remote package can be
tried from checked files in a private temporary folder and is installed only
if Keep is chosen. The applied row offers **Turn off theme**, which keeps
`theme: null` and returns to Bonfire immediately and on
the next start. Each actual change also keeps the replaced id as
`previous_theme`; Settings → Themes offers **Back to _name_** while that
theme still exists. Re-choosing the current theme leaves the existing Back
target intact, and restoring a theme after Turn off consumes that target, as
in the owner UI. `Ctrl+Shift+T` is the emergency turn-off shortcut and is
handled above settings and other overlays, so a problematic package cannot
trap the person behind themed UI. A package missing its preferred dark
variant uses its light variant. A dual-variant package's Auto choice follows
iced's live system light/dark mode without changing its persisted package id.

Installed rows identify the package's authors, version, available dark/light
modes and description, and name any package fields this Yapfast left out.
An entry whose stored manifest is missing, changed or otherwise refused by
the core remains visible under **Themes that couldn't be opened**, together
with the verification error; it is never silently treated as uninstalled.

The page also keeps `theme_performance` (`full`, `balanced`, or `light`) and
`reduce_motion` in `layout.json`. Balanced is the default. These settings are
shared by installed themes; Light still permits colours, fonts, shapes and
spacing, while package animation is disabled. Reduce motion independently
turns off package flourish; both choices keep Wired's short stock transitions
so menus and dialogs do not lose their ordinary opening and closing feedback.

Resolved package metrics are registered beside the kit `Theme` under the same
theme id, so the public `Theme` struct remains the single conversion boundary
and keeps its original shape. Surface radii, outlines and shadows now reach the
shared buttons, fields, composer, toggles, chips, menus, dialogs, tooltips,
cards, code blocks, embeds, message/list rows, rail icons, call tiles, pickers,
popouts, toasts and scrollbars. Header and list-label grounds, text and case,
profile-section frames and title bars, card top insets, and message-row inset
and stripe settings use the same path. Values a package does not provide keep
Wired's exact stock style.

Spacing controls timeline message and group gaps, avatar size, chat padding,
channel and member row heights, and the sidebar and member-pane widths. A pane
width the person resized explicitly in `layout.json` wins over the theme's
suggested width. Popup duration, curve, opening and closing style apply to the
app's animated layers; hover and press choices apply to shared action and icon
buttons and rail icons. Performance and Reduce motion are applied when these
motion values are resolved, without a second settings store.

Installed layouts can dock the composer above or below the history, move or
hide the member pane, dock the server rail/navigation on the left or right,
hide the server rail, centre the message lane at
`chat_max_width`, and hide message avatars. Flat, bubbles, IRC and the
non-rotated part of cards are drawn by the existing virtual timeline. Bubble
grounds (including linear gradients), own-message ink, radius, padding and
maximum share are honoured; IRC time/name columns and separator are honoured;
cards use their ground, padding and shadow. Missing layout fields retain
Wired's stock renderer. A top rail scrolls the same themed server tiles across
the window; tree mode places the rows through the selected server above its
channels and the remaining rows below, and honours its one/many folder
expansion rule. Clicking the selected server or direct-message root folds and
opens that channel branch without persisting transient window state. Grid
members use the package's column count, tile size and member row styling. Card
tilt/tape and ransom names remain deferred with the rotation-dependent work. A
collapsed member pane uses the owner's 56 px avatar strip and expands to the
full list while pointed at.

Layout borders can replace the ordinary solid edge with solid, dashed or
dotted strokes at ten declared places: composer, card, menu, message, embed,
code, tile, divider, actions and pane edges. Dialog and tooltip outline
consumers remain to be ported. Their
dash/gap, optional width and colour come from the resolved owner theme;
otherwise the surface's shape border and Wired colour remain the fallback.

The non-rotating list subset also covers rail width, tile size/radius/gap/pad,
inside or below labels, lift hover, pill mode, divider size,
and mention-badge colours. Channel and member rows use package height and gap,
text size/case/ink, solid separators, zebra stripes and foot reserve; channel
rows additionally use icon size/gap/ink, selected underlines, voice meta and
unread glyph/badge modes. Member rows use avatar size/gap, presence text or
glyphs, activity/presence meta, online badges, row margins/insets, header
templates and metrics, and the Activity switch. Rail, channel and member-row
frames and selected overlays use the package's density-aware nine-slice list
art. Header, separator, footer, selected-prefix, unread/status glyph, rail
ground/below/between/foot and member-avatar frame slots use that same renderer;
avatar frame windows, presence rings/dots and the channel-header rule are also
honoured. List-art sprite sheets follow Performance and Reduce motion just as
surface art does. Reflected rail tiles and tilt/magnify list motion are not
applied.

Package icon replacements are decoded once and flow through the kit's existing
glyph calls. Package app/ring sounds replace the corresponding core samples;
menu, popup, toggle, send, reaction and theme-applied cues use the interface
sound slots. Settings → Themes separately lets each applied package replace
notification sounds, enable its interface sounds and set their interface
volume. Owner defaults (notifications on, interface sounds off, 50% interface
volume) need no `theme_sounds` entry; only differences are kept. The package's
built-in `theme-sounds` option still participates in its own resolved sound
levels. `theme_transitions` keeps the owner's separate Transitions between
channels permission. When enabled in Full performance without Reduce motion,
the resolved owner fade or dissolve transition is drawn over the retained
previous chat surface. Iced 0.14 has no public live-window snapshot API, so
the transition covers the conversation rather than Slint's whole compositor
image. Theme art layers are decoded once, with their blur, blend and dim baked
over the surface ground, and can cover, contain, stretch or tile the window, rail,
sidebar, account, chat, header, members, aside, profile pane, call, Settings,
menu, actions, embed, code, label, picker, toast, selected rows, cards,
dialogs, popouts and the lightbox. Card-over layers stay above an unthemed
profile card's content, and Footer art keeps its declared height (64 px when
unspecified) at the member list's foot. Image overlays are drawn above their
declared window or chat scope. Tooltip art/glass remains to be ported; tooltip
colour, border, radius and shadow tokens already apply.
Procedural scanline, LCD-grid, grain, vignette, rolling-band and flare overlays
use the same scope and performance rules. LCD `ghost` keeps the grid but not
the previous conversation image: capturing that image requires an iced
renderer/window hook and is deferred with the fork-dependent work. Declared
drift runs at 60 Hz in Full and 30 Hz in Balanced (heavy layers rest in
Balanced), and rests in Light or Reduce motion. Sprite sheets follow that same
clock. Ambient and
theme-applied, sent-message and reaction particles draw at their declared
depth; Light, Reduce motion and heavy-effect rules keep them still or omit
them. Glass surfaces use one reduced, preblurred copy of the resolved window
art and the package's ground, rim, highlight, gap and radius settings. This is
the owner's fake-glass treatment, not live backdrop blur. It is rebaked after
the window resizes so panes stay aligned to the current window pixels.
Text-role shadow, glow, outline and emboss are extra glyph passes behind
message bodies, author/time text, headings, header text and member labels, so
links and cross-message selection remain Wired's. Channel rows and cards draw
declared click ripples, hover gloss and selected highlighters. Foil bands
cross profile cards and member-grid tiles; they follow the pointer or sweep
on the art clock, and rest halfway across in Light or Reduce motion. Slint's
tiny highlighter/foil rotations stay omitted with the other rotation styles.
Sidebar, member, chat and composer art reserve insets keep ordinary content
clear of decorations at pane edges.

Chrome label substitutions, prefix/suffix marks and time marks are applied to
ordinary navigation, timeline and composer labels. An installed top bar can
show workspace buttons and its centre/right text. Theme content is opt-in per
package in Themes and currently controls avatar shape/ring/decorations and
theme or palette name colours. Media recolouring and dithering apply to still
and animated avatars, icons, emoji, stickers, attachments and previews. Light
performance uses the dither palette as a gradient instead of running its pixel
pattern.

## Custom themes

Create theme opens the owner's guided creator on a copy of the theme shown.
Its Start, Colours, Shape, Fonts and Advanced steps derive the package through
`yapfast_core::themes::creator`: accent, light/dark or a chosen picture,
picture blur and dim, message layout, corners, density, outlines, glass and
built-in or imported body/heading fonts. Advanced lists every derived colour
by its token name, swatch and hex. Undo restores the preceding recipe and Start
over restores the recipe that opened. Colours offers up to five of the newest
pictures from Pictures and its Wallpapers folders beside the file chooser.
Every draft is resolved through the same
temporary package registry as installed themes, so colours, shapes, fonts,
picture art and glass preview across the iced window immediately without
becoming a saved theme.

Save writes the creator recipe and package folder, keeps the custom theme and
chooses it. Export writes a shareable package folder including its manifest,
licence, picture and fonts. The person remains responsible for permission to
share pictures and imported fonts. Cancel, leaving Themes or closing settings
shows the theme from before and removes the draft. The separate Colour editor
keeps the earlier palette-only workflow: its hex fields accept `#rgb`,
`#rrggbb` or `#rrggbbaa` and mark anything else. A saved theme's card has a
pencil that opens it again to rename, change or delete it; deleting the chosen
one shows Yap. Closing the window to the
[tray](tray.md) closes Settings, so it drops the unsaved edit too.

`layout.json` keeps them in `custom_themes`, in the order Appearance shows
them after the built-in ones and Themes shows them under Your themes:

```json
{
  "id": "custom:1",
  "name": "Plum",
  "base": "yap",
  "dark": true,
  "colours": { "rail": "#111215", "chat": "#2a1a3a", "row-hover": "#1a1b20" },
  "creator": { "accent": 16760647, "messages": "cards", "corners": 80 },
  "folder": "themes/creator/1"
}
```

`base` is the built-in theme the first copy came from (a copy of a custom
theme takes its base). Yapfast writes every colour, but a colour that's missing,
such as one added in a later version, or that isn't a colour, takes the
base's value, and a name Yapfast doesn't know is ignored, so themes survive new
colours. Guided themes additionally keep their `creator` recipe and a relative
`folder` containing their package assets. Ids are `custom:` and a number, and no built-in id has a colon, so
they can't collide; on reading, Yapfast gives a fresh one to any custom theme
whose id isn't of that form or repeats another's. The editor's temporary fields
exist only while it's open.

At startup, a missing folder under Yapfast's exact managed
`themes/creator/<id>` path can be regenerated from its saved creator recipe
when it uses only reproducible built-in assets. Arbitrary custom folder paths,
and recipes whose imported font or picture bytes are gone, are never rewritten.

| Theme | Id | Dark | Accent | From |
| --- | --- | --- | --- | --- |
| Yap | `yap` | yes | marigold | Yapfast's own |
| Bonfire | `bonfire` | yes | blurple | Discord's Dark theme, the default |
| Catppuccin Mocha | `catppuccin-mocha` | yes | mauve | [Catppuccin](https://github.com/catppuccin/catppuccin) |
| Dracula | `dracula` | yes | purple | [Dracula](https://draculatheme.com/contribute) |
| Nord | `nord` | yes | frost nord8 | [Nord](https://www.nordtheme.com/docs/colors-and-palettes) |
| Gruvbox Dark | `gruvbox-dark` | yes | orange | [Gruvbox](https://github.com/morhetz/gruvbox) |
| Tokyo Night | `tokyo-night` | yes | blue | [Tokyo Night](https://github.com/tokyo-night/tokyo-night-vscode-theme), Night style |
| Catppuccin Latte | `catppuccin-latte` | no | mauve | [Catppuccin](https://github.com/catppuccin/catppuccin) |
| Rosé Pine Dawn | `rose-pine-dawn` | no | rose | [Rosé Pine](https://rosepinetheme.com/palette/) |
| Sakura | `sakura` | no | blossom pink | Yapfast's own |

The palettes of Catppuccin, Dracula, Nord, Gruvbox, Tokyo Night and Rosé Pine
are published under the MIT licence; thanks to their authors. For Tokyo
Night, the steps between its grounds and text (bg_highlight, fg_gutter, dark3,
dark5) are named as folke's tokyonight.nvim names them. Yapfast takes their
colours, not their names for UI parts, so each mapping is Yapfast's:

- The ground steps go rail, sidebar, chat from the deepest to the one text
  sits on, with raised surfaces (popovers, reactions, stage tiles) and the
  composer one step off the chat. Where a palette has no shade below its
  background (Nord's nord0, Tokyo Night's bg_dark), the rail, panel and edge
  darken it, and hover and some row grounds are halfway mixes of two palette
  colours.
- Text, secondary, muted and faint take the palette's text ramp; faint keeps
  at least 2.5:1 against the sidebar, near Yap's 2.8:1.
- Content on the accent and on red (on-accent, on-danger) takes the palette's
  darkest or lightest colour, whichever reads; on pastel accents, as in
  Catppuccin and Tokyo Night, that's dark.
- Light themes wash hovers with a dark overlay (black, or the palette's text
  colour) instead of white, sweep skeletons with a white band, and draw the
  tooltip light, since its text is the theme's text colour.

Bonfire maps Discord's Dark theme from the CSS variables of its web client
(`.theme-dark` with `.theme-darker`, which is what its settings call Dark;
`.theme-dark` alone is Ash, `.theme-midnight` Onyx), read from the public
bundle: the rail and sidebar are `--background-base-lowest` (#121214), the chat
`--background-base-lower` (#1a1a1e), the account panel and composer
`--background-base-low` (#202024), raised surfaces and the tooltip
`--background-surface-high` (#242429), text `--text-default`, muted
`--text-muted`, links `--text-link`, the accent
`--control-primary-background-default` (blurple, #5865f2, hover #4452bb, white
on it), red `--control-critical-primary-background-default` (#d22d39), live
`--status-positive`, and hovers, selections, lines and secondary buttons the
`--interactive-background-*`, `--border-subtle` and
`--control-secondary-background-*` washes laid over their grounds. It carries
no Discord name or logo.

Discord role colours stay as servers set them, and some read faintly on a
light theme; Yapfast doesn't adjust them yet.
