pub const SYSTEM_PROMPT: &str = include_str!("../../../SYSTEM_PROMPT.md");

pub const CORE_KNOWLEDGE: &str = include_str!("../CORE_KNOWLEDGE.md");

pub const SYSTEM_PROMPT_COMPOSED: &str = concat!(
    include_str!("../../../SYSTEM_PROMPT.md"),
    "\n\n---\n\n",
    include_str!("../CORE_KNOWLEDGE.md"),
);
