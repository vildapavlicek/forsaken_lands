# Scribe's Journal - Critical Learnings

## 2026-02-19 - **Insight:** `VillageView` acts as a sub-state of `GameState::Running` but is implemented as a parallel `States` enum. Systems using it often assume `GameState::Running` is active. **Rule:** When documenting UI states, clarify if they rely on a parent GameState.
