# Minions

[![Build Status](https://travis-ci.org/blahgeek/Minions.svg?branch=master)](https://travis-ci.org/blahgeek/Minions)

Minions is a productive tool to help your daily works, e.g.

- Quickly search on google without switching to browser first
- Quickly get the translation of selected text in a web page
- Quickly launch a desktop entry
- Quickly ...

It's like a launcher, but more than that. It's simple, elegant, and yet powerful and extensive.

# Get Started

Download latest AppImage from [Release](https://github.com/blahgeek/Minions/releases) page, `chmod +x` it, then run. Or if you have rust installed, simply clone this project and compile with `cargo`.

Check [Quick Start](./doc/quickstart.md) for usage tips and **screenshots**.

Check [Action List](./doc/actions.md) to see full list of actions in Minions (and their requirements).

Check [Configuration](./doc/config.md) to customize your Minions.

Check [Plugin Documentation](./doc/plugin.md) if you want to write your own plugin (it's really easy!)

## Status

Minions is in beta, but usable. Feedback and contribution are welcome.

## Why

- I miss LaunchBar in OS X after switching to Linux
- I want to learn rust by writing some projects
- I do not want to use [cerebro](https://cerebroapp.com/) (it's un-unix-y and [it uses javascript](https://dorey.github.io/JavaScript-Equality-Table/) and [it uses electron](https://josephg.com/blog/electron-is-flash-for-the-desktop/))

## How-to build

- `cargo build --release`
- or `./build_appimage.sh`

