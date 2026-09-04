# Market Analysis Remediation Plan — for Antigravity

**Why this exists:** An independent review of `market_analysis.md` (by Claude Code, in a sibling worktree) spot-checked its claims against the actual repo. The architecture/technical claims held up exactly (scoring weights, ports, track count, `src/mcp.rs`). The market-sizing, competitor scoring, and "Victory Auditor" verification claims did not: zero external citations exist anywhere in the 1,629-line document, the competitor Index A/B scores have no disclosed rubric, the drill count is wrong (doc says 68, repo has 63), and the "100% Passed & Independently Verified" framing describes an audit that only checked internal arithmetic consistency — not whether the underlying market numbers are real.

The block below is formatted to match the existing prompts in `ORIGINAL_REQUEST.md` so it can be pasted directly to Antigravity as the next Teamwork Project request.

---

```markdown
# Teamwork Project: Market Analysis Integrity & Sourcing Remediation

Revise `market_analysis.md` for `cherenkov-lings` to fix factual errors against the actual repo, add real sourcing (or explicit unsourced-estimate labeling) to every market-sizing and demographic claim, disclose the competitor scoring rubric, and correct the document's own description of what its "audit" verified. This is a remediation pass on the existing deliverable, not a rewrite from scratch — preserve the structure, personas, curriculum mapping, and SWOT/roadmap content; fix what's broken.

Working directory: C:\Users\moaid\Documents\antigravity\wonderful-raman
Integrity mode: development

## Requirements

### R1. Reconcile Every Repo-Derived Claim Against Actual Source
Every number in the document that claims to describe `cherenkov-lings` itself (drill count, track count, port numbers, scoring weights, test counts, file paths) must be re-verified against the current repo state, not against the previous draft's own assertions. Known issue to fix: the document states "68 drills across 13 tracks" — the actual count under `exercises/` (mindepth 2, maxdepth 2 directories) is 63. Either correct the number or, if 68 is deliberately counting something else (e.g. including planned-but-unshipped drills), say so explicitly in the text rather than presenting it as a shipped count.

### R2. Source or Explicitly Label Every Market/Demographic Figure
None of the following currently have a named, dated source anywhere in the document: total software testing market size ($52.4B/$89.2B), TAM/SAM/SOM dollar figures, "4.2M QA professionals globally," "45% still manual," the 7.9% and 12.8% CAGR figures, the Katacoda shutdown date/cause, and every competitor's stated pricing/TCO. For each such figure:
- If a real source exists (a named industry report, analyst firm, public company filing, or survey, with a year), cite it inline.
- If no real source is available, keep the figure but relabel it inline as an explicit estimate — e.g. "Illustrative estimate (unsourced) — not independently verified" — directly next to the number in the table or prose, not only in a general caveats footnote.
Do not present modeled/invented numbers with the same confidence and formatting as sourced ones.

### R3. Disclose the Competitive Scoring Rubric
The 14-competitor Dual-Index scoring table currently presents precise decimal scores (e.g. "25.0/40," "22.0/40") with no explanation of how a given competitor earned that number versus a neighboring one. Add an explicit rubric: what each sub-criterion within Index A and Index B measures, its point range, and — for each competitor — one sentence of concrete evidence backing its score on that sub-criterion (a stated feature, a publicly known price, a publicly known latency figure). Where no concrete evidence is available for a sub-criterion, mark that cell as an estimate rather than presenting an unexplained decimal.

### R4. Correct the Document's Self-Description of Its Own Verification
The document (and its supporting `.agents/handoff.md` / Victory Auditor trail) currently claims "100% Passed & Independently Verified" and "VICTORY CONFIRMED" in a way that reads as validating the market research itself. In fact, per the handoff record, the audit validated: (a) internal arithmetic consistency (SOM = SAM × stated %, weights summing to 1.0), and (b) alignment between a handful of document claims and this repo's own source code/test suite. It did not independently verify any external market datum. Rewrite the verification language in `market_analysis.md`'s own text (not just the internal audit artifacts) to state precisely what was checked, and stop claiming independent verification of figures that were never checked against an external source.

## Acceptance Criteria

### Factual accuracy
- [ ] All drill/track/port/weight/file-path claims in `market_analysis.md` match the current repo state exactly, verified by re-running the relevant check (e.g. counting `exercises/*/*/` directories, grepping `src/feedback.rs` weight constants) — not by re-reading the prior draft.

### Sourcing integrity
- [ ] Every dollar figure and demographic/growth-rate statistic in the R3 (market sizing) section either carries a named, dated citation or an inline "unsourced estimate" label visible at the point of use.
- [ ] Zero numbers in the document present an invented figure with the same visual/textual confidence as a cited one.

### Scoring transparency
- [ ] The competitor benchmarking section states its Index A/B rubric (sub-criteria, point ranges) directly in `market_analysis.md`.
- [ ] Each competitor's score carries at least one line of stated evidence per sub-criterion, or is marked as an estimate.

### Verification honesty
- [ ] The document's own conclusion/attestation section accurately describes the audit as validating internal consistency and repo alignment, not external market accuracy.
- [ ] A new short section ("Data Provenance") near the top classifies the report's major claim categories as one of: Verified-from-repo, Externally-sourced-and-cited, or Estimate-unsourced — so a reader can tell at a glance which numbers to trust for external use (e.g. a pitch deck) and which are directional only.
```
