# Bestel

> *"Let me tell you a tale, exile."*

**Bestel** est un assistant IA en terminal pour **Path of Exile 1 & 2**, incarné par le chroniqueur de Lioneye's Watch. Il voit ton build Path of Building en direct, t'aide sur les mécaniques, l'optimisation de personnage, l'économie de la ligue.

## v0.1 — vertical slice

- TUI Ratatui (Windows-first, fonctionne aussi Linux/Mac)
- Watcher Path of Building : tout build sauvegardé est récupéré automatiquement
- Provider : **Anthropic** (clé API)
- Outil interne : `get_active_build`

## Prérequis

- Path of Building (PoE1) ou Path of Building (PoE2) installé
- Une clé API Anthropic dans `ANTHROPIC_API_KEY`
- Rust 1.80+

## Build & lancement

```sh
cargo build --release
$env:ANTHROPIC_API_KEY = "sk-ant-..."
.\target\release\bestel.exe
```

Sauvegarde un build dans PoB → il apparaît dans le panneau de gauche. Pose une question à Bestel à droite.

## Roadmap

Voir [ROADMAP.md](ROADMAP.md). Au programme : OpenAI/Gemini, fetch wiki live, vision/screenshots, serveur MCP, PoB headless.

## Licence

MIT.
