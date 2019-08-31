# rlinks

[![Crates.io](https://img.shields.io/crates/v/rlinks.svg)](https://crates.io/crates/rlinks) [![Crates.io](https://img.shields.io/crates/d/rlinks.svg)](https://crates.io/crates/rlinks) [![license](https://img.shields.io/badge/license-GPL-blue.svg)](https://github.com/jlricon/rlinks/blob/master/LICENSE)![https://travis-ci.com/jlricon/rlinks](https://travis-ci.com/jlricon/rlinks.svg?branch=master)

Rusty Links -rlinks- is a dead dlink checker

## Usage

```
Rusty Links 0.5.0
Jose Luis Ricon <jose@ricon.xyz>
Finds dead links in websites

USAGE:
    Rusty Links [FLAGS] [OPTIONS] [URL]

FLAGS:
    -h, --help       Prints help information
    -s, --show_ok    Show links that are ok
    -V, --version    Prints version information

OPTIONS:
    -p, --n_par <N_PAR>    Number of parallel requests (Default 100)

ARGS:
    <URL>    URL to check links for (e.g. http://www.google.com)

```

## Benchmarks

I tested this against [this](https://nintil.com/this-review-is-not-about-reviewing-the-elephant-in-the-brain/) long article with over a hundred links. linkchecker was run with `linkchecker --no-robots -r1 --check-extern https://nintil.com/this-review-is-not-about-reviewing-the-elephant-in-the-brain/`

| Program     | Parallelism | Time    |
| ----------- | ----------- | ------- |
| rlinks      | 10          | 9.22 s  |
| rlinks      | 100         | 3.976 s |
| linkchecker | 10          | 17.723  |
| linkchecker | 100         | 16.249  |
