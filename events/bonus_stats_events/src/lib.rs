use {bevy::prelude::*, bonus_stats_resources::StatMode};

/// Adds a new bonus to the global `BonusStats` resource.
///
/// This **Observer** event (triggered via `commands.trigger`) immediately modifies the
/// statistical modifiers used in game calculations (e.g., damage, health, research speed).
///
/// # Observers
/// - `bonus_stats::on_add_stat_bonus`: Receives the event and updates the `BonusStats` resource.
///
/// # Usage
/// Trigger this event to apply temporary or permanent buffs.
/// For example, equipping a weapon might trigger this to add "damage:melee".
#[derive(Event, Debug, Clone)]
pub struct AddStatBonus {
    /// The unique key for the stat (e.g., "damage:melee", "hp", "research").
    /// Categories are separated by colons.
    pub key: String,
    /// The numeric value of the bonus.
    pub value: f32,
    /// How this bonus aggregates with others (Additive, Percent, or Multiplicative).
    pub mode: StatMode,
}

/// Removes an active bonus from the global `BonusStats` resource.
///
/// This **Observer** event (triggered via `commands.trigger`) immediately reverses
/// a previously applied statistical modifier (e.g., when an item is unequipped or a buff expires).
///
/// # Observers
/// - `bonus_stats::on_remove_stat_bonus`: Receives the event and updates the `BonusStats` resource.
#[derive(Event, Debug, Clone)]
pub struct RemoveStatBonus {
    /// The unique key for the stat (e.g., "damage:melee", "hp", "research").
    pub key: String,
    /// The numeric value of the bonus to remove.
    pub value: f32,
    /// How this bonus aggregated with others (Additive, Percent, or Multiplicative).
    pub mode: StatMode,
}

/// Increments an existing statistical modifier in the global `BonusStats` resource.
///
/// This **Observer** event (triggered via `commands.trigger`) immediately adds a value
/// to a previously applied buff or passive bonus. It acts as a semantic alternative to
/// `AddStatBonus` for progressive stat scaling (e.g., leveling up an active power or stacking a buff).
///
/// # Observers
/// - `bonus_stats::on_increase_stat_bonus`: Receives the event and increments the tracked value in the `BonusStats` resource.
///
/// # Important Mechanic
/// - `StatMode::Multiplicative` bonuses stack additively (e.g., x2 + x2 = x4), not multiplicatively.
#[derive(Event, Debug, Clone)]
pub struct IncreaseStatBonus {
    /// The unique identifier of the stat to increment (e.g., "damage:melee", "hp:max").
    pub key: String,
    /// The numeric value to add to the existing bonus.
    pub value: f32,
    /// The mathematical mode defining how this bonus is applied. This must match the original mode of the stat.
    pub mode: StatMode,
}

#[derive(Event, Debug, Clone)]
pub struct DecreaseStatBonus {
    pub key: String,
    pub value: f32,
    pub mode: StatMode,
}
