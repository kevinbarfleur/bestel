# Roadmap / TODO

Living list of deferred work. Add items at the bottom; resolve in place
(don't delete) so we keep the audit trail.

## macOS support — deferred

**Status:** out of scope as of v0.1.

**Why deferred:** initial Mac compat pass (commit `897beeb`) shipped a
working build but the dev experience was rough — likely tied to unsigned
DMG Gatekeeper friction, missing notarization, and the fact that the
primary developer plays Path of Exile on Windows. Will revisit if there's
demand from Mac users on the Path of Exile side.

**What's already in place if we revisit:**
- `dirs::home_dir()` / `dirs::document_dir()` paths are portable — no
  Windows-only filesystem code.
- `PobWatcher::start()` returns gracefully when no PoB folder is found
  (`find_pob_dirs()` returns an empty Vec, `boot_watcher` logs a warn
  without aborting). A Mac without PoB installed will boot in
  generalist mode.
- `tauri-plugin-opener` is cross-platform.
- The webview renders via WKWebView natively on macOS (no extra setup).

**What needs to be redone:**
- Bundle config: add `dmg` + `app` to `tauri.conf.json` `bundle.targets`,
  re-add `icons/icon.icns` to the icon array.
- Window config: add `titleBarStyle: "Overlay"` + `hiddenTitle: true`
  for the macOS window so OS traffic-lights overlay our paper topbar.
- TopBar: re-add the `isMac` synchronous platform detection
  (`navigator.platform`) and the `topbar--mac` class with ~78px
  `padding-left` to clear the traffic-lights gutter; conditionally
  hide our right-side custom Min/Max/Close buttons.
- Release workflow: re-add `macos-latest` matrix entries for both
  `aarch64-apple-darwin` (Apple Silicon) and `x86_64-apple-darwin`
  (Intel).
- For a polished release: Apple Developer cert + notarization via
  `tauri-action`'s signing inputs to avoid Gatekeeper warnings.

**Linux note:** Linux is supported in v0.1 (.deb + .AppImage produced
via the release workflow), but the maintainer doesn't run Linux locally,
so issues should be reported via GitHub Issues with reproduction steps.
