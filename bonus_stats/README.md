# Bonus Stats System

This crate manages dynamic statistical bonuses using a flexible **Topic:Key** architecture. It is designed to allow arbitrary tagging for stats like damage, armor, health, etc., without rigid schema changes.

## Core Concept: Topic & Key

The system organizes bonuses into a two-level hierarchy map: `HashMap<Topic, HashMap<Key, Bonus>>`.

- **Topic** (Category): The high-level stat type (e.g., `damage`, `armor`, `hp`, `move_speed`).
- **Key** (Sub-Key/Identifier): The specific condition or source of the bonus (e.g., `melee`, `fire`, `race:goblins`, `heavy`).

This structure allows for limitless keys under any topic.

### Key Parsing Logic

When a bonus is added with a string key like `"damage:melee"`, it is parsed as:
- **Topic**: `damage`
- **Key**: `melee`

If a complex key like `"damage:race:goblins"` is used:
- **Topic**: `damage`
- **Key**: `race:goblins`

## Usage in Damage Calculation

The primary example of this system is `calculate_damage` in `bonus_stats_resources`. It resolves the final bonus by querying the `damage` topic against provided tags.

### 1. Source Tags (e.g., Weapon Tags)
Source tags define properties of the *attacker* or *weapon*.
- **Format Expected**: `topic:key` (e.g., `damage:melee`, `damage:fire`).
- **Logic**: The system iterates over source tags, checks if they start with `damage:`, and then looks up the *suffix* (`key`) in the `damage` topic.
- **Example**: 
  - Tag: `damage:melee` -> Look up Key: `melee` in Topic: `damage`.
  - Tag: `bone_sword` -> Ignored (no `damage:` prefix).

### 2. Target Tags (e.g., Monster Tags)
Target tags define properties of the *defender* or *victim*.
- **Format Expected**: `key` (e.g., `race:goblins`, `siled`, `boss`).
- **Logic**: The system iterates over target tags and looks them up *directly* as keys in the `damage` topic.
- **Example**:
  - Tag: `race:goblins` -> Look up Key: `race:goblins` in Topic: `damage`.

## Adding New Bonus Types (Extensibility)

To introduce a new stat, for example **Armor**, you do **not** need to modify the data structure. You simply start using the new Topic.

### Example: Adding Armor Bonuses

1.  **Define Bonuses**:
    ```rust
    // Add +5 armor against Orcs
    bonus_stats.add("armor:race:orcs", StatBonus { value: 5.0, mode: StatMode::Additive });
    
    // Add +10 flat armor from distinct source
    bonus_stats.add("armor:base", StatBonus { value: 10.0, mode: StatMode::Additive });
    ```

2.  **Calculate Armor**:
    Implement a function similar to `calculate_damage`:
    ```rust
    pub fn calculate_armor(base: f32, target_tags: &[String], stats: &BonusStats) -> f32 {
        let mut total = BonusStat::default();
        if let Some(armor_bonuses) = stats.bonuses.get("armor") {
             // Look up target tags directly
             for tag in target_tags {
                 if let Some(bonus) = armor_bonuses.get(tag) {
                     total += *bonus;
                 }
             }
             // You could also support source tags if needed (e.g. piercing armor)
        }
        // ... apply total to base ...
    }
    ```

## Example Scenarios

| Stat String | Topic | Key | Usage Meaning |
| :--- | :--- | :--- | :--- |
| `damage:melee` | `damage` | `melee` | Bonus applied when source has `damage:melee` tag. |
| `damage:race:goblins` | `damage` | `race:goblins` | Bonus applied when target has `race:goblins` tag. |
| `xp_gain:quest` | `xp_gain` | `quest` | Bonus applied to XP events tagged with `quest`. |
| `armor:heavy` | `armor` | `heavy` | Bonus applied when checking armor against `heavy` attacks? (Depends on logic). |

This system is completely data-driven and agnostic to the specific game logic, allowing designers to invent new stat relationships (tags) without code changes, provided the calculation logic supports looking them up.
