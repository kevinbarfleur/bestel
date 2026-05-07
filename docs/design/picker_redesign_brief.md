# Brief design — Bestel : refonte des modales pickers + passe lisibilité globale

Document de brief à destination d'un designer externe. Deux chantiers à couvrir,
dans l'ordre :

1. **Refonte des trois modales de sélection** (modèle / build / chat) — ergonomie
   et lisibilité actuellement déficientes.
2. **Passe globale lisibilité** sur toute l'application — typographie trop petite,
   italique abusif, hiérarchie floue.

Ce document décrit *ce qui doit être visible et avec quelle priorité*, pas la
mise en page. Les choix visuels (grilles, espacements, layout) sont à la main du
designer.

---

## 1. Contexte produit

**Bestel** est une app desktop (Tauri 2 + Vue 3) qui sert d'assistant IA pour
*Path of Exile 1 et 2* — un ARPG très complexe. L'utilisateur charge un build
(fichier XML exporté de Path of Building) et discute avec un agent LLM qui
analyse son personnage, recherche le wiki, et donne des conseils.

L'agent s'appelle "Bestel" et il a une persona : chroniqueur de *Lioneye's Watch*
(un PNJ de l'Acte 1 de PoE1). Le ton est légèrement archaïque mais lisible.

L'utilisateur peut :
- Choisir parmi plusieurs **modèles LLM** (Claude Haiku/Sonnet/Opus, DeepSeek,
  modèles locaux Ollama, etc.) — chacun a un coût, une vitesse, une dispo.
- Charger un **build PoB** (fichier XML local détecté automatiquement).
- Reprendre une **conversation** précédente.

Pour chaque modèle payant, l'utilisateur doit fournir une **clé API** (Anthropic
ou DeepSeek). Cette saisie se fait dans la modale du sélecteur de modèle.

---

## 2. Direction visuelle actuelle (à conserver dans l'esprit)

Le mood est volontairement **manuscrit / almanach annoté**, pas SaaS-dashboard.
Le designer précédent disait : « est-ce que ça pourrait apparaître dans un
grimoire annoté ? Si oui, c'est le mood. Si ça ressemble à un dashboard, c'est
cassé. »

Conséquences :
- **Pas de boîtes ni de cartes** avec ombre / fond plein. Les sections sont
  délimitées par des **filets** (`hairlines` 1px) et des **leader-dots**
  (`label ····· valeur`).
- **Pas de bouton solide à fond plein.** Les actions sont des **liens texte**
  (couleur ambre `#af6025`, soulignement pointillé) ou des labels en **petites
  capitales**.
- Couleurs principales : `--paper #f4f1ea` (parchemin), `--ink #2a2826` (encre),
  `--amber #af6025` (accent ambre) en clair ; inversion en mode sombre (tabac
  `#1d1a17` / crème `#ebe4d3`).
- Couleurs élémentaires (feu / froid / foudre / chaos / phys) pour les valeurs
  PoE liées aux résistances.

Cette direction reste valide. Les deux chantiers ne demandent **pas** un
re-skin — ils demandent que la grammaire actuelle soit appliquée plus
**lisiblement**.

---

## 3. Audit du design system actuel (typographie + italique)

### Polices

| Token | Police | Usage |
|---|---|---|
| `--serif` / `--hand` / `--hand-display` / `--label` | **EB Garamond** | Tout le UI (titres, body, labels, micro-caps). Une seule famille pour tout. |
| `--script` | **Kalam** (cursive) | Méta-info italique : "thinking", `h-section__meta`, certains placeholders. |
| `--script-display` | **Caveat** (cursive) | Annotations marginales, peu utilisé. |
| (pas de token) | **JetBrains Mono** | Valeurs techniques : preview de clé API, `model_id`, prix `$/Mtok`. |

### Échelle typographique actuelle (mesurée dans le code)

| Taille | Usage typique |
|---|---|
| **9 px** | Group label small caps (en-tête de groupe dans la liste), row meta (sous-titre micro d'un item de liste). |
| **10 px** | Caption de pane (`models`, `details`), `label-caps`, footer hint, `leader-row__k` (clé d'une ligne info). |
| **10.5–11 px** | Note italique sous un champ ("stored as plain text…"), texte mono d'accompagnement. |
| **11.5–12 px** | Mono technique (preview de clé API masquée, `model_id`). |
| **12–13 px** | Sous-titre italique ("via Anthropic API"), placeholder de champ, lien d'action. |
| **13–14 px** | Input value, `leader-row__v` (valeur d'une ligne info), titre de row dans la liste, body prose. |
| **14 px** | Description longue ("about" du modèle). |
| **22–24 px** | Titre principal de la pane détail (nom du modèle / classe du build). |

**Aucun élément du UI ne dépasse 24 px.** L'écart entre 9 px (méta) et 24 px
(titre) est très resserré.

### Italique — recensement des usages actuels

L'italique est utilisé un peu partout, sans vrai pattern :

- **Sous-titres descriptifs** : "via Anthropic API", "via local Ollama daemon".
- **Empty states** : "No models match.", "pick a model on the left to see its details.".
- **Notes de bas de champ** : "stored as plain text in `~/.bestel/runtime/keys.json`".
- **Placeholders** d'input : "paste ANTHROPIC_API_KEY…".
- **Méta de section** (`.h-section__meta` en Kalam italique).
- **Prose de "thinking"** dans le chat (segments de raisonnement de l'agent).

L'italique en EB Garamond, à 11–13 px, sur fond parchemin, **est très peu
contrasté**. C'est un des deux gros problèmes de lisibilité (l'autre étant les
small caps à 9–10 px).

### Couleurs de texte (token → usage)

- `--ink #2a2826` — texte principal (lu en confort).
- `--ink-soft #4a4744` — texte secondaire (encore confortable).
- `--ink-faint #8a8682` — texte de service (méta, hints, notes italiques) → **sous-contrasté à petite taille**.
- `--ink-ghost #b9b4a9` — placeholder, séparateurs → **très peu lisible**.
- `--amber #af6025` — accent / liens / actions.

### Hiérarchie visuelle actuelle des modales

Trois rangs :
1. **Titre H1 hand-display 24 px** (nom du modèle / build / chat) — bien.
2. **Sous-titre italique 12–13 px** ink-soft — peu contrasté.
3. **Tout le reste** (specs, prix, boutons, infos secondaires) en **10 px small
   caps `--ink-faint`** ou en **leader-row 10/14 px** — la lecture devient
   homogène et plate, on ne voit plus l'ordre d'importance.

---

## 4. Problèmes de lisibilité identifiés

### 4.1. Sur les modales (priorité 1)

**Modale "select a model"**
- L'utilisateur **ne comprend pas où coller sa clé API**. Le champ `ApiKeyField`
  est rendu après la section "links", lui-même après la section "specs". Pour le
  voir, il faut scroller — alors que c'est l'action principale quand on choisit
  un modèle qui en demande une.
- Le bouton d'action principal **"use this model"** est rendu comme un *lien
  texte* en bas à droite, en small caps 11 px ambre. Il ne ressemble pas à un
  bouton. Sur un public non-technique, le geste "cliquer ce lien pour activer le
  modèle" n'est pas intuitif.
- La liste à gauche montre 4 catégories de providers (`anthropic api`,
  `codex cli`, `claude code cli`, `ollama (local)`). Le label de groupe est en
  9 px small caps — quand on filtre par recherche, on ne voit pas bien dans
  quel groupe on est.
- L'**état "non disponible"** d'un modèle est rendu uniquement en grisant (pas
  de pictogramme, pas d'explication courte du *pourquoi*). L'utilisateur clique,
  rien ne se passe (toast d'erreur, mais le clic semble inactif).

**Modale "select a build"**
- Pas évident qu'il faut **double-cliquer ou appuyer sur Entrée** pour charger
  un build. Le panneau de droite affiche les stats du build *survolé*, pas du
  build *chargé* — distinction non explicite.
- Les **résistances** sont en 4 leader-rows colorées (feu / froid / foudre /
  chaos). Codé en couleurs élémentaires, mais à 14 px de valeur c'est limite et
  les couleurs sont ternes (calibrées pour parchemin). Difficile de spotter une
  résistance basse.
- Le bouton "load this build" est aussi un lien texte, comme dans le ModelPicker.

**Modale "select a chat"**
- Le bouton "+ start new" est un lien texte ambre, peu remarquable alors que
  c'est l'action la plus fréquente.
- La suppression d'un chat (delete inline avec confirm) est dans le panneau de
  droite — la cohérence "j'agis dans la liste" / "j'ai des infos à droite" est
  cassée.

### 4.2. Globaux (priorité 2)

- **Tout est en 10–14 px.** En dehors des H1 24 px, rien ne respire. Sur un
  écran 1080p à 100% de scaling, c'est tendu. Sur 4K à 100% c'est illisible.
- L'**italique en EB Garamond** ne fait pas son travail : à petite taille c'est
  juste plus pâteux, ça ne hiérarchise rien.
- Le `--ink-faint #8a8682` est beaucoup utilisé (méta, hints, notes) et tombe
  sous le seuil WCAG AA contre `--paper` à petite taille.
- Trop de small caps (`label-caps`, captions de pane, en-têtes de groupe,
  leader-row keys, footers de modal) — l'œil ne distingue plus les rangs.

---

## 5. Brief #1 — Refonte des modales pickers

**Forme de base** (à conserver) : modale grande (~90vw × 80vh), deux colonnes
(liste à gauche ~320 px, détail à droite). Fermable avec Esc. Navigable au
clavier (↑↓ ⏎ Esc).

Le designer est libre de remettre en cause la **forme à deux colonnes** s'il
juge qu'une autre disposition résout mieux les problèmes — mais il faut
préserver le clavier (le sélecteur s'ouvre avec `Ctrl+P` / `Ctrl+B` / `Ctrl+H`
et l'utilisateur s'attend à pouvoir tout faire au clavier).

### 5.1. Modale "select a model" — informations à afficher, par importance

#### Liste de gauche (sidebar)

**Doit être visible immédiatement** :
- Le **nom lisible** du modèle (`Claude Sonnet 4.5`, `DeepSeek V3.2`, `Qwen 2.5 7B`).
- Le **provider** auquel il est rattaché (Anthropic API / DeepSeek API / Codex CLI /
  Claude Code CLI / Ollama local) — actuellement traité en regroupement par
  catégories, mais on peut envisager autre chose (chip, icône, etc.).
- L'**état de disponibilité** : disponible (clé présente / daemon présent) /
  non disponible (clé manquante / daemon hors ligne). Doit être lisible en un
  coup d'œil.
- Le **modèle actif** doit être clairement marqué — c'est une info qui change
  rarement mais cruciale.

**Important mais secondaire** :
- Vitesse perçue (`fast` / `balanced` / `heavy`).
- Tier de coût (`cheap` / `mid` / `premium` / `subscription` / `free`).
- Prix `$X.XX / Mtok` quand connu (input et output peuvent différer).

**Filtrage** : champ de recherche en haut, qui filtre en live sur nom +
provider + tier + description.

#### Panneau de droite (détail du modèle survolé/sélectionné)

**Importance 1 — toujours visible sans scroller** :
- **Nom du modèle** (titre).
- **Provider** sous-titre.
- **Champ "API key"** quand le modèle en requiert une. C'est la **raison
  numéro un** pour laquelle l'utilisateur ouvre cette modale après la première
  installation. Aujourd'hui c'est masqué bas de page.
  - Si la clé n'est pas posée : input password + bouton "save" + bouton "show".
  - Si la clé est posée : preview masquée (`sk-ant-…4f2c`) + boutons "replace" +
    "remove".
  - Si la clé vient de l'environnement shell (et pas du disque) : indication
    explicite que Bestel ne l'écrasera pas.
  - Avertissement explicite : la clé est stockée en clair sur disque dans
    `~/.bestel/runtime/keys.json`.
- **Action principale "use this model"** — gros bouton, pas un lien. Désactivé
  si le modèle n'est pas dispo, avec une explication courte du pourquoi
  ("API key required" / "Provider not detected" / "Already active").

**Importance 2** :
- Specs : speed, cost tier, prix `$/Mtok`, model ID.
- État de disponibilité (avec raison si "non").
- Description prose (1–3 phrases sur ce que le modèle est fait pour).

**Importance 3** :
- Liens externes (docs du modèle, page pour obtenir une clé API). Aujourd'hui
  rendus en `↗` ; visibles mais pas critiques.

#### Pain points spécifiques à corriger
- L'**ApiKeyField doit remonter en priorité 1**.
- Le **bouton "use this model" doit être un vrai bouton** clairement actionnable.
- L'**état "indisponible"** doit avoir un libellé court et actionnable
  (idéalement : un CTA "Set API key" qui scrolle vers le champ ou ouvre un
  popover).

### 5.2. Modale "select a build"

#### Liste de gauche

**Importance 1** :
- **Nom de fichier** (ex. `TornadoShot.xml`) — ancrage utilisateur le plus fort.
- **Classe + ascendancy + niveau** (ex. `Ranger / Deadeye · lvl 92`).
- **Jeu** : `[PoE1]` ou `[PoE2]`.
- **Build actuellement chargé** clairement marqué.

**Importance 2** :
- Skill principal détecté (`Tornado Shot`, `Toxic Rain`).
- Date de dernière modification du fichier (`2h ago`, `3d ago`).

#### Panneau de droite (détail du build survolé/sélectionné)

**Importance 1** :
- En-tête : `Ranger / Deadeye · lvl 92` (jeu en chip à côté).
- **Vitals** : life, mana, energy shield, EHP. Quatre chiffres lisibles.
- **Résistances** : feu / froid / foudre / chaos. Quatre chiffres avec pénalités
  (négatif = rouge), proximité du cap visible. **Doit pouvoir être lu en moins
  de 2 secondes**, c'est la première chose qu'un joueur regarde.
- **Action principale** : "load this build" → gros bouton, pas un lien.

**Importance 2** :
- Skill principal + 2-4 skills secondaires.
- Spirit (PoE2 uniquement).
- DPS estimé si disponible.

**Importance 3** :
- Chemin du fichier complet.
- Date de modification.

#### Pain points spécifiques à corriger
- Le **build actuellement chargé** doit se distinguer de l'élément
  *survolé/sélectionné dans la liste* — actuellement l'utilisateur ne sait pas
  s'il regarde son build courant ou un autre.
- Les **résistances** doivent être assez grosses pour être scannables.
- L'**action de chargement** doit être explicitement un bouton.

### 5.3. Modale "select a chat"

#### Liste de gauche

**Importance 1** :
- **Titre du chat** (auto-généré depuis le premier message ou éditable).
- **Date relative** ("just now", "2h ago", "yesterday").
- Action **"+ new chat"** au sommet ou bas de liste — doit être visible et
  cliquable, pas un lien discret.

**Importance 2** :
- Nombre de tours dans la conversation.
- Modèle utilisé pour ce chat.
- Build attaché à ce chat (s'il y en a un).

#### Panneau de droite

**Importance 1** :
- Titre du chat.
- **Preview du dernier message** (utilisateur ou agent — au choix), 100–200
  caractères, pas en italique.
- **Action principale "open chat"** → gros bouton.

**Importance 2** :
- Métadonnées : modèle, build attaché, date de création, durée totale.

**Importance 3** :
- **Action "delete"** avec confirmation inline. Doit être visible mais pas
  proéminente — c'est une action destructive.

#### Pain points spécifiques à corriger
- L'**action "+ new chat"** doit être plus visible.
- La **suppression** doit avoir un état de confirmation explicite (pas juste un
  clic qui supprime sans warning).

### 5.4. Composants partagés à revoir en cohérence

- `PickerSearchInput` (champ de recherche en haut de sidebar).
- `PickerListItem` (row d'un item dans la sidebar).
- Le footer commun avec hints clavier (`↑↓ navigate · ⏎ select · esc close`) —
  actuellement en 10 px small caps, peu lisible.

---

## 6. Brief #2 — Passe globale lisibilité

À traiter **après** la refonte des modales (qui sera l'application concrète des
nouvelles règles). À discuter et valider avant pass globale, mais voici la
direction :

### 6.1. Suppression de l'italique

L'italique est à **éliminer du UI** (pas du contenu généré par l'agent — un
`<em>` dans une réponse markdown reste italique). Cibles :

- Sous-titres descriptifs ("via Anthropic API") → reste en regular, taille
  ajustée et couleur secondaire (`--ink-soft`).
- Notes de bas de champ ("stored as plain text…") → regular, plus grosse
  qu'aujourd'hui, ou repensée comme tooltip / icône info.
- Empty states → regular, taille body.
- Placeholders d'input → regular (la couleur faint suffit à distinguer).
- Méta de section (`.h-section__meta`, actuellement en Kalam italique) → à
  réconsidérer : soit suppression, soit Kalam regular à plus grosse taille.

L'**exception légitime** : les segments "thinking" de l'agent dans le chat —
c'est un effet narratif (pensée intérieure), pas une hiérarchie UI. Décision à
valider mais probablement à conserver.

### 6.2. Échelle typographique à élargir

Cible : passer d'un range **9–24 px** à un range **12–28 px** au minimum, avec
une vraie progression de rangs.

Proposition de structure (à adapter par le designer) :

| Rang | Usage | Taille indicative |
|---|---|---|
| H1 | Titre de pane détail (nom modèle / classe build / titre chat) | 24–28 px |
| H2 | En-tête de section ("specs", "api key", "vitals") | 14–16 px |
| Body | Description prose, valeurs de leader-rows | 14–16 px |
| Label | Clés de leader-rows, captions de pane | 12–13 px (au lieu de 10) |
| Meta | Sous-titres, métadonnées de row | 12 px regular (au lieu de 9–10 italique) |
| Mono | Valeurs techniques (clé API masquée, prix, model_id) | 13–14 px |

**Petites caps** : à utiliser **plus parcimonieusement**. Aujourd'hui elles
servent à la fois pour les en-têtes de section, les labels de leader-rows, les
captions de pane, les hints de footer, les actions text. C'est trop. Une règle
possible : *small caps réservées aux en-têtes de section et catégories*, le
reste passe en regular.

### 6.3. Hiérarchie visuelle plus marquée

L'œil doit pouvoir distinguer **trois rangs** sans effort dans n'importe quelle
modale ou panneau :

1. **Titre / contexte** (ce que je regarde).
2. **Action principale** (ce que je peux faire).
3. **Méta / contexte secondaire** (info de service).

Aujourd'hui le rang 2 est noyé dans le rang 3 (même couleur, même taille
quasiment). C'est *le* problème ergonomique majeur.

### 6.4. Boutons d'action principale = vrais boutons

Le mood "almanach" interdisait les boutons à fond plein. Cette règle peut être
**révisée pour les actions principales** des modales (use this model, load this
build, open chat, save api key). Soit :

- Solid ink (`--ink` background, `--paper` text).
- Outlined avec bordure pleine (pas pointillée) ambre ou ink.
- Cartouche encadré clairement.

À voir avec le designer comment ça cohabite avec les liens texte qui restent
pour les actions secondaires (replace key, remove key, open external link, etc.).

### 6.5. Contrastes et accessibilité

- Toutes les valeurs `--ink-faint` à petite taille (< 14 px) doivent passer en
  `--ink-soft` ou être agrandies.
- `--ink-ghost` ne doit pas porter de texte lisible — uniquement séparateurs et
  placeholders.
- Mode sombre doit être audité avec les nouvelles tailles : aujourd'hui les
  mêmes tokens sont utilisés, certaines combinaisons faiblissent en dark.

---

## 7. Contraintes techniques (à savoir)

- **Stack** : Vue 3 + TypeScript + Pinia + Tailwind (utilisé minimalement) +
  CSS scoped par composant. Pas de framework UI (Tailwind UI, etc.).
- **Tokens** dans `crates/bestel/ui/src/styles/tokens.css`. Typographie + tailles
  + transitions + z-index.
- **Grammaire commune** dans `crates/bestel/ui/src/styles/global.css` :
  classes utilitaires `.link`, `.link--soft`, `.label-caps`, `.h-section`,
  `.leader-row`. Toute refonte qui change ces classes est globale.
- **Composants pickers actuels** :
  - `crates/bestel/ui/src/components/build/ModelPicker.vue`
  - `crates/bestel/ui/src/components/build/BuildPicker.vue`
  - `crates/bestel/ui/src/components/chat/ChatPicker.vue`
  - `crates/bestel/ui/src/components/pickers/PickerLayout.vue`
  - `crates/bestel/ui/src/components/pickers/PickerSearchInput.vue`
  - `crates/bestel/ui/src/components/pickers/PickerListItem.vue`
  - `crates/bestel/ui/src/components/pickers/ApiKeyField.vue`
- **Modale parente** : `crates/bestel/ui/src/components/runic/RunicModal.vue`
  (avec variant `panes` qui définit la dimension utilisée par les pickers).
- **Toasts** disponibles via `useToastsStore` (variants : success / error /
  info / warning) — pour notifications "key saved", "build loaded", etc.
- **Mode clair / mode sombre** déjà câblés (`.theme-dark` sur `<html>`),
  togglable depuis la topbar. Tokens existent en double.
- **Polices** servies localement (woff2 sous `crates/bestel/ui/public/fonts/`),
  pas de Google Fonts runtime. Ajouter une police = embed.

---

## 8. Livrables attendus du designer

1. **Maquettes Figma** (ou équivalent) des trois modales, états :
   - Vide (aucun modèle survolé, ou aucune clé API posée).
   - Avec sélection (modèle survolé, clé saisie en cours, etc.).
   - État erreur (clé invalide, modèle indisponible).
   - Mode clair et mode sombre.
2. **Spécification typographique révisée** : échelle, usages, règles
   d'application des small caps, gestion de l'italique.
3. **Recommandation pour les boutons d'action principale** : style retenu
   (solide / outline / cartouche), variantes (primary / disabled / loading).
4. **Vue d'ensemble** d'un écran type post-refonte (chat principal + sidebar
   build) pour valider que la passe lisibilité globale tient debout.

Pas besoin de redessiner toute l'app — il faut donner les **règles** et **deux
ou trois écrans pivots** ; l'implémentation suivra.

---

## 9. Hors scope de ce brief

- Refonte du chat lui-même (rendu des messages, tool calls, artefacts) — déjà
  travaillé en phase précédente, peut être touché en passe lisibilité mais pas
  redesigné.
- Refonte de la topbar (logo, build pill, model pill, theme toggle, window
  controls) — peut être touchée en passe lisibilité.
- Refonte du panneau build (sidebar gauche du chat) — concerné par la passe
  lisibilité mais sa structure leader-dot fonctionne, on n'y touche que la
  typographie.
- Le viewer d'arbre passif (modale fullscreen Pixi.js) — pas concerné.
- L'identité visuelle générale (couleurs, mood manuscrit) — à conserver.
