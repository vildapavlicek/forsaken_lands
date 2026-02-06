---
name: rust-coding-style
description: Code style guidelines and preferred idioms for Rust development in the Forsaken Lands project.
---

# Rust Code Style Guidelines

Current guidelines for writing idiomatic Rust code in this project.

## Iterators vs Loops

### Prefer `fold` for aggregations

When aggregating values from a collection, prefer using `Iterator::fold` over initializing a mutable variable and updating it in a `for` loop.

#### ❌ Wrong: Mutable Accumulator

```rust
let mut total = BonusStat::default();
for tag in &details.tags {
    if let Some(stat) = bonus_stats.get_with_prefix("damage", &tag) {
        total = total + *stat;
    }
}
```

#### ✅ Right: `fold`

```rust
let total = details.tags.iter().fold(BonusStat::default(), |acc, tag| {
    acc + bonus_stats
        .get_with_prefix("damage", tag)
        .cloned()
        .unwrap_or_default()
});
```
