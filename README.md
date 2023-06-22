# Turtle Time 🐢

A multiplayer turtle game using [Bevy engine](https://bevyengine.org/), [bevy_ggrs](https://github.com/gschup/bevy_ggrs) and [matchbox_server](https://github.com/johanhelsing/matchbox/tree/main/matchbox_server) for P2P matchmaking/signaling.

Try it out here: https://mikeder.net/turtletime

## Features

* 2-8 players spawn in a rectangle and can move independently. [WASD]
* Edible strawberries and chili peppers will spawn randomly on the map.
* Eating strawberries will give you a temporary sprint ability. [LSHIFT]
* Eating chili peppers will give you the ability to shoot [5] fireballs! [SPACE] or [RETURN]
* Players hit by enough fireballs are set inactive, last remaining active player wins!

## Inspiration

[Extreme Bevy](https://johanhelsing.studio/posts/extreme-bevy)

[RPG Tutorial](https://github.com/mwbryant/rpg-bevy-tutorial)

[GGRS Demo](https://github.com/gschup/bevy_ggrs_demo)

## TODO

1. Audio overhaul, replace the default sound w/ custom sounds.
2. Scoreboard and Leaderboard for game stats, pickups, wins, loses, etc.
3. More game modes, COOP survival, PVE.
4. Controller support.


