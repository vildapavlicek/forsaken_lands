# Scribe's Journal

## 2024-05-22 - **Insight:** **Rule:** Bevy 0.17 `trigger.target()` is deprecated.
In Bevy 0.17, `trigger.target()` is deprecated. Systems observing events should instead call `trigger.event()` to access the event data, and read the target entity from the event struct directly (if applicable).
