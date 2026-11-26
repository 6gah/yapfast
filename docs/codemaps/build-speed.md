# Faster development builds

Written from commit `4682fe5`, on a branch that has no numbers in it yet.
Read "What was not measured" before trusting any of this.

## What is opt-in

`scripts/build-speed` sets three knobs as environment variables for the one
process it starts. Nothing else in the tree changes: `cargo build`,
`scripts/qbuild`, `scripts/check` and `scripts/ui-session` build exactly as
before (`scripts/build-speed:1`).

| Knob | What it sets | What it should buy |
|---|---|---|
| sccache | `CARGO_BUILD_RUSTC_WRAPPER=sccache` | A rebuild after a branch switch, a `cargo clean` or a second checkout reuses compiled objects. Nothing for an edit. |
| mold | `-C link-arg=-fuse-ld=mold` in the target's `RUSTFLAGS` | A faster final link, which is most of a small edit's wall time in a workspace this large. |
| Cranelift | `CARGO_PROFILE_FAST_CODEGEN_BACKEND=cranelift` | Much faster codegen for the `fast` profile; the binary runs slower, which `fast` already tolerates. Needs a nightly toolchain with `rustc_codegen_cranelift`. |

Using them:

```
nix develop path:./nix --command scripts/build-speed build
nix develop path:./nix --command scripts/build-speed build -- test -p yapfast-iced
eval "$(nix develop path:./nix --command scripts/build-speed env)"
nix develop path:./nix --command scripts/build-speed measure
```

`build` prints which knobs it could turn on and which tool was missing
(`scripts/build-speed:1`); `env` prints them as `export` lines for a shell
that wants them for a while; `measure` runs the matrix below and writes
`.local/build-speed/results.json`.

## What was not measured, and why

**No build ran on the machine this branch was written on**: it has no
shell, no cargo, no Nix (see [INDEX.md](INDEX.md)). So there are no
timings and no memory numbers here, and inventing them would be worse than
leaving them out. The table below is the shape the numbers take, filled in
by `scripts/build-speed measure`.

What is claimed from reading the code, not from a run:

- The workspace's UI crate is the expensive one: 60+ modules, a view built
  from scratch per message (`docs/client/structure.md:26`).
- Almost every edit that touches a `.rs` file under `crates/yapfast-iced`
  or `crates/yapfast-iced-kit` ends in a link of the `yapfast` binary, so
  the linker is on the critical path of nearly every rebuild — that is the
  case for mold.
- Dependencies (iced, Skia-free tiny-skia, the Matrix SDK) are what a clean
  build spends most of its time on, and what sccache would not have to do
  twice — that is the case for sccache.
- Native C and C++ (`crates/yapfast-core/native/video.c:1` and the rest of
  that folder, `vendor/x264-src`, `vendor/ffmpeg-src` with NASM-assembled
  objects) is built by `cc`/CMake/NASM from
  `crates/yapfast-core/build.rs:1`, not by rustc: **Cranelift cannot change
  it either way**, and mold does link those objects into the final binary.

## The numbers

Definitions `scripts/build-speed measure` uses, so runs are comparable:

- **clean**: `cargo clean -p yapfast -p yapfast-iced -p yapfast-iced-kit
  -p yapfast-core`, then `cargo build --locked --profile fast`. The
  dependencies stay built: this measures the crates that change, not the
  third-party graph, which is what a day of work pays for.
- **cached-fresh**: `cargo build --locked --profile fast` with nothing
  changed since the last one.
- **one-line-edit**: one line of `crates/yapfast-iced/src/dump.rs` changed
  (a space added, then put back), then the same build.
- **peak MiB**: the largest sum of `VmRSS` over the build's process tree,
  sampled five times a second while it runs (`scripts/build-speed:1`).

| Configuration | clean | cached-fresh | one-line edit | peak MiB (clean) |
|---|---|---|---|---|
| plain | _not measured_ | _not measured_ | _not measured_ | _not measured_ |
| sccache | _not measured_ | _not measured_ | _not measured_ | _not measured_ |
| mold | _not measured_ | _not measured_ | _not measured_ | _not measured_ |
| sccache + mold | _not measured_ | _not measured_ | _not measured_ | _not measured_ |
| cranelift (`--cranelift`) | _not measured_ | _not measured_ | _not measured_ | _not measured_ |

The measurement that matters least is `cached-fresh`: with everything up to
date cargo still takes a second or two to check the graph, and no knob
changes that.

## Where each one may break, and what to do

- **mold**: it links the final binaries only. If it fails, it fails at the
  last step of `cargo build` — `note: run with RUSTFLAGS=...`, a
  `mold` error on an undefined symbol, or a link that the toolchain's
  `cc` wrapper refuses. Then check it by hand on the smallest binary first:
  `RUSTFLAGS="-C link-arg=-fuse-ld=mold" cargo build --profile fast
  -p yapfast-iced-kit --tests`. The C objects
  (`crates/yapfast-core/native/*.o`) are ordinary ELF objects and mold
  handles them; the one thing to watch is a `--gc-sections` or
  `-Wl,--as-needed` the platform's own `cc` used to pass.
- **Cranelift**: it compiles Rust only. Untested here, and the branch
  therefore leaves it **off**. What would break first, in order of
  likelihood: `vendor/iced_tiny_skia` and `vendor/iced_wgpu` (patched,
  heavy on generics), `vendor/cosmic-text` (patched), and any crate using
  explicit SIMD intrinsics. Turn it on for one run with
  `scripts/build-speed build --cranelift`, which refuses when the
  component is missing and says why (`scripts/build-speed:1`). It is never
  a setting for `scripts/check`: a type error it doesn't catch, or a
  miscompile, must not reach a commit.
- **sccache**: worst case it silently doesn't cache (e.g. a `RUSTFLAGS`
  change every build defeats it) and costs a wrapper process per rustc
  call. `sccache --show-stats` tells whether it is hitting.

## Keeping it out of everyone else's build

- No file under `.cargo/` is changed (`.cargo/config.toml:1` keeps what it
  had), so a plain `cargo build` is untouched.
- The profile definitions are unchanged too (`Cargo.toml:204`): Cranelift
  reaches the profile as an environment variable for one invocation, not
  as a checked-in `codegen-backend`.
- `scripts/qbuild --json` reports the time it measured, so comparing a
  build with and without a knob needs no extra tooling
  (`scripts/qbuild:1`).
