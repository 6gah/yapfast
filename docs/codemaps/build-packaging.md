# Build and packaging

Written from commit `4682fe5`.

## What it is

One Cargo workspace, one pinned toolchain, a Nix shell to build in, native
C and C++ under it, and four packages out of it.

## Profiles

| Profile | What it is |
|---|---|
| `dev` | `Cargo.toml:166`: line tables only for the app, no debug info for dependencies; four crates at `opt-level = 3` (`Cargo.toml:175`), because DeepFilterNet can't keep up unoptimized |
| `release` | `Cargo.toml:193`: `lto = "off"` (it measured no faster and bigger), stripped |
| `measure` | `Cargo.toml:199`: release plus symbols and incremental, for memory and CPU work |
| `fast` | `Cargo.toml:204`: inherits `measure` — the profile local work and the checks use (`AGENTS.md:1`) |

`docs/measurements/README.md:1` says what to measure and how.

## Workspace

- Members: `crates/yapfast-core`, `crates/yapfast-discord`,
  `crates/yapfast-iced`, `crates/yapfast-iced-kit`, `testing/hosts`
  (`Cargo.toml:9`); `vendor/matrix-rtc-livekit`, `vendor/webrtc-sys`,
  `vendor/davey`, `vendor/deep_filter`, `vendor/x264-src` and
  `vendor/ffmpeg-src` are excluded from it and built by their own crates
  (`Cargo.toml:12`).
- Features: `default = ["desktop", "discord"]`, with `matrix` beside it
  (`Cargo.toml:47`, `:50`, `:51`, `:56`, `:62`). `desktop` is what brings
  the window and the desktop's parts in.
- Patches: `iced_tiny_skia`, `iced_wgpu`, `cosmic-text`, `webrtc-sys`,
  `davey` and `tungstenite` are `vendor/` paths (`Cargo.toml:154`),
  patched in place; `docs/client/toolkit.md:1` lists what each patch is.
- The toolchain is pinned to 1.97.1 with clippy and rustfmt
  (`rust-toolchain.toml:1`), and Nix reads that same file
  (`nix/package.nix:11`).

## Native code

- `crates/yapfast-core/build.rs:1` compiles `crates/yapfast-core/native/`
  (video, camera, decoders, encoders: `native/video.c:1`, `camera.cc:1`,
  `hwenc.c:1`, `av1.c:1`, `h264.c:1`, `hevc.c:1`, `vp8.c:1`, `clip.c:1`,
  `decode.c:1`, `picture.c:1`) with clang, CMake and NASM from the
  development shell (`docs/building.md:57`).
- `crates/yapfast-iced/build.rs:1` packs Twemoji into one font the window
  loads, with `emoji_font_tables.rs:1` (`docs/building.md:40`).
- The root `build.rs:1` gives the Windows executable its icon and hands the
  program its commit.

## Packages

- Nix: `nix/package.nix:1` builds the Discord-only release;
  `nix/flake.nix:1` is the dev shell and the Windows cross shell;
  `nix/native.nix:1` shares libwebrtc and the system libraries;
  `nix/appimage.nix:1` builds the AppDir (`docs/building.md:5`).
- Linux: `packaging/arch/PKGBUILD:1`, `scripts/package-linux.py:1`,
  `scripts/build-appimage:1`.
- Windows: `packaging/windows/yapfast.iss:1`, `scripts/package-windows.py:1`,
  `scripts/build-windows:1` (`docs/windows-beta.md:1`).
- Releases: `scripts/release_package.py:1`, `scripts/sign-release:1`,
  `scripts/release-env:1`, with the keys in `packaging/keys/` and the
  update checks in `src/update/signature.rs:1` (`docs/packaging.md:1`).
- Nightly: `.github/workflows/nightly.yml:1` and `.github/nightly-notes.jq:1`.

## Flows

- **Build after an edit**: `scripts/qbuild` (this branch's wrapper), or
  `nix develop path:./nix --command cargo build --profile fast`. In a new
  worktree run `scripts/warm-worktree` first, which copies the main
  checkout's build (`scripts/warm-worktree:1`).
- **Check before the last commit of a piece of work**: `scripts/check
  --fast`; `--matrix` adds the builds with Matrix (`scripts/check:1`).
  About 40 s after a Rust edit, 4 minutes after a `.slint`-era UI edit
  (`AGENTS.md:1`); its steps run one after another because each UI compile
  takes 6–8 GiB.
- **A new dependency**: add it to the workspace table or its crate, then
  expect `Cargo.lock` to change; a Git dependency also needs an
  `outputHashes` entry (`nix/package.nix:30`).

## Gotchas

- `cargo update` unrestricted breaks the Matrix/LiveKit pins
  (`docs/building.md:50`).
- An 8 GB machine wants `CARGO_BUILD_JOBS=2` (`docs/building.md:65`).
- The vendored iced renderers are patched: an edit under `vendor/`
  recompiles them and the UI crate (`AGENTS.md:1`), so settle such a patch
  in as few builds as possible.
- `vendor/twemoji` is Discord's artwork under CC-BY 4.0; its attribution
  must ship (`docs/building.md:37`).
- Build output lives in `target/<profile>/`; nothing in this repo builds
  on the device a chat runs on unless the machine has the shell.

## Tests and checks

- `scripts/check --fast` (all of it), `scripts/check-doc-links` (links in
  the docs).
- `crates/yapfast-iced-kit/tests/script_fallback.rs:1` and
  `testing/test_font_notices.py:1` cover the font and script fallback.
- `.githooks/pre-commit:1` — what runs before a commit.
