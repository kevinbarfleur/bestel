---
name: bestel-rag-knowledge-workflow
description: Workflow for Bestel knowledge and retrieval work: LanceDB, hybrid search, chunk metadata, embeddings, reference docs, KB evals, and source provenance. Use when changing RAG or bundled knowledge.
---

# RAG and knowledge workflow

## Steps

1. Classify data type: conceptual reference, current fact, user-authored sheet, build-structured data, or eval artifact.
2. Use deterministic lookup for exact facts.
3. Use RAG for semantic discovery.
4. Add metadata filters before relying on prompt disambiguation.
5. Update KB eval queries when retrieval changes.
6. Keep source provenance attached to chunks.

## Metadata to preserve

- source path
- heading path
- game applicability
- source type
- freshness or version pin
- trust tier when applicable

## Do not

- treat internal references as final current-game citations
- index raw PoB as the primary authority for exact stats
- mix PoE1 and PoE2 retrieval without filters
