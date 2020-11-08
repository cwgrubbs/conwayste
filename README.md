# conwayste

Multiplayer Conway's Game of Life!

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0) ![build status](https://api.travis-ci.com/conwayste/conwayste.svg?branch=master) [![Discord](https://img.shields.io/discord/463752820026376202.svg?label=&logo=discord&logoColor=ffffff&color=7389D8&labelColor=6A7EC2)](https://discord.gg/mjSsUMw)

![life in action](https://s7.gifyu.com/images/BlaringTidyDutchsmoushond-mobile.gif)
![Patterns!](https://s8.gifyu.com/images/conwayste.gif)

## How to Play

Click on the desired menu option after the game boots. `Start Game` is a good place to... start ;).

Once in game:

* Left click toggles a cell (by default).
* The number keys control what left click does (whether it toggles a cell or drops a pattern).
* If dropping a pattern, you can use `Shift-left` and `Shift-right` to rotate the pattern.
* `Enter` to toggle chatbox focus.
* `+` and `-` to zoom in and out
* Press `r` to toggle running/paused (*Will not work in multiplayer mode*).
* `Space` to single step (*Will not work in multiplayer mode*).
* `Esc` to go back to the menu.

# Setup
Conwayste has been developed with cross-platform support in mind since day one using the Rust programming language! Your dependencies will likely vary based on your choice of operating system.

The easiest way to get the Rust compiler and toolchain is using [Rustup](https://rustup.rs/).

This has been validated as runnable on:
  * Ubuntu Linux (18.04 and 20.04)
  * Fedora Linux 32
  * Windows 10
  * OpenBSD
  * macOS Catalina (10.15.7)

There be dragons for any other operating system not listed above. Please tread carefully :smile:.

## Windows / MacOS

The Conwayste client and server compile and run right out of the box. Skip directly to [Installation](#installation).

## Linux

On Linux, the ALSA development files are required. These are provided as part of the `libasound2-dev` package on Debian and Ubuntu distributions and `alsa-lib-devel` on Fedora. For any other distribution, please refer to your package manager and/or compile them from source.

## Installation

Please clone this repository, and build the client and server using `cargo`. The build may take several minutes to complete, depending on your system specs.

```
$ git clone https://github.com/conwayste/conwayste
$ cd conwayste/
$ cargo build --bin client
$ cargo build --bin server
```

# Playing the Game

## Running The Client
```
$ cargo run --bin client
```

## Running the Server
```
$ cargo run --bin server
```

# FAQ

### Did you write your own game engine?

Nope! We are using the [`ggez`](https://github.com/ggez/ggez) engine and give many thanks to its developers and contributers. Head over to their [GitHub page](https://github.com/ggez/ggez) to learn more about it.

### When will this be ready?

The developers have busy lives and enjoy working on this in their spare time. If you are waiting for a release, then you I encourage you to contribute :smile:.

### My installation fails in Linux. What should I do?

It's likely that we have not kept the installation steps up-to-date. Please Check the Ubuntu section in `.travis.yml` for a guaranteed up-to-date list of packages if your installation fails. :)

### I found a bug! What should I do?

It would help the developers a lot if you could submit an issue in GitHub describing the bug.

## Contributors

* aaronm04
* manghi

_Your name could be here! Pull requests are welcome!_
