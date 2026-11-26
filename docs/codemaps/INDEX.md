# Code maps: read this first

Written from commit `4682fe5` on `private-devtools` (which branched from
`origin/themes-inspector`; see "Which tree you are in" below). Each map
names that commit too. A citation's `path:1` is the file's module doc: this
codebase opens every file with one, and it says what the file is for.

## Which tree you are in

The fork's `origin/main` is the **old Slint client** (last commit
2026-11-05) and has no `crates/yapfast-iced`, no `control_file.rs` and no
`--theme-dev`. The iced client, the test display's control file and the
parts inspector live on `origin/themes-inspector` (2026-11-23), which is
what this branch starts from. Check with
`git merge-base --is-ancestor origin/themes-inspector HEAD`.

## The five commands that save the most

- `scripts/ui-session start --discord` — the window on a private display
  against `testing/fake_discord.ts` (`scripts/ui-session:1`).
- `scripts/ui-session do 'goto settings/appearance' 'dump'` — go to a
  screen and read it as text instead of a screenshot (`dump.rs:1`,
  `goto.rs:1`). `dump` is the cheap default; a screenshot
  (`do 'shot name'`) is for pixels, not for reading.
- `scripts/qbuild` — cargo's errors and warnings only, plus the log's path
  (`scripts/qbuild:1`).
- `scripts/test-fake-discord [FILTER]` — the Discord service tests against
  the fake (`testing/fake_discord.ts:1`).
- `scripts/check --fast` — fmt, doc links, clippy, tests, smoke build,
  once before a last commit (`scripts/check:1`).

Build and test inside the dev shell: `nix develop path:./nix --command
cargo ...` with `--profile fast` (`CLAUDE.md:3`, `AGENTS.md:1`).

## Where things are

| | |
|---|---|
| The window: state, messages, layers | `crates/yapfast-iced/src/app/` (`app/mod.rs:1`) |
| The window's parts, one module each | `rail.rs`, `sidebar/`, `header.rs`, `timeline/`, `composer.rs`, `members.rs`, `profile/`, `menus.rs`, `call/` |
| The kit: theme, parts, widgets, fonts | `crates/yapfast-iced-kit/src/` (`lib.rs:1`) |
| Everything Discord talks to | `crates/yapfast-discord/src/` (`lib.rs:1`) |
| The core: state, actions, media, voice | `crates/yapfast-core/src/` (`lib.rs:1`) |
| The private display and the fake server | `scripts/ui-session`, `testing/ui_session.py`, `testing/fake_discord.ts` |
| Docs that are current | `docs/architecture.md`, `docs/client/*.md`, `docs/discord/*.md`, `docs/themes.md`, `docs/building.md` |

## The maps

1. [app-shell-navigation.md](app-shell-navigation.md) — `App`, `Message`,
   `update`/`settle`/`view`, how a place opens.
2. [chat-embeds-video.md](chat-embeds-video.md) — the timeline, one
   message, its embeds, its pictures and videos.
3. [themes.md](themes.md) — the theme, the parts registry, `--theme-dev`
   and its inspector, installed packages.
4. [profiles-activity.md](profiles-activity.md) — profile cards, the full
   profile, the editor, activity cards.
5. [voice-calls-streams.md](voice-calls-streams.md) — the call, its
   stage, Go Live's worker, audio.
6. [settings.md](settings.md) — Settings' sections and pages,
   `layout.json`, preferences.
7. [discord-client-and-fake.md](discord-client-and-fake.md) — the service,
   the gateway and API, and the fake server's env file.
8. [plugins.md](plugins.md) — **there is no plugin system on this
   branch**; where it would go, and how to tell if one arrives.
9. [testing-tools.md](testing-tools.md) — the display, the control file,
   the fake, the measurement harnesses, the UI tests.
10. [build-packaging.md](build-packaging.md) — profiles, native code, Nix,
    the release packages.

## Rules that are not negotiable here

- Nothing goes upstream: no pull request, ever (`CONTRIBUTING.md:1`).
  Work lands on `private-devtools` in the `6gah` fork only.
- Discord is reached only through `testing/fake_discord.ts`, never a real
  account (`client.rs:18` reads `YAPFAST_DISCORD_TEST_SERVER`).
- A change in behaviour updates its doc in the same commit (`AGENTS.md:1`).
  History and measurements go in the commit message.
- Memory is a product goal: share what two places hold, touch only what
  changed per event, stream data rather than buffering it (`AGENTS.md:1`).
