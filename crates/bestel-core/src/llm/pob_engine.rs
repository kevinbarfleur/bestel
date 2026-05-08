//! Lazy global accessor for the bundled headless PoB engine.
//!
//! Path resolution precedence:
//! 1. `BESTEL_POB_ENGINE_DIR` env var — explicit override pointing at a dir
//!    containing `luajit/luajit.exe`, `harness/api-stdio.lua`, and PoB
//!    submodule mirrors (`PathOfBuildingCommunity/src` + optionally
//!    `PathOfBuilding-PoE2/src`).
//! 2. Walking up from the current working directory looking for the
//!    repo-relative layout (`external/luajit/windows-x86_64/luajit.exe`
//!    + `crates/bestel-pob-engine/vendor/{api-stdio-bestel, PathOfBuilding*}`).
//!    This is what `cargo test` / `cargo tauri dev` see.
//! 3. Returns `None` — `pob_calc` then surfaces a clear "engine not
//!    configured" error.
//!
//! The resolved [`PobEngineHandle`] is cached for process lifetime.

use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use bestel_pob_engine::{EngineConfig, PobEngineHandle};

use crate::sources::FileCache;

static ENGINE: OnceLock<Option<Arc<PobEngineHandle>>> = OnceLock::new();

/// Get (or lazily build) the global PoB engine handle. Returns `None`
/// when the vendored LuaJIT binary or the PoB submodule is missing.
pub fn global() -> Option<Arc<PobEngineHandle>> {
    ENGINE.get_or_init(build_global).clone()
}

fn build_global() -> Option<Arc<PobEngineHandle>> {
    let layout = resolve_layout()?;
    let log_dir = FileCache::default_dir().join("pob-engine-logs");
    let config = EngineConfig {
        luajit_path: layout.luajit,
        harness_path: layout.harness,
        pob_root_poe1: layout.pob_poe1,
        pob_root_poe2: layout.pob_poe2,
        log_dir,
        idle_timeout: Duration::from_secs(600),
        command_timeout: Duration::from_secs(60),
        max_restarts_per_window: 3,
        restart_window: Duration::from_secs(300),
    };
    Some(PobEngineHandle::new(config))
}

struct Layout {
    luajit: PathBuf,
    harness: PathBuf,
    pob_poe1: PathBuf,
    pob_poe2: PathBuf,
}

fn resolve_layout() -> Option<Layout> {
    if let Ok(dir) = std::env::var("BESTEL_POB_ENGINE_DIR") {
        let dir = PathBuf::from(dir);
        return resolve_in_dir(&dir);
    }

    // Walk up from cwd looking for the workspace marker.
    let mut cur = std::env::current_dir().ok()?;
    for _ in 0..6 {
        if let Some(layout) = resolve_in_workspace(&cur) {
            return Some(layout);
        }
        if !cur.pop() {
            break;
        }
    }
    None
}

fn resolve_in_workspace(root: &Path) -> Option<Layout> {
    let luajit = root
        .join("external")
        .join("luajit")
        .join("windows-x86_64")
        .join(luajit_exe_name());
    let harness = root
        .join("crates")
        .join("bestel-pob-engine")
        .join("vendor")
        .join("api-stdio-bestel")
        .join("api-stdio.lua");
    let pob1 = root
        .join("crates")
        .join("bestel-pob-engine")
        .join("vendor")
        .join("PathOfBuildingCommunity")
        .join("src");
    let pob2 = root
        .join("crates")
        .join("bestel-pob-engine")
        .join("vendor")
        .join("PathOfBuilding-PoE2")
        .join("src");
    if !luajit.exists() || !harness.exists() || !pob1.exists() {
        return None;
    }
    Some(Layout {
        luajit,
        harness,
        pob_poe1: pob1,
        pob_poe2: pob2,
    })
}

fn resolve_in_dir(dir: &Path) -> Option<Layout> {
    let luajit = dir.join("luajit").join(luajit_exe_name());
    let harness = dir.join("harness").join("api-stdio.lua");
    let pob1 = dir.join("PathOfBuildingCommunity").join("src");
    let pob2 = dir.join("PathOfBuilding-PoE2").join("src");
    if !luajit.exists() || !harness.exists() || !pob1.exists() {
        return None;
    }
    Some(Layout {
        luajit,
        harness,
        pob_poe1: pob1,
        pob_poe2: pob2,
    })
}

fn luajit_exe_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "luajit.exe"
    } else {
        "luajit"
    }
}
