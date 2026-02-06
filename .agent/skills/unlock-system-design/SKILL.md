---
description: Best practices and architecture for the Unlock and Reward system
---

# Unlock System Design

This skill documents the architecture and best practices for the Unlock and Reward system in Forsaken Lands.

## Core Architecture

The system uses an event-driven architecture to separate the *cause* of an unlock from the *effect* (reward).

### Flow
1. **Action**: Player performs an action (e.g., finishes research, crafts item).
2. **Trigger**: Game System emits `StatusCompleted`.
3. **Evaluation**: Unlock System listens to `StatusCompleted`, checks if it satisfies any `UnlockDefinition`.
4. **Unlock**: If satisfied, Unlock System emits `UnlockAchieved`.
5. **Reward**: Reward Systems (BonusStats, Recipes, etc.) listen to `UnlockAchieved` and enable features.

## Events

### `StatusCompleted` (Input)
Signals that a game state has changed or a task is finished.
- **Topic**: Unique identifier for the event (e.g., `"research:steel_sword"`, `"quest:intro"`).
- **Usage**: Emitted by gameplay systems (Research, Crafting, Questing).

### `UnlockAchieved` (Output)
Signals that a player has earned a reward.
- **Unlock ID**: ID of the unlock condition file/struct.
- **Reward ID**: ID used by reward systems to grant effects (e.g., `"research:steel_sword"`).
- **Usage**: Emitted **ONLY** by the `unlocks` crate. **DO NOT** trigger this manually from other systems.

## Defining Unlocks

### Inline Unlocks (Preferred)
Most assets (`ResearchDefinition`, `RecipeDefinition`, `StatBonusDefinition`) support declaring their own unlock condition inline. This avoids creating separate `.unlock.ron` files.

**Example (.stats.ron):**
```ron
(
    id: "research:autopsy_siled_alpha", // Reward ID (matches Reward System listener)
    bonuses: { ... },
    unlock: Some((
        id: "research:autopsy_siled_alpha_unlock", // Internal Unlock ID
        display_name: Some("Autopsy: Siled Alpha"), // UI Name
        reward_id: "research:autopsy_siled_alpha", // Must match asset ID
        condition: Leaf(Research(id: "autopsy_siled_alpha")), // Trigger
    )),
)
```

### Separate Unlock Files
For complex unlocks or rewards that don't map 1:1 to a specific asset, use a standalone `.unlock.ron` file in `assets/unlocks/`.

## Best Practices

### DO
- Use `StatusCompleted` to signal that your system has finished a task.
- Use `inline unlocks` for assets that unlock themselves (e.g., a Recipe unlock).
- Use `UnlockAchieved.reward_id` to listen for when your feature should be improved/enabled.

### DO NOT
- **DO NOT** manually trigger `UnlockAchieved` from your gameplay systems. This breaks the single source of truth and bypasses unlock conditions.
- **DO NOT** mix up `Unlock ID` (the condition) and `Reward ID` (the effect). They are often similar but serve different purposes.
