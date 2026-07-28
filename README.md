# RustyInk

![Art by DALL·E](https://rustyink.techulus.xyz/images/og.png)

[![crates.io](https://img.shields.io/crates/v/rustyink)](https://crates.io/crates/rustyink)
![Crates.io](https://img.shields.io/crates/d/rustyink)
[![Build & test](https://github.com/arjunkomath/RustyInk/actions/workflows/build_test.yml/badge.svg)](https://github.com/arjunkomath/RustyInk/actions/workflows/build_test.yml)
[![Publish to Pages](https://github.com/arjunkomath/RustyInk/actions/workflows/publish.yml/badge.svg)](https://github.com/arjunkomath/RustyInk/actions/workflows/publish.yml)
[![justforfunnoreally.dev badge](https://img.shields.io/badge/justforfunnoreally-dev-9ff)](https://justforfunnoreally.dev)

A sleek and minimalist static site generator written in Rust. Designed with simplicity in mind, RustyInk makes website creation a breeze.

Here is a live [DEMO](https://techulus.xyz), my blog is built using RustyInk.

If you've any questions, find me on X [@arjunz](https://x.com/arjunz).

## Features

- Markdown and custom themes
- OpenGraph, Twitter Cards, and sitemaps
- AMP support
- Hot reloading
- Custom metadata passthrough

## Installation

```bash
brew install arjunkomath/tap/rustyink
# or
cargo install rustyink
```

## Quick start

```bash
rustyink new my-site
rustyink dev my-site --watch
rustyink build my-site
```

See the [documentation](https://rustyink.techulus.xyz) for project structure, configuration, templates, helpers, and deployment.

## License

[MIT](LICENSE)
