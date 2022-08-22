- * Fallout shelter
 - Gas generator that will need refills
 - Crafting station at shelter
 - Pumps up water, player has hydration level/stamina
 - Autofarmer, kinda slow to generate rations
 - Fenced In, you can shelter there forever
- Airdrops
 - Buildable parts
 - One gas airdrop when gen runs out
 - Zombies will swarm them
 - Water bottle
- Craftable traps with recipes
- Endless survival
 - Zombies are attracted to noise
 - A drone could go out and make noise
- * Zombies
  - Randomly placed
  - * Mindless roam
  - Head to noise/sound
- * Player
 - * Point to shoot
 - * AD to rotate, WS to move forward/backwards
 - Rightclick/keybind to place trap
 - Compass
 - Gameover
   - Zombies can strike you
   - Runs out of water or food

# Crafting system

Airdrop items:

- Electronic components
- Battery pack
- Mechanical components
- Fuel
- Water
- Food
- Explosive
- Wood (always, crate is made of wood)

Player can carry items to the shelter, where they have a workshop. Combining items produces new items:

## Flying drone

Can pick up, carry and drop one other object. Battery lasts for 1 minute (empty) or 30s with an object.

electronics + battery + mechanical components

## Buzzer

After 10s timeout makes noise for 1 minute. Then the battery dies out.

electronics + battery

## Proximity bomb

electronics + battery + explosive

## Fragmentation bomb

proximity bomb + mechanical components

## Incendiary device

When exploded or on fire, sets other nearby object on fire. Doesn't explode on it's own. Has to be placed next to a bomb. Can cause a chain reaction if multiple placed close by.

fuel + explosive

## Crossbow

wooden parts + mechanical parts

## Bolts

Set of 12.

wooden parts + mechanical parts

## Sentry tower

Detects movement and notifies the player.

wooden parts + electronics + battery

## Defense tower

Shoots zombies. Can run out of bolts.

sentry tower + crossbow + bolts


## Development tasks

- [ ] Game states
  - [x] Running in the desert
  - [ ] Sheltering 
  - [ ] Paused
  - [ ] Game over (killed)
  - [ ] Game over (win)
  
- [ ] Make zombies movement backed by physics system
- [ ] Make player movement backed by physics system

- [ ] Design document

- [ ] Graphics
  - [ ] Ground texture
  - [ ] Player
  - [ ] Zombie
  - [ ] Bolt
  - [ ] Crate
    - [ ] Parachute
  - [ ] Bomb
  - [ ] Sentry tower
  - [ ] Defense tower
  - [ ] Incendiary device
  - [ ] Explosion
  - [ ] Flame
  - [ ] Drone
  
- [ ] Crafting system
- [ ] Items in crates
- [ ] Items in shelter
- [ ] Items in backpack
- [ ] Zombie attack
- [ ] Sounds
- [ ] Zombie hearing
