# Bevoids

Bevoids is a small game, heavily inspired by the classic `Asteroids`.

Written in [Rust] using [Bevy]

![Game menu](images/menu.jpg) ![Game Playing](images/playing.jpg)

## Installation

Currently theres no easy way of installing the game 😢

However, clone the repo and run the game using:

```shell
cargo run --release -- --assets $PWD/bevoids_game/assets
```

Should work on both Linux and Windows, though you'd need to adjust the `--assets` argument.

If `--assets` is not specified, the game will attempt to read from `$PWD/assets` by default.

[Rust]:https://www.rust-lang.org
[Bevy]:https://bevyengine.org