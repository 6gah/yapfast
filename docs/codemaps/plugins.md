# Plugins

Written from commit `a0c1d53` on upstream's `feat/plugins` branch
(`upstream/feat/plugins`), not from this branch: none of the crates below
exist on `private-devtools`. Paths are as that branch has them.

## What it is

A plugin is a WebAssembly module plus a manifest, run in a helper process
and reached from the app through one pipe. Five parts:

- **`crates/yapfast-plugin-api`** — the wire types and the contract: the
  manifest, the hooks, the capabilities, the guest's imports
  (`crates/yapfast-plugin-api/src/lib.rs:1`). Serde and nothing else, as
  both processes build it (`lib.rs:5`). A change here is an ABI change; a
  breaking one raises `API_VERSION` (`lib.rs:31`), the manifest's `api`.
- **`crates/yapfast-plugin-host`** — the helper binary: the shallow fetch
  with gitoxide, the Extism sandbox, the scheduler, the pipe
  (`crates/yapfast-plugin-host/src/lib.rs:1`).
- **`crates/yapfast-plugins`** — everything that knows about plugins and
  nothing about Yapfast: manifests into UI, settings schemas, the store,
  the marketplace, installing, the bridge to the helper
  (`crates/yapfast-plugins/src/lib.rs:1`). It never names the app's state,
  a message or a window; the tap points live in the core (`lib.rs:8`).
- **`crates/yapfast-core/src/plugins/`** — the tap points and what the app
  hands across (`state.rs:1`, `world.rs:1`, `dress.rs:1`, `audio.rs:1`).
- **`pdk/`** — the kits: Rust (`pdk/rust/src/lib.rs:1`), C
  (`pdk/c/yapfast.h:1`), Zig (`pdk/zig/src/yapfast.zig:1`), TinyGo
  (`pdk/tinygo/template/plugin.go:1`), AssemblyScript
  (`pdk/assemblyscript/assembly/index.ts:1`), each with a template and a
  `shout` example.

The app's side is a Settings section, `Section::Plugins`
(`crates/yapfast-iced/src/settings/mod.rs:88`), with its page
(`crates/yapfast-plugins/src/page.rs:1`), behind `#[cfg(feature =
"plugins")]` (`crates/yapfast-iced/src/settings/mod.rs:50`).

## Key files, one line each

| File | What it is |
|---|---|
| `crates/yapfast-plugin-api/src/manifest.rs:1` | `yapfast-plugin.json`: what a plugin is, needs and adds; strict JSON |
| `crates/yapfast-plugin-api/src/hook.rs:1` | The hooks a plugin intercepts, and the deadline each has |
| `crates/yapfast-plugin-api/src/capability.rs:1` | `Capability`, `Permissions`, `HostPattern`: what a plugin may do |
| `crates/yapfast-plugin-api/src/limits.rs:1` | Every size and count a plugin is held to |
| `crates/yapfast-plugin-host/src/fetch/git.rs:1` | One plugin out of one commit: a shallow gitoxide clone, files read from the tree |
| `crates/yapfast-plugin-host/src/fetch/url.rs:1` | `Source`: the URL a plugin is installed from, `#path=` and all |
| `crates/yapfast-plugin-host/src/runtime/` | Extism in `module.rs`, the host functions in `host.rs`, what stays loaded in `kept.rs` |
| `crates/yapfast-plugin-host/src/pipe.rs:1` | The line protocol between the app and the helper |
| `crates/yapfast-plugins/src/store.rs:1` | `<data>/plugins`: the lock file, the content-addressed store, a plugin's data |
| `crates/yapfast-plugins/src/installing.rs:1` | The fetch's report into a review and a package the store writes |
| `crates/yapfast-plugins/src/marketplace.rs:1` | The registry repository, its entries, and what browsing gives the page |
| `crates/yapfast-plugins/src/settings.rs:1` | A plugin's `settings` array into a schema, its values, its form |
| `crates/yapfast-plugins/src/permissions.rs:1` | What the person allowed, read from `grants.json` |
| `crates/yapfast-plugins/src/bridge/mod.rs:1` | The app's end of the pipe: sessions, supervision, the wire |

## Where state lives

Everything a plugin keeps is under `<data>/plugins`
(`crates/yapfast-plugins/src/store.rs:4`):

```text
lock.json            what is installed, and which plugins are on
grants.json          what the person allowed
store/<sha256>/      manifest.json and plugin.wasm, content-addressed: the
                     current version and the one a rollback returns to
data/<id>/           a plugin's storage, and its settings.json
cache/               the helper's compile cache; safe to delete
```

A plugin's own values are `data/<id>/settings.json`, written by the
`Store` in `crates/yapfast-plugins/src/settings.rs:1`. Nothing there runs a
plugin: a module is hashed against the digest the lock file records before
its path is handed to anything (`crates/yapfast-plugins/src/store.rs:15`).

## How a plugin gets installed and updated

1. A source URL is read into a `Source`
   (`crates/yapfast-plugin-host/src/fetch/url.rs:1`): a repository, a ref
   and a folder.
2. The app spawns the helper's `fetch`
   (`crates/yapfast-plugins/src/installing.rs:1`), which clones shallowly
   with gitoxide — the commit's objects and nothing behind it — and reads
   each file straight out of the tree, so no checkout, no `core.*` setting
   and no filter driver runs (`crates/yapfast-plugin-host/src/fetch/git.rs:1`).
   The caps are enforced as it runs, by a watchdog and by each file's
   object header (`fetch/git.rs:9`).
3. The helper prints one JSON `Report`; the app turns it into a `Review`
   and a `Package`, copies the module and icon out of the scratch
   directory, and writes them content-addressed under `store/<sha256>`
   (`crates/yapfast-plugins/src/installing.rs:1`).
4. `lock.json` records the pin — source, commit, digest — and keeps the
   version a rollback returns to (`crates/yapfast-plugins/src/store.rs:9`).
   An update re-runs 2–3 for the same source and ref, which is why the
   store holds two versions; a rollback is the older one again.
5. A marketplace install checks the fetched commit and the module's SHA-256
   against the entry before anything runs, so a repository that changed
   under its pin is refused (`crates/yapfast-plugins/src/marketplace.rs:20`).
   The registry is a git repository and nothing else: no server, no API
   (`marketplace.rs:4`).

## How a plugin API is added

The API is the guest's imports, so one new call touches five places, in
this order:

1. The wire type in `crates/yapfast-plugin-api/src/guest.rs:1` (and its
   module, such as `account.rs`, `frame.rs`, `text.rs`).
2. The host function in `crates/yapfast-plugin-host/src/runtime/host.rs:1`,
   with its cap from `limits.rs:1` and its permission from
   `capability.rs:1`.
3. The app's answer in `crates/yapfast-plugins/src/bridge/`: a
   `session_*.rs` for what the guest asked, a `supervisor_*.rs` for what
   the app asks back (`bridge/mod.rs:1`).
4. The guest wrapper in each PDK that offers it, led by
   `pdk/rust/src/lib.rs:1`.
5. `docs/plugins/abi.md:1`, and `API_VERSION` in
   `crates/yapfast-plugin-api/src/lib.rs:31` if a written plugin breaks.

## How a plugin setting type is added

The type in a manifest (`crates/yapfast-plugin-api/src/manifest.rs:1`);
its rules in the schema, where a type Yapfast doesn't know refuses the
whole manifest (`crates/yapfast-plugins/src/settings.rs:6`); its form row
in `crates/yapfast-plugins/src/settings.rs:1`; the PDK's typed accessor
(`pdk/rust/src/settings.rs:1`); `docs/plugins/settings.md:1`.

## Gotchas

- Two processes, one pipe: the helper is spawned and spoken to in lines
  (`crates/yapfast-plugin-host/src/pipe.rs:1`), and a guest's work is the
  helper's, not the app's.
- A hook fails open: a plugin that traps, runs late or answers nonsense
  leaves the message as it was, and only `"on_failure": "block"` (with
  `messages.intercept`) may stop one
  (`crates/yapfast-plugin-api/src/hook.rs:3`).
- `HookKind::DISPATCHED` leaves out `ActivityPrePublish` on purpose, and a
  test holds that list to `ALL` (`hook.rs:38`).
- The app half is behind `feature = "plugins"`: a build without it has no
  Plugins page and no tap points
  (`crates/yapfast-iced/src/settings/mod.rs:50`).
- A manifest is strict and its stored values are forgiving: a bad schema
  refuses the manifest, a bad value falls back to its default
  (`crates/yapfast-plugins/src/settings.rs:6`).
- `yapfast-plugins` must not name the app: a type from `yapfast-core` or a
  message appearing there is the mistake to look for
  (`crates/yapfast-plugins/src/lib.rs:8`).

## Tests

- `crates/yapfast-plugin-host/tests/` — `fetch.rs`, `sandbox.rs`,
  `scheduler.rs`, `e2e.rs`, `perf.rs`.
- `crates/yapfast-plugins/src/bridge/tests/` — the bridge against a fake
  helper, in parts.
- `testing/plugins/` — the fixtures: `echo.wasm`, `filter.wasm`,
  `trap.wasm`, `spin.wasm`, `hog.wasm`, `stdout.wasm`, `quiet/`,
  `word-filter/`, `showcase/`, `widgets/`, each with its `.wat` source
  (`testing/plugins/README.md:1`).
- `docs/plugins/testing.md:1` says how to run them.
