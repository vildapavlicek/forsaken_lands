# (1) Invaders
Cost: Free
Requires: -
Time: 10s
ID: `invaders`

This is introductory research and story flavored one.
Point of this research is to give player sense that what comes from the portal
are enemies and not friends.
Also introduces research mechanics.

# (2a) Corpse Scavenging
Cost: Free
Requires: `invaders`
Time: 20s
ID: `corpse_scavenging`

Another story-flavor research. After we researched invaders a bit, we want to learn to process
them for useful materials. Since we start at primitive technology, only bones make sense now.

Unlocks: Bones (resource), Bone production (research), xykego (resource)

# (2b) Where did they come from?
Cost: None
Requires: `invaders`
Time: 20s
ID: `portal_discovery`

We have some basic understanding of the invaders but where did they come from?
This planet is supposed to be dead. Story-flavor research. Establishes portals
existence story-wise

Unlocks: Portal Research, `portal_creator`

# (3c) What is this?
Cost: 10 xykego
Requires: `invaders`
Time: 30s
ID: `xykego`

Every monster drops this strange object we do not understand

Unlocks: Theology I

# (3b) Who made the portal?
Cost: None,
Requires: `portal_discovery`
Time: 40s,
ID: `portal_creator`

Research into super natural, as there is no normal explanation for portal.
This establishes existence of the god which spawned portal.

Unlocks: Portal (research), Theology I (research)

# (3a) Bone crafting
Cost: 80 bones
Requires: corpse_scavenging
Time: 60s
ID: `bone_crafting`

Since corpse scavenging introduces first resource - bones, we want to figure out usage for those.
This research unlocks first bone crafts

Unlocks: Bone Sword (research), Bone Idol (research)

# (4a) Bone Sword
Cost: 100 bones
Requires: `bone_crafting`
Time: 90s
ID: `bone_sword`

We can craft from bones, but can we craft weapons from them?

Unlocks: Bone Sword (recipe)

# (4b) Theology I
Cost: 50 xykegos
Requires: `portal_creator`, `xykego`
Time: 60s
ID: `theology_i`

Is there a god? Can we please him?

Unlocks: Bone Idol (recipe), xykego (resource)

# (5c) A vision
Cost: 20 xykegos
Requires: `theology_i`
Time: 90s
ID: `vision_i`

The god has spoken to us. We shall present a gift of devotion.

Unlocks: `bone_idol`

# (5c) Hardened bones (0/10)
Cost: 100 bones (*2 per level)
Requires: `bone_production`
Time: 60s (*2 per level)
ID: `hardened_bones`

We found out we can grind bones to "bone dust" and use that
to enhance other bones durability.

Unlocks: Melee damage +2% (per level, 20% total)

# (6) Bone Idol
Cost: 50 bones
Requires: `bone_crafting`, `vision_i`
Time: 90s
ID: `bone_idol`

Would this be a good gift?

Unlocks: Bone Idol (recipe)

