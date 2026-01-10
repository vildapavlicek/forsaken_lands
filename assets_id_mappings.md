# Asset ID Mappings & Unlock Conditions Guide

This guide explains how IDs work in the unlock system and how to properly define unlock conditions for new content.

## Overview

The unlock system uses **topics** to signal when conditions are met. Different systems emit events to different topics:

| Event Type | Topic Format | Example |
|------------|--------------|---------|
| Research Completed | `unlock:{research_id}` | `unlock:bone_weaponry` |
| Unlock Achieved | `unlock:{unlock_id}` | `unlock:research_bone_weaponry_unlock` |

---

## Research System

### Research Definition (`.research.ron`)

```ron
(
    id: "bone_weaponry",        // ← This is the research_id
    name: "Bone Weaponry",
    ...
)
```

When a player **completes** this research, the system emits:
- Topic: `unlock:bone_weaponry`

### Research Unlock Definition (`.unlock.ron`)

```ron
(
    id: "research_bone_weaponry_unlock",   // ← unlock_id (unique identifier)
    display_name: Some("Bone Weaponry"),
    reward_id: "research_bone_weaponry",   // ← Used by research system to identify which research to make available
    condition: Unlock("bone_crafting"),    // ← References a research_id (triggers when that research completes)
)
```

When this unlock **achieves**, the system:
1. Adds `"research_bone_weaponry_unlock"` to completed unlocks
2. Emits topic: `unlock:research_bone_weaponry_unlock`
3. Research system sees `reward_id: "research_bone_weaponry"` and makes that research available

---

## Recipe System

### Recipe Definition (`.recipe.ron`)

```ron
(
    id: "bone_sword",           // ← This is the recipe_id
    display_name: "Bone Sword",
    ...
)
```

### Recipe Unlock Definition (`.unlock.ron`)

```ron
(
    id: "recipe_bone_sword_unlock",        // ← unlock_id (unique identifier)
    display_name: Some("Bone Sword Recipe"),
    reward_id: "recipe_bone_sword",        // ← Used by crafting system (must match recipe_id)
    condition: Unlock("bone_weaponry"),    // ← References a research_id
)
```

---

## Choosing the Right Condition

### To unlock when **research completes**:
Use the **research_id** directly:
```ron
condition: Unlock("bone_weaponry")
```
This triggers when the player finishes researching "bone_weaponry".

### To unlock when **research becomes available**:
Use the **unlock_id** of the research unlock:
```ron
condition: Unlock("research_bone_weaponry_unlock")
```
This triggers when the research appears in the UI (before player starts it).

### To unlock based on **stats or resources**:
```ron
condition: And([
    Stat(StatCheck(stat_id: "goblin_kills", value: 10.0, op: Ge)),
    Resource(ResourceCheck(resource_id: "bones", amount: 5)),
])
```

---

## Quick Reference Table

| I want to unlock when... | Use this condition |
|--------------------------|-------------------|
| Research "X" is **completed** | `Unlock("X")` where X = research_id |
| Research "X" becomes **available** | `Unlock("research_X_unlock")` |
| Player kills N enemies | `Stat(StatCheck(...))` |
| Player has N resources | `Resource(ResourceCheck(...))` |
| Multiple conditions (all) | `And([...])` |
| Any condition (one of) | `Or([...])` |
| Always unlocked | `True` |

---

## Common Mistakes

❌ **Wrong:** `Unlock("research_bone_weaponry")` 
- This topic doesn't exist! No one emits `unlock:research_bone_weaponry`.

✅ **Correct:** `Unlock("bone_weaponry")` 
- Triggers when research with `id: "bone_weaponry"` completes.

❌ **Wrong:** Recipe `reward_id` doesn't match recipe `id`
```ron
// Recipe file
id: "bone_sword"

// Unlock file  
reward_id: "bone_sword_recipe"  // ← Won't work!
```

✅ **Correct:** `reward_id` matches recipe `id`
```ron
// Recipe file
id: "bone_sword"

// Unlock file
reward_id: "recipe_bone_sword"  // ← Matches with prefix
```

---

## Naming Conventions

| Asset Type | ID Pattern | Example |
|------------|------------|---------|
| Research Definition | `{name}` | `bone_weaponry` |
| Research Unlock | `research_{name}_unlock` | `research_bone_weaponry_unlock` |
| Recipe Definition | `{name}` | `bone_sword` |
| Recipe Unlock | `recipe_{name}_unlock` | `recipe_bone_sword_unlock` |
| Recipe reward_id | `recipe_{name}` | `recipe_bone_sword` |
