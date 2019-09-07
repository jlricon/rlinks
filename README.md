# rlinks

[![Crates.io](https://img.shields.io/crates/v/rlinks.svg)](https://crates.io/crates/rlinks)
[![Crates.io](https://img.shields.io/crates/d/rlinks.svg)](https://crates.io/crates/rlinks)
[![license](https://img.shields.io/badge/license-GPL-blue.svg)](https://github.com/jlricon/rlinks/blob/master/LICENSE)
[![Build](https://github.com/jlricon/rlinks/workflows/Build/badge.svg)](https://github.com/jlricon/rlinks)
[![Snap Status](https://build.snapcraft.io/badge/jlricon/rlinks.svg)](https://build.snapcraft.io/user/jlricon/rlinks)

Rusty Links -rlinks- is a dead links checker

NOTE: CAN ONLY BE COMPILED WITH RUST NIGHTLY!
## Usage

```
Rusty Links 0.6.0
Jose Luis Ricon <jose@ricon.xyz>
RLinks finds dead links in websites, or dumps scraped links to a file

USAGE:
    rlinks <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    check    Checks links
    dump     Dump links
    help     Prints this message or the help of the given subcommand(s)

```
## Install

### Cargo
```
cargo install rlinks
```
## Benchmarks

I tested this against [this](https://nintil.com/this-review-is-not-about-reviewing-the-elephant-in-the-brain/) 
long article with over a hundred links. linkchecker was run with
 `linkchecker --no-robots -r1 --check-extern https://nintil.com/this-review-is-not-about-reviewing-the-elephant-in-the-brain/`

| Program     | Parallelism | Time    |
| ----------- | ----------- | ------- |
| rlinks      | 2 (requests per domain, default) | 4  |
| linkchecker | 10 (threads, default)| 14.9  |
