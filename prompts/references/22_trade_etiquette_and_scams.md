---
description: Whisper templates, courtesy windows, common scams (price-fix, listing-tax, item-swap, currency-at-off-rate, RMT), live-search expectations, hideout AFK conventions, party-kick on completion. PoE1 trade site + PoE2 trade site + Currency Exchange / Settlers Currency Exchange.
fetch_when: User asks about trading mechanics, "how to whisper", "I got scammed", or behaviour around the official trade site. Critical for advice quality and player safety. Always prioritise security advice over speed advice.
---

# 22 — Trade etiquette and scams

This doc is **player-safety-critical**. Bad trade advice → real currency / item loss. Conservative defaults preferred over speed.

The single most useful question this doc answers: *"Is this trade interaction normal or am I being scammed?"*

## Trade site fundamentals

PoE has no in-game auction house — trade happens via the **official web trade site** (pathofexile.com/trade and pathofexile.com/trade2 for PoE2) plus in-game whispers and trade windows.

- **Listing**: seller marks an item with price in their stash; the trade site indexes it.
- **Buyer flow**: search → find listing → press whisper-copy button → paste in-game whisper to seller.
- **Seller flow**: respond, party invite, trade in hideout / town, confirm, accept.

## Whisper templates

The trade site auto-generates a whisper of the form:
> Hi, I would like to buy your *Shaper's Touch* listed for *3 divine* in Settlers (stash tab "PriceLT"; position: left N, top M)

**Buyer rules**:
- **Do not modify the bracketed item ID** — the seller's UI matches against the exact string.
- Add a brief polite sentence at the end if you want; never alter the auto-generated body.
- One whisper per item per session is the convention. Don't spam.

**Seller rules**:
- Respond within ~2 minutes if active. "Sec" / "On the way" / "In 1 min" all standard.
- Party invite, then `/hideout` invite or trade in their hideout.
- If you can't honour the listed price (mistake / sold), apologise and remove the listing — don't haggle.

## Courtesy windows

| Listing age | Buyer expectation |
|---|---|
| Just now | Live-search; seller likely playing. ~2-min response window. |
| < 1 hour | Recent listing; seller probably online. Reasonable to whisper. |
| 1-24 hours | Cold but still legitimate; seller may be in a different activity. Slow response normal. |
| 1+ day old | Dead listing; assume sold-but-unflagged. Don't expect a response. |
| Hideout AFK | Sellers leaving items priced and going AFK is convention — don't spam re-whispers. |

A reasonable default: **send one whisper, wait 2 minutes, move on**. If seller accepts later, fine; otherwise don't escalate.

## In-trade-window etiquette

| Behaviour | Convention |
|---|---|
| Both sides accept = trade locks | Standard. |
| Item-swap by seller before final accept | Forbidden — re-accept required, and is a scam vector (see below). |
| Modifying currency amount mid-trade | Same: requires re-accept. |
| One side waiting for the other to add currency | Common — trade window allows asynchronous addition. |
| Party kick on trade complete | Normal. Not rude. |
| "Hidden currency" trade (random extra item bundled) | Rare; no convention exists. Decline politely. |

## Common scams

A complete list of patterns the agent should warn the user about. **Refusing a suspicious trade is always cheaper than recovering from a scam.**

### 1. Price-fix / Snipe bait

- Seller lists an item at way-below-market, baits inquiry.
- When buyer whispers, seller refuses to sell at listed price ("oh that was a typo, real price is N").
- **What it really is**: psychological anchoring — buyer feels invested in inquiry and may accept marked-up price.
- **Defence**: walk away. Listed price is the only valid price.

### 2. Listing tax / "Stash tab fee"

- Seller demands a small extra currency on top of listed price.
- Justified as "tax", "tip", "stash tab refresh fee".
- **What it really is**: rude markup attempt.
- **Defence**: walk away. Refer to the listed price.

### 3. Item-swap (the classic)

- Seller shows item A in trade window. Buyer ready to accept.
- Seller swaps to item B (a downgrade with same icon / similar name).
- Buyer auto-accepts on muscle memory; trade locks; loses currency.
- **What it really is**: the most common scam in PoE. Veteran players READ the mods *every accept*, especially after any seller modification.
- **Defence**: confirm mods, not icon, before each accept. Read the mods aloud if necessary.

### 4. Currency-equivalent shenanigans

- Trade is "X chaos = Y div" but the seller offers/demands at off-rate.
- Common with naive / new players who don't track current chaos:divine ratio.
- **Defence**: check current chaos:divine on poe.ninja before any currency-bulk trade.

### 5. Bulk-trade pricing trap

- Bulk listing for chaos sets (5/9 chaos for 1 div etc.).
- Seller delivers fewer items than agreed, hoping buyer doesn't count.
- **Defence**: count items in the trade window before accepting.

### 6. RMT (real-money trading) bait

- Stranger offers "free items" or "free currency" via party invite.
- Item is RMT-flagged; account-action risk on accept.
- **Defence**: never accept "free" anything from strangers. Every legitimate gift is a normal trade for ~1 chaos.

### 7. Account-impersonation

- Whisper from "GGG support" asking about an issue / requesting login info.
- **Defence**: GGG never DMs in-game. Anything asking for credentials is a scam.

### 8. Phishing site

- Whisper with a link claiming to be a price-check / build site.
- Site clones poe.ninja or PoB and prompts for password.
- **Defence**: never log into anything via in-game-whispered links.

### 9. Dummy listing (PoE1 specific)

- Seller lists multiple items at staggered prices, intentionally vague mods. Whispered, claims "oh I meant a different listing".
- **Defence**: cite the exact listing ID (auto-generated by whisper) when the seller equivocates.

### 10. Hideout AFK trap

- Seller leaves "open invite" claim in chat for an item, AFKs.
- Buyer arrives, no response, but seller has invite-on-arrival. Account-actions inadvertently allowed.
- **Defence**: don't enter unfamiliar hideouts unless explicitly invited via whisper-confirmed convention.

## Standards for safer trading

- **Always confirm mods, not icon, before accept.**
- **Always check chaos:divine current rate** for currency trades.
- **Always count items** in bulk trades.
- **Never accept "free"** from strangers.
- **Never log into linked sites** from in-game whispers.
- **Trust your gut**: a deal too good to be true probably is.

## PoE2 specifics

PoE2 retains the same trade-site model, with two notable differences:

- **Currency Exchange (PoE2 in-game system)** — bulk currency conversion at posted rates, in-game UI. Replaces some informal bulk-trade interactions.
- **Settlers Currency Exchange (PoE1 league mechanic)** — PoE1's equivalent. Rates are formal, set per league.
- **Both systems eliminate certain scam vectors** — the "off-rate currency trade" scam doesn't apply when the rate is posted.
- **Listing format same** — whisper templates identical between PoE1 and PoE2.

## Reporting and recovery

- If you've been scammed (item-swap, etc.): immediately report via the in-game `/report <name>` chat command + GGG ticket. Recovery is slow but possible.
- If you suspect RMT contact: ignore + report. Don't engage.
- If you encounter price-fix / listing-tax: just walk away. No reporting needed; just note the seller's name and avoid future listings.

## How the agent should behave

1. **When user asks "is this trade normal?"**: walk through the relevant scam-pattern list above.
2. **When user reports being scammed**: prioritise emotional support + tactical recovery (report-flow), not lecture.
3. **When user asks about specific currency**: cross-reference `21_currency_and_barter_taxonomy.md` for role + poe.ninja for current rate.
4. **Never tell user to "just trust the seller"** — neutral skepticism is the correct default.
5. **Default to suggesting Currency Exchange / Settlers** for bulk currency trades (rate-formalised, no scam vector).

## Cross-references

- `21_currency_and_barter_taxonomy.md` — currency types and roles.
- `15_source_registry.md` — poe.ninja for live rates, official trade site.
- `13_retrieval_playbooks.md` — trade lookup recipes.
- `26_validation_and_self_correction.md` Rule 4 — PoE1↔PoE2 trade-site disambiguation.
