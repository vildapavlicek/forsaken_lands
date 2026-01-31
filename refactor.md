# Codebase Analysis & Refactoring Recommendations

## Summary
Analysis of the codebase (excluding asset_editor) identified improvements in readability, consistency, and simplification. The codebase generally uses modern Bevy patterns, but has some issues with mixed responsibilities (especially in `loading`), duplicated code, and monolithic files.

## 1. Readability Improvements

| File / Module | Issue | Recommendation |
| :--- | :--- | :--- |
| `loading/src/lib.rs` | **Monolithic File**: This file is over 600 lines and handles asset loading phases, entity spawning logic for multiple domains (research, crafting, enemies), manual save file parsing, and UI rendering. | **Split**: Move the `spawn_all_entities` logic into their respective crates (e.g., call `research::systems::spawn_research_entities`). Move UI logic to a `loading_ui.rs` module. Move save loading logic to `save_load` crate if possible. |
| `heroes/src/lib.rs` | **Large System File**: Contains definitions for all hero systems (movement, attack, projectile, collisions) in one file. | **Split**: Move specific systems into separate modules like `heroes/src/systems/combat.rs`, `heroes/src/systems/movement.rs`. |
| `research/src/systems_scratch.rs` | **Dead Code**: Contains duplicate/scratch code. | **Delete**: Remove this file entirely. |
| `unlocks/src/systems.rs` | **Mixed Responsibilities**: Contains compilation logic, signal propagation, and event handling. | **Refactor**: Move the compilation logic into `compiler.rs` or a dedicated `builders.rs` module. |

## 2. Consistency Improvements

| File / Module | Issue | Recommendation |
| :--- | :--- | :--- |
| `loading` vs `research`/`crafting` | **Logic Duplication**: The logic to check if a research/recipe is already unlocked or constructed is partially duplicated between `loading/src/lib.rs` and the spawning functions in `research`/`crafting`. | **Centralize**: The `spawn_research_entities` function in `research/src/systems.rs` should be the *only* place responsible for spawning research nodes. `loading` should just call this system. |
| `ui/research_ui/src/lib.rs` | **Pattern Consistency**: `ResearchUiPlugin` manually handles query iteration to build UI data lists in multiple places. | **Standardize**: Create a dedicated helper `ResearchUiDataBuilder` that takes the queries and returns the `Vec<ResearchDisplayData>`. |
| **Crate Structure** | Inconsistent exposure of `systems` module. | **Standardize**: consistently decide if `systems` are public or internal. |

## 3. Code Simplification & Refactoring

| File / Function | Issue | Recommendation |
| :--- | :--- | :--- |
| `ui/research_ui/src/lib.rs` | **Verbose UI Updates**: `handle_tab_switch` and `update_research_ui` copy-paste logic for gathering data. | **Refactor**: Extract data collection into a `build_current_research_data` function. |
| `loading/src/lib.rs` -> `check_assets_loaded` | **Explicit Dependency Checks**: Manually checking `asset_server.is_loaded_with_dependencies` for every folder is verbose. | **Simplify**: Iterate over a collection of handles/folders to check status. |
| `core/src/systems.rs` | **Empty File**: Contains only comments. | **Delete**: Remove if not used. |

## Proposed Action Plan
1.  **Cleanup**: Delete `research/src/systems_scratch.rs` and `core/src/systems.rs`.
2.  **Refactor Loading**: Split `loading/src/lib.rs` and delegate spawning logic back to domain crates.
3.  **Refactor UI**: Simplify `research_ui` to use a shared data builder.
4.  **Refactor Heroes**: Split `heroes/src/lib.rs` into smaller modules.
