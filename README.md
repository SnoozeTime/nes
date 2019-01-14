# NES emulator in Rust (WIP)

My NES emulator written in Rust :') It has very basic functionalities as
it is a *learning project*. It supports the most basic mappers (NROM, UxRom, MMC1
and MMC3) and there is no sound yet!

![The Legend of Zelda](doc/zelda.png?raw=true "Zelda")
![Kirby](doc/kirby.png?raw=true "Kirby")
![Catlevania](doc/castlevania.png?raw=true "Castlevania")
![Super Mario Bros 3](doc/mario.png?raw=true "Mario")
![Ninja Gaiden 3](doc/ninja.png?raw=true "Ninja Gaiden")

Games that are playable:
- Donkey Kong,
- Mario Bros
- Ballon Fight
- Super Mario Bros
- Castlevania
- Metroid
- Contra
- Megaman 2
- Final Fantasy 1
- The Legend of Zelda
- Super Mario Bros 3
- Ninja Gaiden 3
- and most likely a bunch of others

## Remaining

There are still some bugs to iron. For example, sprite 0 hit detection
is not optimal. Some games are not running (Ninja Gaiden). Also, there
are a few slow downs when playing Super Mario Bros 3 so I need to improve
PPU and SDL code a bit.

APU is not done yet.

## Future plans

- RaspberryPi integration
- WASM?

