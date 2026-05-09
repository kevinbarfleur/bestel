//! Bestel evaluation tooling. Currently exports the LLM-as-judge pipeline
//! (`judge`) that reads PersistedRun JSON files plus the evaluation set
//! (`tests/eval/eval_set.toml`) and grades each answer with Claude
//! Sonnet 4.5 via the Anthropic API. Wired into the `bestel eval-judge`
//! subcommand; not part of the runtime hot path.

pub mod judge;
