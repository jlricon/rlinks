# rlinks

[![Crates.io](https://img.shields.io/crates/v/rlinks.svg)](https://crates.io/crates/rlinks)
[![Crates.io](https://img.shields.io/crates/d/rlinks.svg)](https://crates.io/crates/rlinks)
[![license](https://img.shields.io/badge/license-GPL-blue.svg)](https://github.com/jlricon/rlinks/blob/master/LICENSE)
[![Build](https://github.com/jlricon/rlinks/workflows/Build/badge.svg)](https://github.com/jlricon/rlinks)

Rusty Links -rlinks- is a deadd links checker

NOTE: CAN ONLY BE COMPILED WITH RUST NIGHTLY!
## Usage

```
Rusty Links
Jose Luis Ricon <jose@ricon.xyz>
Finds dead links in websites

USAGE:
    Rusty Links [FLAGS] [OPTIONS] [URL]

FLAGS:
    -h, --help       Prints help information
    -s, --show_ok    Show links that are ok
    -V, --version    Prints version information

OPTIONS:
    -p, --n_par <N_PAR>              Number of parallel requests per domain [default: 4]
    -t, --timeout <timeout>          Request timeout [default: 10]
    -u, --user_agent <user_agent>    Choose your own custom user agent string [default: Mozilla/5.0 (compatible;
                                     Rlinks/0.5; +https://github.com/jlricon/rlinks/)]

ARGS:
    <URL>    URL to check links for (e.g. http://www.google.com)

```

## Benchmarks

I tested this against [this](https://nintil.com/this-review-is-not-about-reviewing-the-elephant-in-the-brain/) 
long article with over a hundred links. linkchecker was run with
 `linkchecker --no-robots -r1 --check-extern https://nintil.com/this-review-is-not-about-reviewing-the-elephant-in-the-brain/`

| Program     | Parallelism | Time    |
| ----------- | ----------- | ------- |
| rlinks      | 4 (requests per domain, default) | 6.9  |
| linkchecker | 10 (threads, default)| 14.9  |
