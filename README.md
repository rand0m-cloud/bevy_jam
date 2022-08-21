# Bevy Game Jam #2 - Combine

<https://itch.io/jam/bevy-jam-2>

This is our submission to the jam.

In our game you are a survivor of a zombie infestation, hiding in a small shelter in a vast wasteland. The shelter is surrounded by nothing but zombies as far as you can possibly go. Inside the shelter you are safe, but to survive you need food and water. There is an on-going rescue operation and occasionally the army air-drops crates with food, water and other items. The problem is that they are being dropped in random places in the desert and getting them is dangerous. The dangers include dehydration, starvation and of course getting bitten and infected.

The goal of the game is to survive and eventually get evacuated. Toward this end you will have to build and maintain various devices by combining the items found in the desert. These include improvised weapons and explosives. One of the devices you can make is a signaling gun. If you shoot it, after 5 minutes a helicopter will come and evacuate you. But it will also attract a lot of zombies, so you will have to survive those 5 minutes. The only way would be to fortify the landing zone a lot before calling the helicopter.

Items can be collected and moved in your backpack, but it has limited capacity. So you have to be strategic about what to take and where to store it. Best storage place is the shelter. It includes a small workshop, where you can craft new devices. There you can also recharge batteries for various devices you install in the desert. But to be able to travel far, you may chose to store some supplies in the desert. For that you can use the crates from airdrops.


## Staying Alive

You will die if:

1. You get completely dehydrated
2. You stave to death
3. A zombie bites or scratches you
4. Explosion blows you away
5. You catch fire... so play with fire carefully


### Hydration

You start with 100% hydration. It drops over time. The rate of dehydration depends on your circumstances.

| Circumstance           | Dehydration rate (pp / minute) |
|------------------------|--------------------------------|
| In the shelter         | 1                              |
| Standing in the desert | 2                              |
| Walking in the desert  | 3                              |
| Running in the desert  | 5                              |

If the hydration level is below 60% and there is a bottle of water around (in the crate, in your backpack or in the shelter), you will automatically drink it and restore 30 pp of hydration. So if venturing in the desert make sure to bring some water with you... and bring some more back to your shelter.


### Nutrition

You can survive without food a bit longer, but eventually starvation will kill you too. The nutrition works similar to hydration.

You start with 100% nutrition. It drops over time. The rate of starvation depends on your circumstances.

| Circumstance           | Starvation rate (pp / minute) |
|------------------------|-------------------------------|
| In the shelter         | 0.5                           |
| Standing in the desert | 0.5                           |
| Walking in the desert  | 1                             |
| Running in the desert  | 2                             |

If the nutrition level is below 60% and there is a food ratio (in the crate, in your backpack or in the shelter), you will automatically eat it and restore 30 pp of nutrition. 


### Zombies, hell yeah!

The zombies in the desert are rather stupid, but they make up for it with their numbers. If undisturbed, they will roam around mindlessly. But if they see you, they will run toward you and attack! The zombie will spot you only when you are in front of it (180° FOV) and no further than 100m (bad eye sight I guess).

They are not very fast and you can out outrun them. Just don't get surrounded! Here is how fast things go:

| What           | How fast (km / h) |
|----------------|-------------------|
| Player walking | 5                 |
| Player running | 12                |
| Zombie walking | 3                 |
| Zombie running | 9                 |


If a zombie gets closer than 1m to you it will bite with a 60% chance of infecting you. Infection is worse than death. Game over!

So you might think it's easy to stay out of their way or just outrun the zombies. Well, the problem is that zombies are attracted to noise. So be quite! But then another problem is that an excited zombie roars. And they get excited whenever they see you (you sexy thing) or when you hit them without immediate kill (see the Fighting Zombies section later). That means, that triggering one zombie will likely attract a whole lotta' more. It's really best to avoid them at all cost.

| Noise               | Base level (dB at 1m distance) |
|---------------------|--------------------------------|
| Player walking      | 20                             |
| Player running      | 50                             |
| Shooting a crossbow | 40                             |
| Opening a crate     | 50                             |
| Drinking water      | 30                             |
| Eating food         | 30                             |
| Bomb explosion      | 200                            |
| Drone flying        | 150                            |
| Zombie roaring      | 100                            |
| Buzzer ringing      | 80                             |


The sound level drops by 6 dB per doubling of distance (sound level = base level - 6 dB * sqrt(distance)). Zombies can hear sounds louder than 5 dB and will go toward the loudest sound they heard, until they hear something louder, reach the source of noise or spot the player. Examples:

A zombie can hear the player walking from 6m, but running from 55! So walk, don't run. Unless you are running from zombies. In that case good luck to you.

Say a zombie heard another one roaring 200 meters away. The noise level was ~15 dB so it will start walking toward the noise. While walking, it heard a bomb exploding 1000m away. Since the noise level was only about 10 dB (due to the distance), it will continue walking toward the roar. But if another bomb goes off just 800m away (~30 dB), it will abandon the roar and go toward the explosion. So strategic noise making can save your life! Just make sure to bang as far away from you as you can.


### Fighting Zombies 

TLDR: Don't.

If you are desperate, you can try shooting zombies with your crossbow. If you hit a zombie, there is a chance you kill it immediately and - what#s more important - quietly. The chance is 100% / square root of distance (in meters). So from 2 meters the chance is 70%, but you are unlikely to get that near, unless a zombie is charging at you. That might be your last chance to survive. Sneaking behind up to 10 meters seems doable, but then the chance is only 30%. The maximum range of the crossbow is 100m with a 10% chance of instant kill.

If you hit the zombie, but without killing it, it will roar and you will squeek. See above.

TODO: Shooting mechanics, bolts
TODO: Explosions and flames
TODO: Moving around in the desert, carrying capacity, actions, map, compass


## Crafting system

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

### Flying drone

Can pick up, carry and drop one other object. Battery lasts for 1 minute (empty) or 30s with an object.

electronics + battery + mechanical components

### Buzzer

After 10s timeout makes noise for 1 minute. Then the battery dies out.

electronics + battery

### Proximity bomb

electronics + battery + explosive

### Fragmentation bomb

proximity bomb + mechanical components

### Incendiary device

When exploded or on fire, sets other nearby object on fire. Doesn't explode on it's own. Has to be placed next to a bomb. Can cause a chain reaction if multiple placed close by.

fuel + explosive

### Crossbow

wooden parts + mechanical parts

### Bolts

Set of 12.

wooden parts + mechanical parts

### Sentry tower

Detects movement and notifies the player.

wooden parts + electronics + battery

### Defense tower

Shoots zombies. Can run out of bolts.

sentry tower + crossbow + bolts


## Development tasks

- [ ] Game states
  - [x] Running in the desert
  - [ ] Sheltering 
  - [ ] Paused
  - [ ] Game over (killed)
  - [ ] Game over (win)
  
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
