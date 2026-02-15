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

#[derive(Event, Debug, Clone)]
pub struct RemoveStatBonus {
    pub key: String,
    pub value: f32,
    pub mode: StatMode,
}

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
