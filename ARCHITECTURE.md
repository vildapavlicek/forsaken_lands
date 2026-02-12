# Forsaken Lands Architecture

## Overview
**Forsaken Lands** is a Rust-based game built on the **Bevy Engine (0.18)**. The project follows a "monorepo" workspace structure where functionality is highly granularized into separate crates (components, resources, systems, assets).

## Workspace Structure
The codebase is organized into several top-level directories, each serving a specific architectural role:

### 1. Core integration (`core/`)
-   **Role**: The glue that binds the application together.
-   **Key File**: `core/src/lib.rs` (the `CorePlugin`).
-   **Responsibility**: Registers all other feature plugins (`HeroesPlugin`, `CraftingPlugin`, etc.), initializes `GameState`, and configures global `SystemSet` scheduling.

### 2. Assets (`game_assets/`)
-   **Role**: Defines the data structures for game content.
-   **Pattern**: Pure data structs deriving `Asset`, `Serialize`, `Deserialize`, and `TypePath`.
-   **Format**: Assets are serialized as **RON** (Rusty Object Notation) files.
-   **Examples**: `ResearchDefinition`, `RecipeDefinition`.

### 3. Feature Crates (`heroes/`, `crafting/`, `buildings/`)
-   **Role**: Implement specific game features.
-   **Pattern**: Each feature typically exposes a `Plugin` that registers its own:
    -   **Components**: Data attached to entities (`components/*`).
    -   **Events**: Triggers for actions (`events/*`).
    -   **Resources**: Global state (`resources/*`).
    -   **Systems**: Logic that processes events and updates components.

### 4. Tooling (`tools/`)
-   **Key Tool**: `tools/asset_editor`.
-   **Role**: A standalone `egui` application used to create and edit game assets (RON files). This ensures type safety and validation for game data without manually editing text files.

## Key Design Patterns

### 1. The Plugin System
Everything is a Plugin. The `main.rs` simply adds the `CorePlugin`. The `CorePlugin` adds `HeroesPlugin`, `VillagePlugin`, etc.
*   **Encapsulation**: Features manage their own systems and resources.
*   **Dependency Injection**: Features declare dependencies in their `Cargo.toml`.

### 2. The "Pipeline" Pattern (e.g., Damage/Research Calculation)
Complex logic is decoupled from Bevy systems using pure Rust functions.
*   **Example**: `bonus_stats` crate.
*   **Mechanism**:
    1.  **Context Gathering**: A system gathers data (tags, base stats) for the calculation.
    2.  **Pure Calculation**: `calculate_stat(category, base_value, tags)` is called. This function is pure and testable outside of ECS.
    3.  **Execution**: The system applies the result (e.g., sets a timer duration or modifies HP).

### 3. Event-Driven Architecture
Systems rarely mutate state directly based on input. Instead, they emit **Events**.
*   **Flow**: `Input` -> `Action Event` -> `Processing System` -> `Component Update`.
*   **Benefit**: Decouples the "what happened" from the "how it is handled," allowing multiple systems to react to the same event (e.g., `EnemyDeath` triggers multiple systems: `LootDrop`, `XPGain`, `QuestProgress`).

### 4. Granular Components
Components are split into very small, focused creates (`components/hero_components`, `components/unlocks_components`).
*   **Benefit**: Reduces compile times and enforces strict dependency boundaries.
*   **Rule**: If multiple features need a component, it usually lives in `components/shared_components` or its own dedicated crate.

## Asset Management
Game content is data-driven via RON files located in `assets/`.
*   **Loading**: Assets are loaded using `bevy_common_assets` or custom loaders.
*   **Editing**: The `asset_editor` tool reads the source code definitions and lists available assets, effectively functioning as a CMS for the game.

## Developer Workflow
1.  **New Feature**: Create new crates for `components`, `events`, `resources`, and the feature logic itself.
2.  **New Asset**: Define struct in `game_assets/`, implementing `Asset`. Add to `asset_editor` for GUI support.
3.  **Logic**: Implement pure logic functions where possible, wrap in Bevy systems.
4.  **Integration**: Add the new Plugin to `CorePlugin`.
