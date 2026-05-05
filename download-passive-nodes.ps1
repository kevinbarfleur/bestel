# Download and reduce the passive-tree JSON for PoE1 + PoE2 into the
# compact format used by `crates/bestel-core/data/passive_nodes_*.json`.
#
# Sources:
#   PoE1 — https://raw.githubusercontent.com/PathOfBuildingCommunity/PathOfBuilding/master/src/TreeData/<latest>/tree.json
#   PoE2 — https://raw.githubusercontent.com/PathOfBuildingCommunity/PathOfBuilding-PoE2/main/src/TreeData/<latest>/tree.json
#
# Usage:
#   pwsh -File .\download-passive-nodes.ps1
#
# Currently the seed JSON is hand-curated with the most popular keystones and
# ascendancy nodes. This script is a placeholder — when GGG ships a new patch
# and you want full coverage, fill in the URLs above (or use the GGG endpoint
# https://www.pathofexile.com/passive-skill-tree/data/<treeVersion>) and write
# a parser that reduces every node to the {id, name, kind, description,
# ascendancy?} shape.
#
# For now, manual updates of the seed files are the recommended path —
# Bestel only needs the names and short descriptions; the full tree positional
# data is irrelevant for our hover tooltips.

Write-Host "Passive-tree dictionary regen is currently a placeholder."
Write-Host ""
Write-Host "The seed dictionaries at crates\bestel-core\data\passive_nodes_poe{1,2}.json"
Write-Host "are hand-curated and cover the most common keystones and ascendancy nodes."
Write-Host ""
Write-Host "To extend coverage, edit those JSON files directly. Schema:"
Write-Host '  {"id": 5934, "name": "Resolute Technique", "kind": "keystone",'
Write-Host '   "description": "Your hits can''t be Evaded. Never deal Critical Strikes."}'
Write-Host ""
Write-Host "kind ∈ keystone | notable | mastery | jewel_socket |"
Write-Host "       ascendancy_keystone | ascendancy_notable | ascendancy_small"
