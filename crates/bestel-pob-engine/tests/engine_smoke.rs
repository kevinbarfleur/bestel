//! End-to-end smoke test: spawn a real LuaJIT subprocess, load a fixture
//! build, and verify `pob_calc(category=offence)` returns sane output.
//!
//! Skipped automatically if the vendored LuaJIT binary or the PoB submodule
//! is missing — e.g. on a fresh clone before `vendor-luajit.ps1` ran or
//! `git submodule update --init` finished.

use std::path::PathBuf;
use std::time::Duration;

use bestel_pob_engine::lifecycle::Game;
use bestel_pob_engine::{
    CalcRequest, Category, EngineCalcs, EngineConfig, PobEngineHandle,
};

fn workspace_root() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
        .unwrap_or(manifest)
}

fn engine_paths() -> Option<EngineConfig> {
    let root = workspace_root();
    let luajit = root
        .join("external")
        .join("luajit")
        .join("windows-x86_64")
        .join("luajit.exe");
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
    let log_dir = root.join("target").join("pob-engine-logs");
    if !luajit.exists() || !harness.exists() || !pob1.exists() {
        return None;
    }
    Some(EngineConfig {
        luajit_path: luajit,
        harness_path: harness,
        pob_root_poe1: pob1,
        pob_root_poe2: pob2,
        log_dir,
        idle_timeout: Duration::from_secs(60),
        command_timeout: Duration::from_secs(60),
        max_restarts_per_window: 3,
        restart_window: Duration::from_secs(300),
    })
}

fn fixture_xml(name: &str) -> Option<String> {
    let path = workspace_root()
        .join("tests")
        .join("fixtures")
        .join("pob")
        .join(name);
    std::fs::read_to_string(path).ok()
}

#[tokio::test]
async fn cold_calc_offence_against_fixture_build() {
    let config = match engine_paths() {
        Some(c) => c,
        None => {
            eprintln!(
                "skipping pob engine smoke test — vendored LuaJIT or PoB submodule missing"
            );
            return;
        }
    };
    let xml = match fixture_xml("poe1_inquisitor.xml") {
        Some(x) => x,
        None => {
            eprintln!("skipping — fixture poe1_inquisitor.xml not found");
            return;
        }
    };

    let engine = PobEngineHandle::new(config);
    let resp = engine
        .calc(CalcRequest {
            game: Game::Poe1,
            build_xml: xml,
            category: Category::Offence,
            skill_index: None,
            calcs: EngineCalcs {
                enemy_is_boss: Some(true),
                ..Default::default()
            },
        })
        .await
        .expect("first calc should succeed");

    // The harness echoes a `stats` table. We don't assert on specific
    // numbers — that would couple to PoB version drift — but the table must
    // be a JSON object and the Calcs echo must reflect our request.
    assert!(resp.stats.is_object(), "stats should be an object");
    assert!(resp.calcs.is_object(), "calcs should be an object");

    engine.shutdown().await;
}
