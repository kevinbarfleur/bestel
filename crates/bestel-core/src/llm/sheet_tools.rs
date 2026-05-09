//! Tool definitions and handlers for the Build Sheets feature.
//!
//! Four tools are exposed to the model:
//!
//!   - `sheet_propose_section` — agent-drafted (or re-drafted) prose for one
//!     of the 6 fixed sections. Emits `LlmDelta::SheetDraftUpdate` for the
//!     UI; tool result tells the agent to await the user's Confirm/Edit.
//!
//!   - `sheet_ask` — questions_v2-style picker for a personality question
//!     (purpose of an aura, role of a unique). Emits `LlmDelta::SheetAskUser`;
//!     tool result tells the agent to await the user's answer.
//!
//!   - `sheet_finalize_request` — persists the full sheet to the
//!     `build_sheets` SQLite table. Returns the assigned `sheet_id`.
//!
//!   - `get_active_build_sheet` — returns the validated sheet for the given
//!     fingerprint if any (and whether it is stale vs. the current PoB hash).
//!
//! Draft state during the interview lives in the agent's conversation
//! context — not in the DB. The DB only sees finalized sheets. This keeps
//! the state machine simple and avoids a per-chat draft table whose
//! lifecycle would need its own management.

use anyhow::{anyhow, Context, Result};
use serde_json::{json, Value};
use tokio::sync::mpsc;

use crate::persistence::global_db;
use crate::sheets::{
    self,
    types::{BuildSheet, BuildSheetSection, DefiningItemEntry, IntentEntry, KnownGap, SchemaVersion},
};

use super::LlmDelta;

pub const SHEET_PROPOSE_SECTION: &str = "sheet_propose_section";
pub const SHEET_ASK: &str = "sheet_ask";
pub const SHEET_FINALIZE_REQUEST: &str = "sheet_finalize_request";
pub const GET_ACTIVE_BUILD_SHEET: &str = "get_active_build_sheet";
pub const SHEET_OPEN_INTERVIEW: &str = "sheet_open_interview";

/// JSON schemas exposed to the model. Append these to the workspace tool
/// list in `tools::tool_schemas()`.
pub fn schemas() -> Vec<Value> {
    vec![
        json!({
            "name": SHEET_PROPOSE_SECTION,
            "description": "Surface a draft for ONE section of the active Build Sheet. Use this whenever you've read the PoB enough to propose prose for `identity`, `archetype`, `damage`, `defense`, `items`, or `intent`. The user sees a card with your prose and a Confirm/Edit choice. After calling this, end your turn — the user's reply (chat message or button click) is the next input. Do NOT call multiple sections in one turn; the user works through them one at a time. The 6 section ids are fixed in v1: 'identity', 'archetype', 'damage', 'defense', 'items', 'intent'.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "section_id": {
                        "type": "string",
                        "enum": ["identity", "archetype", "damage", "defense", "items", "intent"],
                        "description": "Stable id for the section. Drives the UI stepper."
                    },
                    "title": {
                        "type": "string",
                        "description": "Human-readable section title (matches the design's section labels: 'Identity', 'Archetype & skill', 'Damage scaling', 'Defense layers', 'Defining items', 'Intent & goals')."
                    },
                    "body": {
                        "type": "string",
                        "description": "Drafted prose. Concise — 1-3 sentences for prose sections, a chip-friendly enumeration for `identity` and `items`. The user will confirm or correct."
                    }
                },
                "required": ["section_id", "title", "body"],
                "additionalProperties": false
            }
        }),
        json!({
            "name": SHEET_ASK,
            "description": "Ask the user a leverage-based purpose question about a high-personality element of their build (an aura with multiple jobs, a unique that could play multiple roles, a defensive layer whose intent isn't readable from the PoB alone). The UI renders a questions_v2-style chip picker. Use this AFTER you've drafted the corresponding section but BEFORE finalizing — the user's answer flips the section's `purpose` field. Pick options that are mutually informative: each option should describe a *different* primary intent the user could plausibly have. There is no fixed cap on these questions; ask one when the leverage of getting it right is high (item that defines the build's identity), skip when low (a flex slot).",
            "input_schema": {
                "type": "object",
                "properties": {
                    "question_id": {
                        "type": "string",
                        "description": "Stable id for the question (e.g. 'cospri-purpose', 'brass-dome-purpose'). Drives delta routing to the right card."
                    },
                    "title": {
                        "type": "string",
                        "description": "The question itself, framed as a single sentence ending in a question mark."
                    },
                    "subtitle": {
                        "type": "string",
                        "description": "Optional one-line context explaining why the answer matters (max ~120 chars)."
                    },
                    "options": {
                        "type": "array",
                        "items": {"type": "string"},
                        "minItems": 2,
                        "maxItems": 6,
                        "description": "2-6 mutually-informative answer chips. Keep each option <=80 chars."
                    },
                    "multi": {
                        "type": "boolean",
                        "default": false,
                        "description": "Allow multi-select. Default false — most purpose questions are single-intent."
                    },
                    "has_other": {
                        "type": "boolean",
                        "default": true,
                        "description": "Whether the user can provide a free-text 'Other…' answer."
                    }
                },
                "required": ["question_id", "title", "options"],
                "additionalProperties": false
            }
        }),
        json!({
            "name": SHEET_FINALIZE_REQUEST,
            "description": "Persist the completed Build Sheet to local storage. Call this ONCE, after every section has been confirmed by the user and any leverage-based purpose questions have been answered. Inputs include the full sheet payload — sections, defining items with roles, intent constraints, known gaps. After this call succeeds, the sheet is `validated` and every future chat about this build can read it via `get_active_build_sheet`.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Short human-readable label, e.g. 'Spell Crit · Cold Conversion'."
                    },
                    "fingerprint": {
                        "type": "string",
                        "description": "Build fingerprint string. Format: <ascendancy_lower>:<main_skill_lower>:<sorted_unique_names_lowered_joined_by_+>. CRITICAL: include ALL unique-rarity item names from `get_active_build.items[]`, sorted alphabetically and lowercased — NOT just the role-tagged ones from your Identity card. The runtime computes the fingerprint deterministically from the parsed PoB; if you exclude any unique items, the next chat won't be able to look this sheet up."
                    },
                    "pob_hash": {
                        "type": "string",
                        "description": "Hash of the canonical PoB JSON at authoring time. Used to detect drift on later attaches."
                    },
                    "sections": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "id": {"type": "string"},
                                "label": {"type": "string"},
                                "body": {"type": "string"},
                                "confirmed": {"type": "boolean"}
                            },
                            "required": ["id", "label", "body", "confirmed"],
                            "additionalProperties": false
                        }
                    },
                    "defining_items": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "name": {"type": "string"},
                                "role": {
                                    "type": "string",
                                    "enum": ["engine", "defining", "amplifier", "enabler"]
                                },
                                "purpose": {"type": "string"}
                            },
                            "required": ["name", "role"],
                            "additionalProperties": false
                        }
                    },
                    "intent": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "constraint": {"type": "string"}
                            },
                            "required": ["constraint"],
                            "additionalProperties": false
                        }
                    },
                    "known_gaps": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "label": {"type": "string"},
                                "note": {"type": "string"}
                            },
                            "required": ["label"],
                            "additionalProperties": false
                        }
                    },
                    "authored_in_chat": {
                        "type": "string",
                        "description": "Optional id of the chat session where the sheet was authored."
                    }
                },
                "required": ["name", "fingerprint", "pob_hash", "sections"],
                "additionalProperties": false
            }
        }),
        json!({
            "name": SHEET_OPEN_INTERVIEW,
            "description": "Emit a complete one-shot Build Sheet interview after deep PoB analysis. Use this AFTER you have called get_active_build, pob_calc, and at least 2-3 of {wiki_parse / kb_search / read_internal_reference} on the build's defining items + main skill — never before. Pre-draft EVERY section body from your analysis (6 sections fixed: identity, archetype, damage, defense, items, intent) and pre-populate 3-7 leverage-purpose questions across sections (some sections may have zero questions). Section bodies render as markdown — use **bold** for headline numbers/items, `-` bullets for enumerations, *emphasis* sparingly. KEEP THEM SHORT: identity ≤120 chars (one line), archetype ≤200 chars (one sentence), damage / defense / items ≤400 chars each (bullet lists), intent ≤250 chars (short bullets). The user reads the whole panel in one glance; verbosity ruins it. After this tool returns, end your turn — the user fills the panel and replies with one structured `[INTERVIEW SUBMISSION]` user message. On that next turn, parse the submission, call `sheet_finalize_request` with the merged payload (do NOT re-run analysis or re-emit the interview), then answer the user's original question citing the persisted sheet.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "sections": {
                        "type": "array",
                        "minItems": 6,
                        "maxItems": 6,
                        "description": "All 6 fixed sections, in any order, each pre-drafted from your PoB analysis.",
                        "items": {
                            "type": "object",
                            "properties": {
                                "id": {
                                    "type": "string",
                                    "enum": ["identity", "archetype", "damage", "defense", "items", "intent"]
                                },
                                "title": {
                                    "type": "string",
                                    "description": "Human-readable label, e.g. 'Identity', 'Archetype & skill', 'Damage scaling', 'Defense layers', 'Defining items', 'Intent & goals'."
                                },
                                "draft_body": {
                                    "type": "string",
                                    "description": "Concise prose drafted from your analysis. 1-3 sentences for prose sections; chip-friendly enumeration for `identity` and `items`."
                                }
                            },
                            "required": ["id", "title", "draft_body"],
                            "additionalProperties": false
                        }
                    },
                    "questions": {
                        "type": "array",
                        "minItems": 0,
                        "maxItems": 12,
                        "description": "Leverage-based purpose questions across sections. Use one or two for sections with real personality leverage (an aura with multiple jobs, a unique that could play different roles, a defensive layer whose intent is not readable from the PoB alone). Skip sections that lack ambiguity; do not pad. Each question MUST set its `section_id` so the UI can render it under the right heading.",
                        "items": {
                            "type": "object",
                            "properties": {
                                "question_id": {
                                    "type": "string",
                                    "description": "Stable id, e.g. 'petrified-blood-purpose', 'doryanis-lesson-role'."
                                },
                                "section_id": {
                                    "type": "string",
                                    "enum": ["identity", "archetype", "damage", "defense", "items", "intent"]
                                },
                                "title": {
                                    "type": "string",
                                    "description": "Short, exile-facing question (e.g. 'Why are you running Petrified Blood?')."
                                },
                                "subtitle": {
                                    "type": "string",
                                    "description": "Optional one-line clarification."
                                },
                                "options": {
                                    "type": "array",
                                    "minItems": 2,
                                    "maxItems": 6,
                                    "items": {"type": "string"},
                                    "description": "Mutually-informative options. Each one describes a *different* primary intent the user could plausibly have."
                                },
                                "multi": {
                                    "type": "boolean",
                                    "default": false,
                                    "description": "Whether the user can pick more than one option."
                                },
                                "has_other": {
                                    "type": "boolean",
                                    "default": true,
                                    "description": "Whether the user can provide a free-text answer via an inline-expanding textarea."
                                }
                            },
                            "required": ["question_id", "section_id", "title", "options"],
                            "additionalProperties": false
                        }
                    },
                    "notes_prompt": {
                        "type": "string",
                        "description": "One-sentence hint for the freeform Notes textarea at the bottom of the panel (e.g. 'Anything else about your goals, gear constraints, or play style?')."
                    }
                },
                "required": ["sections", "questions", "notes_prompt"],
                "additionalProperties": false
            }
        }),
        json!({
            "name": GET_ACTIVE_BUILD_SHEET,
            "description": "Look up an existing validated Build Sheet by fingerprint. Call this FIRST whenever a PoB is loaded and you're about to comment on the build — if a sheet exists, prefer reading from it over re-deriving everything from the PoB. Optionally pass `current_pob_hash` so the response also tells you whether the sheet is stale (same fingerprint, drifted hash). Returns `{found: false}` when nothing matches; in that case fall through to your normal flow (and consider proposing the interview).",
            "input_schema": {
                "type": "object",
                "properties": {
                    "fingerprint": {
                        "type": "string",
                        "description": "Build fingerprint to look up. Format: <ascendancy_lower>:<main_skill_lower>:<sorted_unique_names_lowered_joined_by_+>. Use ALL unique-rarity item names from `get_active_build.items[]`, sorted alphabetically and lowercased — same shape that `sheet_finalize_request` uses to persist."
                    },
                    "current_pob_hash": {
                        "type": "string",
                        "description": "Optional hash of the currently-attached PoB. When present and different from the stored hash, the response sets `stale: true`."
                    }
                },
                "required": ["fingerprint"],
                "additionalProperties": false
            }
        }),
    ]
}

/// Send a `LlmDelta` if the channel is available. Logged on send failure
/// (closed channel) but never errors out — sheet UI events are best-effort.
fn try_emit(deltas: &Option<mpsc::UnboundedSender<LlmDelta>>, delta: LlmDelta) {
    if let Some(tx) = deltas {
        // Send is best-effort; a closed channel just means the chat ended
        // before the tool returned (e.g. user navigated away).
        let _ = tx.send(delta);
    }
}

/// Handle `sheet_propose_section`. Emits the delta and returns a tool result
/// instructing the agent to end its turn and await the user's confirm/edit.
pub async fn dispatch_sheet_propose_section(
    input: &Value,
    deltas: &Option<mpsc::UnboundedSender<LlmDelta>>,
) -> Result<String> {
    let section_id = input
        .get("section_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("'section_id' is required and must be a string"))?;
    let title = input
        .get("title")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("'title' is required and must be a string"))?;
    let body = input
        .get("body")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("'body' is required and must be a string"))?;

    try_emit(
        deltas,
        LlmDelta::SheetDraftUpdate {
            section_id: section_id.to_string(),
            title: title.to_string(),
            body: body.to_string(),
            confirmed: false,
        },
    );

    Ok(json!({
        "acknowledged": true,
        "section_id": section_id,
        "user_must_confirm": true,
        "next_step": "End this turn. The user will reply with Confirm (re-call sheet_propose_section with confirmed=true via a follow-up turn) or with corrected text."
    })
    .to_string())
}

/// Handle `sheet_ask`. Emits the delta and returns a tool result instructing
/// the agent to end its turn and await the user's answer.
pub async fn dispatch_sheet_ask(
    input: &Value,
    deltas: &Option<mpsc::UnboundedSender<LlmDelta>>,
) -> Result<String> {
    let question_id = input
        .get("question_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("'question_id' is required"))?;
    let title = input
        .get("title")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("'title' is required"))?;
    let subtitle = input.get("subtitle").and_then(|v| v.as_str()).map(String::from);
    let options_raw = input
        .get("options")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow!("'options' must be an array of strings"))?;
    let options: Vec<String> = options_raw
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();
    if options.len() < 2 {
        return Err(anyhow!("'options' must contain at least 2 strings"));
    }
    let multi = input.get("multi").and_then(|v| v.as_bool()).unwrap_or(false);
    let has_other = input
        .get("has_other")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    try_emit(
        deltas,
        LlmDelta::SheetAskUser {
            question_id: question_id.to_string(),
            title: title.to_string(),
            subtitle: subtitle.clone(),
            options: options.clone(),
            multi,
            has_other,
        },
    );

    Ok(json!({
        "acknowledged": true,
        "question_id": question_id,
        "user_must_answer": true,
        "next_step": "End this turn. The user's reply (chat text or chip selection) is the answer."
    })
    .to_string())
}

/// Handle `sheet_open_interview`. Validates the payload shape, emits a
/// single `SheetInterviewOpen` delta with the verbatim payload, and
/// returns a tool result instructing the agent to end its turn and await
/// the user's `[INTERVIEW SUBMISSION]` reply on the next turn.
///
/// Validation is shape-only here — the JSON Schema in `schemas()` already
/// enforces structure to the model, but we re-check the critical bits
/// because (a) Anthropic does not strictly enforce schemas and (b) the
/// frontend assumes these arrays are always present.
pub async fn dispatch_sheet_open_interview(
    input: &Value,
    deltas: &Option<mpsc::UnboundedSender<LlmDelta>>,
) -> Result<String> {
    let sections = input
        .get("sections")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow!("'sections' must be an array of 6 section drafts"))?;
    if sections.len() != 6 {
        return Err(anyhow!(
            "'sections' must contain exactly 6 entries (got {})",
            sections.len()
        ));
    }
    let questions = input
        .get("questions")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow!("'questions' must be an array (may be empty)"))?;
    if questions.len() > 12 {
        return Err(anyhow!(
            "'questions' must have at most 12 entries (got {})",
            questions.len()
        ));
    }
    if input.get("notes_prompt").and_then(|v| v.as_str()).is_none() {
        return Err(anyhow!("'notes_prompt' is required and must be a string"));
    }

    try_emit(
        deltas,
        LlmDelta::SheetInterviewOpen {
            payload: input.clone(),
        },
    );

    Ok(json!({
        "acknowledged": true,
        "user_must_submit": true,
        "next_step": "End this turn now. The user fills the panel and replies with one structured `[INTERVIEW SUBMISSION]` user message containing every section body (edited or original), every question's selected options + Other text, and the freeform Notes. On that next turn, parse the submission, then call `sheet_finalize_request` with the merged payload and finally answer the user's original question citing the persisted sheet."
    })
    .to_string())
}

/// Helper: re-query the sheet row by fingerprint and emit a `SheetLoaded`
/// delta carrying every field the frontend needs to populate the sidebar
/// `BSLinkedSheetCard`. Shared by `dispatch_sheet_finalize_request`
/// (post-insert, so the sidebar shows up the same turn) and
/// `dispatch_get_active_build_sheet` (every chat that finds an existing
/// sheet by fingerprint).
fn emit_sheet_loaded(
    deltas: &Option<mpsc::UnboundedSender<LlmDelta>>,
    row: &sheets::store::SheetRow,
    stale: bool,
) -> Result<()> {
    let payload = row.parse_payload().context("parse stored sheet payload")?;
    let payload_value = serde_json::to_value(&payload).context("serialize sheet payload to value")?;
    try_emit(
        deltas,
        LlmDelta::SheetLoaded {
            sheet_id: row.id.clone(),
            fingerprint: row.fingerprint.clone(),
            name: row.name.clone(),
            pob_hash: row.pob_hash.clone(),
            stale,
            authored_at: row.authored_at.clone(),
            updated_at: row.updated_at.clone(),
            schema_version: row.schema_version,
            payload: payload_value,
        },
    );
    Ok(())
}

/// Handle `sheet_finalize_request`. Persists the sheet via `sheets::store`
/// and, on success, emits both `SheetFinalized` (banner segment in the
/// chat timeline) and `SheetLoaded` (populates the sidebar
/// `BSLinkedSheetCard`).
pub async fn dispatch_sheet_finalize_request(
    input: &Value,
    deltas: &Option<mpsc::UnboundedSender<LlmDelta>>,
) -> Result<String> {
    let name = input
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("'name' is required"))?;
    let fingerprint = input
        .get("fingerprint")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("'fingerprint' is required"))?;
    let pob_hash = input
        .get("pob_hash")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("'pob_hash' is required"))?;
    let sections_raw = input
        .get("sections")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow!("'sections' must be an array"))?;

    let sections: Vec<BuildSheetSection> = sections_raw
        .iter()
        .map(|s| {
            let id = s.get("id").and_then(|v| v.as_str()).unwrap_or_default();
            let label = s.get("label").and_then(|v| v.as_str()).unwrap_or_default();
            let body = s.get("body").and_then(|v| v.as_str()).unwrap_or_default();
            let confirmed = s.get("confirmed").and_then(|v| v.as_bool()).unwrap_or(false);
            BuildSheetSection {
                id: id.to_string(),
                label: label.to_string(),
                body: body.to_string(),
                drafted: true,
                confirmed,
            }
        })
        .collect();

    let defining_items: Vec<DefiningItemEntry> = input
        .get("defining_items")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .map(|i| DefiningItemEntry {
                    name: i.get("name").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                    role: i.get("role").and_then(|v| v.as_str()).unwrap_or("enabler").to_string(),
                    purpose: i.get("purpose").and_then(|v| v.as_str()).map(String::from),
                })
                .collect()
        })
        .unwrap_or_default();

    let intent: Vec<IntentEntry> = input
        .get("intent")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|i| {
                    i.get("constraint")
                        .and_then(|v| v.as_str())
                        .map(|c| IntentEntry { constraint: c.to_string() })
                })
                .collect()
        })
        .unwrap_or_default();

    let known_gaps: Vec<KnownGap> = input
        .get("known_gaps")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|g| {
                    g.get("label").and_then(|v| v.as_str()).map(|label| KnownGap {
                        label: label.to_string(),
                        note: g.get("note").and_then(|v| v.as_str()).map(String::from),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let authored_in_chat = input
        .get("authored_in_chat")
        .and_then(|v| v.as_str())
        .map(String::from);

    let payload = BuildSheet {
        schema_version: SchemaVersion::CURRENT,
        sections,
        defining_items,
        intent,
        known_gaps,
    };

    let db = global_db()
        .ok_or_else(|| anyhow!("local database is not initialized; cannot persist sheet"))?;

    let id = sheets::store::insert_sheet(
        &db,
        fingerprint,
        pob_hash,
        name,
        &payload,
        authored_in_chat.as_deref(),
        true,
    )
    .context("insert build sheet")?;

    try_emit(
        deltas,
        LlmDelta::SheetFinalized {
            sheet_id: id.clone(),
            name: name.to_string(),
        },
    );

    // Re-query so the SheetLoaded delta carries authored_at / updated_at
    // / schema_version straight from the row, with no synthesis. If the
    // re-query fails for any reason, the banner still surfaces — the
    // sidebar will populate next turn when the agent reads the sheet.
    if let Ok(Some(row)) = sheets::store::find_by_fingerprint(&db, fingerprint) {
        let _ = emit_sheet_loaded(deltas, &row, false);
    }

    Ok(json!({
        "status": "saved",
        "sheet_id": id,
        "fingerprint": fingerprint,
        "validated": true,
    })
    .to_string())
}

/// Handle `get_active_build_sheet`. Reads the validated sheet for the given
/// fingerprint, plus a `stale` flag if `current_pob_hash` is provided. On
/// success also emits `SheetLoaded` so the sidebar `BSLinkedSheetCard`
/// populates without waiting for the next turn.
pub async fn dispatch_get_active_build_sheet(
    input: &Value,
    deltas: &Option<mpsc::UnboundedSender<LlmDelta>>,
) -> Result<String> {
    let fingerprint = input
        .get("fingerprint")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("'fingerprint' is required"))?;
    let current_pob_hash = input
        .get("current_pob_hash")
        .and_then(|v| v.as_str());

    let db = match global_db() {
        Some(db) => db,
        None => {
            return Ok(json!({
                "found": false,
                "reason": "db_unavailable",
            })
            .to_string());
        }
    };

    let row = match sheets::store::find_by_fingerprint(&db, fingerprint)? {
        Some(row) => row,
        None => {
            // The directive in `next_required_action` is the load-bearing
            // contract for the build-review pivot. The system prompt and the
            // build-review skill both tell the model "if no sheet, pivot to
            // the interview" — but those instructions are read once, far
            // upstream of the tool result. By the time the model sees this
            // payload it has already loaded the build-review skill and is
            // primed to run the diagnostic. Putting the pivot directive in
            // the tool result itself gives it temporal proximity: the model
            // just received this string, the very next action is the one
            // most likely to follow what the result says. Keep this
            // directive verbatim with `MANDATORY_PIVOT` and the explicit
            // forbidden-tools list — softer phrasing has been observed to
            // be ignored.
            return Ok(json!({
                "found": false,
                "fingerprint": fingerprint,
                "next_required_action": "MANDATORY_PIVOT: No Build Sheet exists for this build. Before answering the user's question, you MUST run the build-sheet interview. Your very next tool call must be `sheet_propose_section` with section.id='identity' (a draft of the Identity card). Do NOT call `pob_calc`, `wiki_search`, `wiki_open`, `read_internal_reference` for thresholds/vocabulary/etc., or any other research tool until every section of the sheet is `confirmed=true` and `sheet_finalize_request` has succeeded. The interview is short (4-7 turns), is paid for once per build, and lets every future chat about this character read from the sheet without re-asking. After finalize, return to the user's original question and answer it citing the sheet. The only override is if the user explicitly says 'skip the sheet' or 'audit me from scratch' — and then you must acknowledge the override in plain prose before proceeding. Read `read_internal_reference('32_build_sheets.md')` if you need the exact interview structure.",
                "interview_reference": "32_build_sheets.md",
            })
            .to_string());
        }
    };

    let stale = current_pob_hash
        .map(|h| h != row.pob_hash)
        .unwrap_or(false);
    let payload = row.parse_payload().context("parse stored sheet payload")?;

    // Populate the sidebar card right away. Errors here are non-fatal —
    // the agent's tool result still goes back; the user just won't see
    // the sidebar update until the next turn.
    let _ = emit_sheet_loaded(deltas, &row, stale);

    Ok(json!({
        "found": true,
        "sheet_id": row.id,
        "fingerprint": row.fingerprint,
        "name": row.name,
        "pob_hash": row.pob_hash,
        "stale": stale,
        "authored_at": row.authored_at,
        "updated_at": row.updated_at,
        "schema_version": row.schema_version,
        "payload": payload,
        "next_required_action": if stale {
            "STALE_SHEET: A sheet exists but the PoB hash differs since authoring. Surface the drift to the user before answering: name what changed (gear / gem swaps), then default to use-as-is unless the change clearly invalidates the framing. Same research rules as READ_FROM_SHEET_FIRST below: read the sheet, fill the gaps."
        } else {
            "READ_FROM_SHEET_FIRST: A validated sheet exists. Read sections.identity / archetype / damage / defense / items / intent before reaching for any other tool — they were authored from a deep PoB analysis at finalize time and often already contain the pob_calc numbers, threshold lookups, wiki facts about uniques, and intent constraints that bear on the user's question. If the sections answer it, cite them and skip duplicate research. If the question goes beyond what the sheet covers (a number that was not computed at authoring time, a mechanic the sheet does not name, a threshold that depends on the specific tier the user is asking about, recent patch impact), do exactly the missing research with pob_calc / wiki_open / kb_search / read_internal_reference — no more, no less. The sheet replaces the build-discovery interview; it does not replace the question-specific research that the sheet itself cannot answer. End your answer with `read_from_sheet · key1 · key2` in italic so the user sees which sheet sections you leaned on."
        },
    })
    .to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::Db;
    use crate::sheets::{compute_fingerprint, types::BuildSheet};

    fn fingerprint() -> String {
        compute_fingerprint(
            "Inquisitor",
            "Ice Nova of Frostbolts",
            &[
                "Brass Dome".to_string(),
                "Cospri's Will".to_string(),
            ],
        )
    }

    #[tokio::test]
    async fn finalize_then_lookup_round_trip() {
        let db = Db::open_in_memory().expect("mem db");
        // Seed one validated sheet.
        let payload = BuildSheet::with_default_sections();
        let fp = fingerprint();
        sheets::store::insert_sheet(&db, &fp, "h0", "Spell Crit · Cold", &payload, None, true)
            .expect("seed");

        // The dispatch handler talks to global_db, so wire it up.
        // (in_memory cannot be wired to global_db cleanly, so we exercise
        // the underlying store directly here. The tool handlers are thin
        // adapters around `sheets::store` — covered in integration smoke.)
        let row = sheets::store::find_by_fingerprint(&db, &fp).expect("query").expect("hit");
        assert!(row.validated);
        assert_eq!(row.name, "Spell Crit · Cold");
    }

    #[tokio::test]
    async fn ask_dispatch_emits_delta_and_returns_pause_marker() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let input = json!({
            "question_id": "cospri-purpose",
            "title": "What's the primary purpose of Cospri's Will here?",
            "options": [
                "Apply elemental ailments without crit",
                "Curse-on-hit Frostbite",
                "Defensive layer",
            ],
            "multi": false,
            "has_other": true,
        });
        let result = dispatch_sheet_ask(&input, &Some(tx)).await.expect("dispatch ok");
        let parsed: Value = serde_json::from_str(&result).expect("parse");
        assert_eq!(parsed["user_must_answer"], json!(true));
        assert_eq!(parsed["question_id"], json!("cospri-purpose"));

        let delta = rx.recv().await.expect("delta");
        match delta {
            LlmDelta::SheetAskUser { question_id, options, .. } => {
                assert_eq!(question_id, "cospri-purpose");
                assert_eq!(options.len(), 3);
            }
            other => panic!("unexpected delta {other:?}"),
        }
    }

    #[tokio::test]
    async fn propose_section_dispatch_emits_draft_delta() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let input = json!({
            "section_id": "damage",
            "title": "Damage scaling",
            "body": "Scales primarily off spell damage and crit multi.",
        });
        let result = dispatch_sheet_propose_section(&input, &Some(tx))
            .await
            .expect("dispatch ok");
        let parsed: Value = serde_json::from_str(&result).expect("parse");
        assert_eq!(parsed["user_must_confirm"], json!(true));

        let delta = rx.recv().await.expect("delta");
        match delta {
            LlmDelta::SheetDraftUpdate { section_id, body, confirmed, .. } => {
                assert_eq!(section_id, "damage");
                assert!(body.starts_with("Scales"));
                assert!(!confirmed);
            }
            other => panic!("unexpected delta {other:?}"),
        }
    }
}
