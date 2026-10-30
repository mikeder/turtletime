# Turtle Time 🐢

A multiplayer turtle game using [Bevy engine](https://bevyengine.org/), [bevy_ggrs](https://github.com/gschup/bevy_ggrs) and [matchbox_server](https://github.com/johanhelsing/matchbox/tree/main/matchbox_server) for P2P matchmaking/signaling.

Try it out here: https://mikeder.net/turtletime

## Features

* 2-8 players spawn in a rectangle and can move independently. [WASD]
* Edible strawberries and chili peppers will spawn randomly on the map.
* Eating strawberries will give you a temporary sprint ability. [LSHIFT]
* Eating chili peppers will give you the ability to shoot [5] fireballs! [SPACE] or [RETURN]
* Players hit by enough fireballs are set inactive, last remaining active player wins!

## Accounts and leaderboards

Accounts are optional and come from the [Tiny Game Platform](https://git.sqweeb.net/mikeder/tiny-game-platform). Signed in players get their online wins, losses and matches played tracked on leaderboards, and everyone sees the message of the day on the main menu. Local matches don't count.

To turn it on, set the game's application id in `src/platform/api.rs` (`APP_ID`), or at build time with `TGP_APP_ID=<id> cargo build`. Without one the Sign in and Leaderboards buttons are hidden. Then, in the platform dashboard for the game:

1. Declare three stats, each with aggregation **sum** and **client writable** on: `matches_played`, `wins`, `losses`.
2. Add a leaderboard for each stat you want to show (for example "Most wins" ranking `wins`, descending).
3. Add a `motd` title data entry, `{"text": "..."}` or a plain JSON string.
4. For the web build, add the site's origin (e.g. `https://mikeder.net`) to `tgp.allowed_origin`.

Each player reports their own results, since matches are peer to peer, so a modified client could fake its stats.

## Inspiration

[Extreme Bevy](https://johanhelsing.studio/posts/extreme-bevy)

[RPG Tutorial](https://github.com/mwbryant/rpg-bevy-tutorial)

[GGRS Demo](https://github.com/gschup/bevy_ggrs_demo)

## TODO

1. Scoreboard and Leaderboard for game stats, pickups, wins, loses, etc.
2. More game modes, COOP survival, PVE.
3. Controller support.
