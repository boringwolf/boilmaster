# Mark XIV Thermocoil Boilmaster

Web service for Final Fantasy XIV game data and asset discovery.

## Installation

### Building From Source

**Requirements**

<!-- NOTE: See /rust-toolchain.toml when updating. -->

- [Rust](https://www.rust-lang.org/tools/install) >= 1.85.0

```bash
git clone https://github.com/ackwell/boilmaster
cd boilmaster
cargo run --release
```

### Docker Usage

boilmaster is published as a Docker image on the github container registery. An example `docker-compose.yml` such as the below can be used to bring the service online.

```yml
services:
  boilmaster:
    image: ghcr.io/ackwell/boilmaster:latest
    container_name: boilmaster
    environment:
      # Other configuration here, see the Configuration section below for more information.
    volumes:
      - ${PWD}/persist:/app/persist
    ports:
      - 8080:8080
    restart: unless-stopped
```

## Configuration

The default configuration for boilmaster can be found in `boilmaster.toml`. This file can be considered a source of truth for all configuration options available.

In addition to the configuration file, all options may also be set via environment variables. The name of these variables is the same as their path in TOML; replacing `.` with `_`, in uppercase, with the prefix `BM_`. i.e. the config file key `http.api1.sheet.limit.default` can be set with the environment variable `BM_HTTP_API1_SHEET_LIMIT_DEFAULT`.

Configuration is only read during application startup, a restart is required if changes are made.

Before exposing the service to the public, it is strongly advised to change the `http.admin.auth.username` and `http.admin.auth.password` values.

## Differences compared to original version

- Use external data files instead of tracking all patches of the game. Following files should be present in `game` directory
  (or mounted to `/app/game` when using docker):
  - ffxivgame.ver
  - sqpack/ffxiv/0a0000.win32.dat
  - sqpack/ffxiv/0a0000.win32.index
  - sqpack/ffxiv/0a0000.win32.index2
- All version related functionalities are disabled. The version param will not be handled.

### Switching supported languages

Multi-language queries could be achieved with `exd build` command of [ixion](https://github.com/thewakingsands/ixion), which could generate a merged sqpack file from different servers. Set environment variable `BM_READ_LANGUAGE_EXCLUDE` for different setups. For example:

- Global: `[chs,cht,kr]`
- SDO: `[ja,en,de,fr,cht,kr]`
- Combination of Global and SDO: `[cht,kr]`

Test language support with following path:

```
/api/1/sheet/Item/1?fields=Name@lang(chs),Name@lang(de),Name@lang(en),Name@lang(fr),Name@lang(ja)
```
