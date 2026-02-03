---
description: Bevy 0.17 API reference and common migration issues
---

# Bevy 0.17 API Notes

This document captures API changes and common issues when working with Bevy 0.17.

## Hierarchy Changes: Parent vs ChildOf

The `Parent` component has been renamed/replaced by `ChildOf`.
- **Old:** `Parent` component with `.get()` to retrieve parent entity.
- **New:** `ChildOf` component with `.parent()` method to retrieve parent entity.

## Event vs Message Bifurcation

In Bevy 0.17, events and messages have been split into two distinct systems:

### Events (Observer Pattern)
- Use `#[derive(Event)]` macro
- Trigger with `commands.trigger(MyEvent { ... })`
- Handle with observer functions using `On<MyEvent>`
- Do NOT need `add_event::<MyEvent>()` registration
- Register observer with `.add_observer(my_observer_fn)`

```rust
#[derive(Event)]
pub struct MyEvent {
    pub data: String,
}

fn my_observer(trigger: On<MyEvent>, mut commands: Commands) {
    let event = trigger.event();
    // Handle event...
}

// In plugin:
app.add_observer(my_observer);
```

### Entity-Targeted Events (Bubbling)
- Use `#[derive(EntityEvent)]` with `#[entity_event(propagate)]`
- Trigger on specific entity: `commands.entity(e).trigger(|e| MyEvent { entity: e, ... })`
- Propagation controlled via `trigger.propagate(true/false)`

```rust
#[derive(EntityEvent)]
#[entity_event(propagate)]
pub struct LogicSignalEvent {
    #[event_target]
    pub entity: Entity,
    pub is_high: bool,
}
```

### Messages (Traditional EventReader/EventWriter)
- Still use `add_event::<MyMessage>()` registration
- Use `EventReader<MyMessage>` and `EventWriter<MyMessage>`
- Suitable for bulk processing, not observer pattern

## Deprecated APIs

### Query::get_single() → Query::single()

**Deprecated:**
```rust
if let Ok(value) = query.get_single() { ... }
```

**Use instead:**
```rust
if let Ok(value) = query.single() { ... }
```

The same applies to `get_single_mut()` → `single_mut()`.

## Common Patterns

### Triggering Events from Game Code
```rust
// Numeric value changes
commands.trigger(ValueChanged {
    topic: format!("kills:{}", monster_id),
    value: kill_count as f32,
});

// Completion states
commands.trigger(StatusCompleted {
    topic: format!("research:{}", research_id),
});
```

### Orphan Rule Workaround
When you need a type shared between two crates, define it in one crate and have the other import it. You cannot implement `From<A>` for `B` if both `A` and `B` are foreign types.

```rust
// Define ComparisonOp in unlocks_components
// Import it in unlocks_assets via dependency
use unlocks_components::ComparisonOp;
```

## RON Asset Syntax

### Struct-like Enum Variants
In RON, struct-like enum variants **must use parentheses** `()` not braces `{}`.

**Incorrect (will fail to parse):**
```ron
condition: Completed { topic: "research:bone_sword" }
condition: Value { topic: "kills:goblin", target: 10.0, op: Ge }
```

**Correct:**
```ron
condition: Completed(topic: "research:bone_sword")
condition: Value(topic: "kills:goblin", target: 10.0, op: Ge)
```

This applies to any Rust enum with named fields:
