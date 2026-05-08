//! Test-running infrastructure shared by:
//! - `bestel-test` (the legacy TOML harness, runs scenarios as subprocesses)
//! - `bestel run-battery` (headless CLI orchestrator that runs against live
//!   providers in-process and persists `PersistedRun` records)
//! - the dev-panel window (loads + displays both scenarios and real prompts)

pub mod real_prompts;
pub mod scenario;

pub use real_prompts::{category_counts, load as load_real_prompts, RealPrompt};
pub use scenario::{
    discover_scenarios, load_all as load_scenarios, load_scenario, Cost, Expectation,
    ProviderChoice, Scenario,
};
