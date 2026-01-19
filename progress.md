

# Description
This is working document that describes how the progress of the game should be.

# Tier 1-1
Introduction level, player should get used to basic loop: spawn monster - kill - spawn monster

## Monsters:
ID: siled
Name: Siled
HP: 0.5  
Spawn type: Single
Range: Melee
Drops:
  None (drops unlocked via research:corpse_scavenging)
  Bones 1 (100%)
  Xykego 1 (100%)
  Sinew 1 (25%)

## Research
### Invaders
ID: invaders
Condition: kills:siled >= 10
Time: 10s
Cost: None
Unlocks: research:corpse_scavenging
In-game text: They appeared on our lifeless planet. Black, red eyes, six legs..

Desc: This is first research and is more of a flavor research

### Corpse scavenging
ID: corpse_scavenging
Requires: research:invaders
Time: 20s
Cost: None
Unlocks: resource:bones, resource:xykego, resource:sinew, research:simple_crafting
In-game text: Their death is blessing to us.

Desc: This enables first resources used later as currency for research and crafting


# Tier 1-2
Unlock: resource:Bones >= 10

## Monster
ID: siled
Name: Siled
Spawn type: together with alpha
HP: 0.5
Range: Melee
Drops:
  Bones 1 (100%)
  Xykego 1 (100%)
  Sinew 1 (25%)

ID: siled_alpha
Name: Siled Alpha
HP: 1.0
Spawn type: together with with normal
Range: Melee
Drops:
  Bones 2 (100%)
  Xykego 1 (100%)
  Sinew 1 (25%)

## Research
### Simple crafting
ID: simple_crafting
Condition: research:corpse_scavenging
Time: 30s
Cost: 10 bones, 1 sinew
Unlocks: research:bone_sword, research:sling
In-game text: We can use their dead bodies to produce tools.

Desc: First research that leads to research for recipes.

### Bone Sword
ID: bone_sword
Condition: research:simple_crafting
Time: 60s
Cost: 20 bones
Unlocks: recipe:bone_sword
In-game text: Using their bones to get more bones.

Desc: First research to unlock crafting (or maybe second, sling might be first)

#### Bone Sword (recipe)
ID: bone_sword
Type: Melee weapon
Display name: Bone Sword
Damage: 1.5
Atk. Spd: 0.75s
Range: 150 (Melee)


### Sling
ID: sling
Condition: research:simple_crafting
Time: 90s
Cost: 10 sinew
Unlocks: recipe:primitive_sling
In-game text: Let's use their intestines to enhance our rock throwing.

Desc: Our first ranged weapon

#### Sling (Recipe)
ID: primitive_sling
Type: Ranged Weapon
Display name: Primitive Sling
Damage: 1.5
Atk. Spd: 0.90s
Range: 250 (Medium, Ranged)

### Xykego
ID: xykego
Condition: research:simple_crafting
Time: 90s
Cost: 10 sinew
Unlocks:
In-game text: They all have this but we do not know what it is..

### Portal
ID: portal_discovery
Condition: research:invaders, research:xykego
Time: 120s
Cost: None
Unlocks:

In-game text: Where do they come from?


Story flavor text

# Tier 1-3
Unlock: research:sling

Description: In this tier we will introduce first ranged enemy

## Monsters:
ID: goblin_scout
change: 40%
Name: Goblin Scout
HP: 2.0
Spawn type: single
Range: Medium range
  Bones 1 (100%)
  Xykego 1 (100%)
  Sinew 1 (25%)
  wood 1 (10%)

ID: siled
chance: 60%
Name: Siled
Spawn type: together with alpha
HP: 0.5
Range: Melee
Drops:
  Bones 2 (100%)
  Xykego 1 (100%)
  Sinew 1 (25%)

ID: siled_alpha
Name: Siled Alpha
HP: 1.0
Spawn type: together with with normal
Range: Melee
Drops:
  Bones 2 (100%)
  Xykego 1 (100%)
  Sinew 1 (25%)

## Research
### Research name
ID: theology_i
Condition: research:portal_discovery
Time: 150s
Cost: xykego 20
Unlocks:
In-game text: Who made the portal? Are there still gods?

Desc:

#### Sling (Recipe)
ID: primitive_sling
Display name: Primitive Sling
Damage: 1.5
Atk. Spd: 0.90s
Range: 250 (Medium, Ranged)

### Research name
ID: invaders_ii
Condition: kills:goblin_scout >= 1
Time: 150s
Cost: 
Unlocks:
In-game text: They are green and ugly with pointy features

Desc:

# Tier 1-4
Unlock: research:invaders_ii
Description:

## Monsters:
ID: goblin_scout
Chance: 60%
Name: Goblin Scout
HP: 2.0
Spawn type: single
Range: Medium range
Drops:
  Bones 2 (100%)
  Xykego 1 (100%)
  Sinew 1 (25%)
  wood 1 (10%)

ID: goblin
Chance: 40%
Name: Goblin
HP: 4.0
Spawn type: single
Range: Melee
Drops:
  Bones 3 (100%)
  Xykego 1 (100%)
  Sinew 1 (35%)
  wood 1 (20%)

## Research
### Vision I
ID: vision_i
Condition: research:theology_i 
Time: 180
Cost: Free
Unlocks:
In-game text: God has sent us a message. We shall worship


### Bone Crafting
ID: bone_crafting
Condition: research:vision_i 
Time: 150
Cost: 200 bones
Unlocks:
In-game text: We can craft more than just tools of war.

### Bone Idol
ID: bone_idol
Condition: research:bone_crafting
Time: 210s
Cost: 150 bones
Unlocks: recipe:bone_idol
In-game text: God shall be pleased

Desc:

#### Bone Idol (Recipe)
ID: bone_idol
Type: Idol
Display name: Bone Idol 

# Tier 1-5
Unlock: craft:bone_idol
Description:

## Monsters:
ID:
Name:
Spawn type:
Range:
Drops:

## Research
### Research name
ID:
Condition:
Time:
Cost:
Unlocks:
In-game text: 

Desc:

#### Item (Recipe)
ID: 
Display name: 
Damage: 1.5
Atk. Spd: 0.90s
Range: 250 (Medium, Ranged)

# Template:
```
# Tier X-Y
Unlock: 
Description:

## Monsters:
ID:
Name:
Spawn type:
Range:
Drops:

## Research
### Research name
ID:
Condition:
Time:
Cost:
Unlocks:
In-game text: 

Desc:

#### Item (Recipe)
ID: 
Type:
Display name: 
Damage: 1.5
Atk. Spd: 0.90s
Range: 250 (Medium, Ranged)
```
