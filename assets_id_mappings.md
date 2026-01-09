# Asset ID Mappings

## Research Unlocks

To ensure proper linkage between the Unlock system and the Research system, specific ID naming conventions must be followed.

### Naming Convention

For a Research Definition with ID `{NAME}`:

1.  **Research Asset** (`.research.ron`):
    *   `id`: `"{NAME}"`
    *   Example: `"basic_archery"`

2.  **Unlock Asset** (`.unlock.ron`):
    *   `id`: `"research_{NAME}_unlock"`
    *   `reward_id`: `"research_{NAME}"`
    *   Example:
        *   `id`: `"research_basic_archery_unlock"`
        *   `reward_id`: `"research_basic_archery"`

### Logic Flow

1.  **Unlock Achievement**:
    *   When the unlock condition is met, the system fires an `UnlockAchieved` event with the `reward_id` (e.g., `"research_basic_archery"`).

2.  **Research Availability**:
    *   The Research system listens for `UnlockAchieved`.
    *   It strips the `"research_"` prefix from the `reward_id` to get the target Research ID (e.g., `"basic_archery"`).
    *   It looks up the Research entity with that ID and changes its state from `Locked` to `Available`.

3.  **Initialization Reference**:
    *   During game startup, the system also checks `UnlockState` to see if `id` (e.g. `"research_basic_archery_unlock"`) has already been completed.
    *   If matched, the research entity is spawned directly in the `Available` state.

### Example Files

**`assets/research/basic_archery.research.ron`**
```ron
(
    id: "basic_archery",
    name: "Basic Archery",
    // ...
)
```

**`assets/unlocks/research_basic_archery.unlock.ron`**
```ron
(
    id: "research_basic_archery_unlock",
    display_name: Some("Basic Archery Research"),
    reward_id: "research_basic_archery",
    condition: ...
)
```
