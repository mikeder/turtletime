# Turtle Time 🐢

A multiplayer turtle game using [Bevy engine](https://bevyengine.org/), [bevy_ggrs](https://github.com/gschup/bevy_ggrs) and [matchbox_server](https://github.com/johanhelsing/matchbox/tree/main/matchbox_server) for P2P matchmaking/signaling.

Try it out here: https://mikeder.net/turtletime

## Features

* 4 players spawn in a rectangle and can move independently. [WASD]
* Eating strawberries will give you a temporary sprint ability. [LSHIFT]
* Eating chili peppers will give you the ability to shoot [5] fireballs! [SPACE] or [RETURN]
* Players hit by 3-4 fireballs are set inactive, last remaining active player wins!

## Road Map

Selectable number of players - [example](https://github.com/johanhelsing/matchbox/blob/main/examples/bevy_ggrs/src/box_game.rs)

## Inspiration

[Extreme Bevy](https://johanhelsing.studio/posts/extreme-bevy)

[RPG Tutorial](https://github.com/mwbryant/rpg-bevy-tutorial)

[GGRS Demo](https://github.com/gschup/bevy_ggrs_demo)
