# Rusty!nk

[![crates.io](https://img.shields.io/crates/v/rustyink)](https://crates.io/crates/rustyink)
[![Build & test](https://github.com/arjunkomath/RustyInk/actions/workflows/build_test.yml/badge.svg)](https://github.com/arjunkomath/RustyInk/actions/workflows/build_test.yml)
[![Publish to Pages](https://github.com/arjunkomath/RustyInk/actions/workflows/static.yml/badge.svg)](https://github.com/arjunkomath/RustyInk/actions/workflows/static.yml)

A blazing fast static site generator in Rust

> 🚧 This project is currently under development. Expect breaking changes. 🚧

A sleek and minimalist static site generator written in Rust. Designed with simplicity in mind, RustyInk makes website creation a breeze. It supports Markdown files, allowing you to write content with ease. Despite its simplicity, RustyInk is lightning-fast and lightweight. Powered by picocss, it ensures an optimized and efficient website rendering process. With RustyInk, you can create beautiful websites that are both minimalistic and performant.

### Installation

You can install RustyInk using Cargo:

```bash
cargo install rustyink
```

### Features
- [x] Markdown support
- [x] Customizable
- [x] Syntax highlighting

### Project Structure

The following folder structure is expected by RustyInk:

```
docs/
├─ public/
│  ├─ favicon.ico
├─ pages/
│  ├─ page.md
│  ├─ about/
│  │  ├─ page.md
├─ Settings.toml
├─ global.css
```

The `docs` folder is the input directory of the project and is always specified while running dev server or building. You can specify a different input directory like this:
```bash
rustyink dev <input-dir-path>
```

- The `public` folder contains all the static assets of the website, these files are copied as-is to the output directory.
- The `pages` folder contains all the Markdown files, this is where you write your content.
- The `Settings.toml` file contains the settings of the website, you can customize the website by changing the values in this file.
- The `global.css` file contains the global CSS of the website, you can write your own CSS in this file.