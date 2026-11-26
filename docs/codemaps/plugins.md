# Plugins

Written from commit `4682fe5`.

## There is no plugin system on this branch

Searched, not assumed: `plugin` appears nowhere in `crates/yapfast-iced/src`,
nowhere in `crates/yapfast-iced-kit/src`, and nowhere in `docs/` except one
passing use in `docs/building.md:1`. There is no `plugins` crate in the
workspace (`Cargo.toml:9`), no plugin page in Settings
(`crates/yapfast-iced/src/settings/mod.rs:78` lists every section: Account,
Voice, Messages, Notifications, Discord, Sounds, Advanced, Appearance,
Themes, Games, Keybinds, About), and the test display's page names follow
that list (`goto.rs:140`).

So `goto settings/plugins` fails here on purpose, and says which pages
exist. Do not go looking for a registry, a host or a sandbox: there isn't
one to find:

- `grep -r plugin crates/yapfast-iced crates/yapfast-core/src` — nothing.
- `git branch -r | grep plugins` — the work exists **upstream only**, on
  branches like `origin/feat/plugins`, `origin/feat/plugins-ui`,
  `origin/plugins-audit`, `origin/plugins-dropdown-parts`,
  `origin/plugins-touchup` and `origin/themes-0.4-nopluginsready`. It is
  not on `main` here, nor on `origin/themes-inspector`, which this branch
  starts from.

## The nearest thing that exists: themes

Themes are the extension point this tree has: a package may add pictures,
sounds, layers, art, motion and a `layout.json`, and its named parts may
restyle any element (`docs/themes.md:1`, [themes.md](themes.md)). A theme
cannot run code: everything it says is data, checked before it loads
(`crates/yapfast-core/src/themes/check.rs:1`).

## Where a plugin system would go

If one arrives from upstream, these are the seams it has to touch, in
order:

1. A page in Settings: a `Section` arm
   (`crates/yapfast-iced/src/settings/mod.rs:78`), its entry in `GROUPS`
   (`settings/mod.rs:140`), a module under `settings/pages/`
   (`settings/pages/mod.rs:1`), and the page's name in the test display's
   list (`goto.rs:140`).
2. Its store: `layout.json`, through `crate::layout::Layout`
   (`crates/yapfast-iced/src/layout.rs:1`) or a folder of its own beside
   it.
3. Anything it shows: a named part, so a theme can restyle it
   (`crates/yapfast-core/src/themes/parts.rs:177`).
4. Anything it does: a new `Action` in the core (`application.rs:1644`) —
   not a call from the window into the plugin, so that the same command
   arrives from the tray, a keybind or the test display.

## Gotchas for an agent reading this

- A missing feature is often a **branch**, not an absence: this repository
  carries a great deal of unmerged work on `origin/*`. Before concluding
  something doesn't exist, run
  `git log --all --oneline -20 --grep 'the word'` and
  `git branch -r --contains <sha>`.
- `main` here is older than the work: see "Which tree you are in" in
  [INDEX.md](INDEX.md).

## Tests

None: there is nothing to test. `crates/yapfast-iced/src/tests/settings.rs:1`
covers the pages that do exist.
