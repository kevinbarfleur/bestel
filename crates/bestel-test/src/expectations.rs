//! Expectation evaluation: read the JSON output from `bestel chat` and
//! check each expectation listed in a scenario file.

use serde_json::Value;

use bestel_core::test_runner::Expectation;

#[derive(Debug)]
pub struct EvalResult {
    pub passed: bool,
    pub failures: Vec<String>,
}

pub fn evaluate(exp: &Expectation, run_json: &Value) -> EvalResult {
    let mut failures: Vec<String> = Vec::new();

    let final_text = run_json
        .get("final_text")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let tool_calls = run_json
        .get("tool_calls")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let sources = run_json
        .get("sources")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let must_match = match exp.must_match_compiled() {
        Ok(v) => v,
        Err(e) => {
            return EvalResult {
                passed: false,
                failures: vec![format!("invalid must_match regex: {e}")],
            }
        }
    };
    let must_not_match = match exp.must_not_match_compiled() {
        Ok(v) => v,
        Err(e) => {
            return EvalResult {
                passed: false,
                failures: vec![format!("invalid must_not_match regex: {e}")],
            }
        }
    };
    for re in &must_match {
        if !re.is_match(final_text) {
            failures.push(format!("missing pattern in answer: /{}/", re.as_str()));
        }
    }
    for re in &must_not_match {
        if re.is_match(final_text) {
            failures.push(format!(
                "forbidden pattern present in answer: /{}/",
                re.as_str()
            ));
        }
    }

    if let Some(name) = &exp.must_call_tool {
        let called = tool_calls
            .iter()
            .any(|c| c.get("name").and_then(|n| n.as_str()) == Some(name));
        if !called {
            failures.push(format!(
                "expected tool '{}' was not called (saw: {})",
                name,
                tool_calls
                    .iter()
                    .filter_map(|c| c.get("name").and_then(|n| n.as_str()))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }

    for forbidden in &exp.forbid_tool {
        if tool_calls
            .iter()
            .any(|c| c.get("name").and_then(|n| n.as_str()) == Some(forbidden))
        {
            failures.push(format!("forbidden tool '{}' was called", forbidden));
        }
    }

    if exp.min_final_text_len > 0 {
        let len = final_text.chars().count();
        if len < exp.min_final_text_len {
            failures.push(format!(
                "final answer too short: {} chars < min {}",
                len, exp.min_final_text_len
            ));
        }
    }

    if exp.min_tool_calls > 0 && tool_calls.len() < exp.min_tool_calls {
        failures.push(format!(
            "too few tool calls: {} < min {}",
            tool_calls.len(),
            exp.min_tool_calls
        ));
    }

    // must_cite_domain semantics: AT LEAST ONE of the listed domains must be
    // cited (OR, not AND). Multiple entries express alternatives.
    if !exp.must_cite_domain.is_empty() {
        let any_cited = exp.must_cite_domain.iter().any(|domain| {
            let in_sources = sources.iter().any(|s| {
                s.as_str()
                    .map(|s| host_of(s).map(|h| host_matches(&h, domain)).unwrap_or(false))
                    .unwrap_or(false)
            });
            if in_sources {
                return true;
            }
            // Fallback: bare URLs in the answer body.
            final_text.split_whitespace().any(|tok| {
                host_of(tok).map(|h| host_matches(&h, domain)).unwrap_or(false)
            })
        });
        if !any_cited {
            failures.push(format!(
                "expected citation of any domain {:?} missing (sources: {:?})",
                exp.must_cite_domain,
                sources.iter().filter_map(|s| s.as_str()).collect::<Vec<_>>()
            ));
        }
    }

    EvalResult {
        passed: failures.is_empty(),
        failures,
    }
}

fn host_of(url: &str) -> Option<String> {
    let after_scheme = url.split("://").nth(1)?;
    let host_path = after_scheme.split('/').next()?;
    let host_only = host_path.split('@').last()?;
    Some(host_only.split(':').next()?.to_lowercase())
}

fn host_matches(host: &str, expected: &str) -> bool {
    let expected = expected.trim().to_lowercase();
    host == expected || host.ends_with(&format!(".{expected}"))
}
