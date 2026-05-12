# Quality gates for Bestel development

## Universal gates

- Code is English-only.
- No unnecessary abstractions.
- Changed behavior has a test, eval, or documented manual validation.
- Docs and code do not contradict each other.

## Rust gates

- No `unwrap()` or `expect()` in runtime paths.
- Async I/O stays async.
- Errors include context at subsystem boundaries.
- DTO/event/schema changes are propagated.

## Frontend gates

- `npm run build` passes.
- Store ownership is clear.
- New backend event variants are handled explicitly.
- Markdown/panel rendering remains sanitized.

## LLM reliability gates

- Same-turn source evidence for final citations.
- No hidden reasoning tags in final text.
- No invented tool names in prompts.
- Tool call budget enforced in code when critical.
- PoE1 and PoE2 separated.

## Build Sheet gates

- Fresh sheet: read before build-specific synthesis.
- Stale sheet: read for intent/context, recalc numbers live.
- Absent sheet: do not use legacy per-section authoring for initial flow.
- Interview submission: finalize once, do not relaunch interview.

## PoB gates

- `pob_calc` numbers include effective Calcs config.
- Active skill metadata is checked before quoting DPS.
- Cache fallback is labeled stale.
- Fingerprint/hash behavior changes include migration or compatibility plan.
