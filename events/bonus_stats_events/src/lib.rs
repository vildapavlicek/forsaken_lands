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

/// Increments an existing bonus value in the global `BonusStats` resource.
///
/// This **Observer** event (triggered via `commands.trigger`) immediately adds a value
/// to an existing statistical modifier. It serves as a semantic alternative to
/// `AddStatBonus` specifically for progressive stat increments (e.g., leveling up a buff
/// rather than applying a new instance of it).
///
/// # Observers
/// - `bonus_stats::on_increase_stat_bonus`: Receives the event and increments the matching
///   stat in the `BonusStats` resource.
#[derive(Event, Debug, Clone)]
pub struct IncreaseStatBonus {
    pub key: String,
    pub value: f32,
    pub mode: StatMode,
}

#[derive(Event, Debug, Clone)]
pub struct DecreaseStatBonus {
    pub key: String,
    pub value: f32,
    pub mode: StatMode,
}
