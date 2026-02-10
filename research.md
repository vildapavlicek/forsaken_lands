The progression should follow from very basic and primitive tools to more complex ones by utilizing resources gathered from enemies.
Weapon upgrades should be closely followed by the research progression as well as enemy progression.

The draft for a progression should be as follows:

1. enemies appear from portal, we kill them with rocks we find
2. as more enemies appear from the portal, it is clear we might need better weapons
3. unlocked simple crafting research
4. sharp rocks, deal more damage with rocks 
5. as we kill more and more enemies their bodies start to pile up
6. unlock corpse scavenging research
7. unlock sling recipe
8. we need even better tools, bones are piling up
9. research bone crafting,
10. recipe bone mace, bone bow
11. break bones into bone dust to enhance bones, bones make toughter bones 
12. unlock weapon upgrades, repeatable research, affects only bone-crafted weapons

NOTE: Each weapons should be use-able for 2-3 levels before replaced by another. Thus 3 types of weapons should takes us from level
1 to level 9 or 10.
NOTE 2: For start, we will make the transition as meelee rock > sling > bone weapons, let's say meelee rock will be only weapon for first 3 levels when it should be replaced by sling, later weapons should come in pairs so player can choose

The issue with weapons is that in early levels ranged weapons will probably be preferred over melee due to ranged enemies

Levels 1-5 are meant to be tutorial levels, let player to familizarize with UI and core loop kill > scavenge > research

Here is the updated progression roadmap. I have aligned the **Research** and **Unlock** pacing to match the new enemy hierarchy (Siled -> Scavenger -> Soldier -> Beast -> Phalanx -> Brute -> Boss).

The weapon progression follows your rule:

* **Levels 1-3:** Rocks (Melee)
* **Levels 4-6:** Sling (Ranged)
* **Levels 7-9:** Bone Weapons (Choice: Sword vs Bow) + Upgrades

---

### Phase 1: Survival (The Tutorial)

#### **Level 1**

* **Theme:** Confusion & Defense.
* **Enemies:** `Siled` (0.5 HP)
* **Player Weapon:** Rock (Melee/Throw)
* **Research:**
* *Autopsy: Siled* (Learn enemy stats)
* *Invaders* (Story beat: "They aren't stopping.")


* **Next Level Condition:** Complete *Invaders* research (Requires 5 Kills).

#### **Level 2**

* **Theme:** Scavenging.
* **Enemies:** Group(`Siled`, `Siled Alpha` [0.75 HP])
* **Player Weapon:** Rock
* **Research:**
* *Autopsy: Alpha Siled*
* *Corpse Scavenging* (Unlocks inventory/loot drops).


* **Next Level Condition:** Complete *Corpse Scavenging* research (Requires 20 Kills).

#### **Level 3**

* **Theme:** Tool Making.
* **Enemies:** `Goblin Scout` (1.0 HP), Group(`Scout`, `Siled`)
* **Player Weapon:** Rock -> **Sling**
* **Research:**
* *Autopsy: Goblin Scout* (Reveals they have bones/items).
* *Simple Crafting* (Unlocks Crafting UI).
* *Recipe: Primitive Sling* (The first ranged weapon to counter Scouts).


* **Next Level Condition:** Craft *Primitive Sling*.

#### **Level 4**

* **Theme:** The Army Arrives.
* **Enemies:** `Goblin` (1.6 HP) — *Moved here from old Lvl 5.*
* **Player Weapon:** Sling
* **Research:**
* *Autopsy: Goblin*
* *Sharp Rocks* (Passive damage buff for the Sling ammo).


* **Next Level Condition:** Kill 20 Goblins (Prove you can handle the regular infantry).

#### **Level 5**

* **Theme:** The Vanguard & Resource Accumulation.
* **Enemies:** `Goblin Warrior` (2.8 HP), `Elite Scout` (2.2 HP) — *Significant HP jump.*
* **Player Weapon:** Sling (Struggling now)
* **Research:**
* *Autopsy: Warrior & Elite Scout*
* *Portal Discovery* (Story beat: "Where are they coming from?").
* *Bone Pile* (Unlocks a new storage resource: Bones).


* **Next Level Condition:** Amass 200 Bones (Preparation for Tier 2 weapons).

---

### Phase 2: Adaptation (The Beast & Brute Era)

#### **Level 6**

* **Theme:** The Beast Tamer’s Pack.
* **Enemies:** `Worg` (4.0 HP), `Goblin Rider` (5.2 HP) — *Fast & Tanky.*
* **Player Weapon:** Sling -> **Bone Set**
* **Research:**
* *Autopsy: Worg* (Unlocks Leather/Fur drops).
* *Bone Crafting* (Unlocks advanced crafting).
* *Recipe: Bone Sword* (High DPS Melee).
* *Recipe: Bone Bow* (High Dmg Ranged).


* **Next Level Condition:** Craft either a Bone Sword or Bone Bow.

#### **Level 7**

* **Theme:** The Phalanx (Tactical Wall).
* **Enemies:** `Goblin Defender` (6.5 HP), `Goblin Archer` (3.5 HP)
* **Player Weapon:** Bone Sword / Bone Bow
* **Research:**
* *Autopsy: Defender*
* *Vision* (Unlocks HP bars or enemy intent indicators).
* *Bone Idol* (Craftable artifact that boosts passive resource gain).


* **Next Level Condition:** Research *Entropy* (A mysterious resource harvested from the dead, needed for Level 8 upgrades).

#### **Level 8**

* **Theme:** The Heavy Infantry.
* **Enemies:** `Hobgoblin Mauler` (9.5 HP), Group(`Mauler`, `Rider`)
* **Player Weapon:** Bone Sword / Bone Bow (Needs upgrading).
* **Research:**
* *Autopsy: Hobgoblin*
* *Bone Dust Pulverizing* (Ability to crush bones into dust).
* *Weapon Enhancement* (Repeatable research: Spend Bone Dust to +1 your weapons).


* **Next Level Condition:** Upgrade a weapon to +3 (or similar power threshold).

#### **Level 9**

* **Theme:** The Boss Fight.
* **Enemies:** `Boss Worg Rider` (35.0 HP)
* **Player Weapon:** Enhanced Bone Weapon (+3 or higher).
* **Research:**
* *Warlord's Weakness* (Expensive research that slightly debuffs the boss).
* *Mana Traces* (Hints at Magic for Level 10).


* **Next Level Condition:** Defeat *Grolnak the Swift* (The Boss).

---


# WIP
# T1-10-19
1. Common/Raider Orcs

    Description: The "standard" grunt. They are usually slightly shorter than humans but broad-shouldered, stooped, and physically strong. They wear dirty, mismatched armor and use crude, heavy weapons.
    Behavior: Primitive and chaotic, they live in tribes and spend their time raiding, pillaging, and fighting.
    Media Examples: The Lord of the Rings (Misty Mountain/Mordor Orcs), Dungeons & Dragons (Mountain Orcs), Warhammer Fantasy. 

2. Elite/Uruk-hai

    Description: A stronger, larger, and more disciplined breed of orc, often created specifically for war. They are generally around human height, more upright, and often possess tougher hides.
    Behavior: More intelligent and organized than common orcs, they can operate in formation and often serve as elite shock troops or leaders of war parties.
    Media Examples: Saruman’s "Fighting Uruk-hai" in The Lord of the Rings, Black Uruks in Mordor. 

3. Goblinoids (Lesser Orcs)

    Description: Smaller, faster, and more numerous than standard orcs, often sharing the same culture or serving under them. They are often cowardly but dangerous in large numbers.
    Behavior: Scavengers and stealthy killers.
    Media Examples: Moria Goblins in The Lord of the Rings, Dungeons & Dragons Goblins/Hobgoblins. 

4. Specialized Orc Types

    Orc Shaman/Witch Doctor: Magic-using orcs that utilize black magic or divine magic from evil gods.
    Snufflers/Trackers: A breed with a highly developed sense of smell used for hunting.
    Warg-Riders: Orcs riding giant, vicious wolves or wargs, serving as fast cavalry.
    Half-Orcs: Human-orc hybrids that are sometimes more intelligent or cunning than pure-bred orcs, often acting as spies or leaders. 

5. Specialized Crossbreeds

    Orog/Ogrillon: The result of a crossbreed between a male orc and a female ogre, creating a much larger, tougher, and more muscular beast.
    Tanarukk: A D&D-specific, demon-tainted orc, which is shorter and stockier but far more savage. 
