# Rusty!nk

[![crates.io](https://img.shields.io/crates/v/rustyink)](https://crates.io/crates/rustyink)
[![Build & test](https://github.com/arjunkomath/RustyInk/actions/workflows/build_test.yml/badge.svg)](https://github.com/arjunkomath/RustyInk/actions/workflows/build_test.yml)
[![Publish to Pages](https://github.com/arjunkomath/RustyInk/actions/workflows/publish.yml/badge.svg)](https://github.com/arjunkomath/RustyInk/actions/workflows/publish.yml)

> 🚧 This project is currently under development. Expect breaking changes. 🚧

A sleek and minimalist static site generator written in Rust. Designed with simplicity in mind, RustyInk makes website creation a breeze.

### Installation

You can install RustyInk using Cargo:

```bash
cargo install rustyink
```

### Create new project

You can initialise a new project using `new` command.

```bash
rustyink new <folder>
```

You can optionally specify a theme also.

```bash
rustyink new <folder> -t pico
```

### Features

- [x] Markdown support
- [x] Custom themes
- [x] SEO
- [x] Custom metadata passthrough

### Project Structure

The following folder structure is expected by RustyInk:

```
docs/
├─ public/
├─ pages/
│  ├─ page.md
│  ├─ path/
│  │  ├─ page.md
│  │  ├─ custom-url.md
├─ theme/
│  ├─ global.css
│  ├─ app.hbs
│  ├─ custom-template.hbs
├─ Settings.toml
```

The `docs` folder is the input directory of the project and is always specified while running dev server or building. You can specify a different input directory like this:

```bash
rustyink dev <input-dir-path>
```

- The `Settings.toml` file contains the settings of the website, you can customize the website by changing the values in this file.
- The `public` folder contains all the static assets of the website, these files are copied as-is to the output directory.
- The `pages` folder contains all the Markdown files, this is where you write your content.
- The `theme` folder contains all site templates and styles. It is written using [handlebars](https://handlebarsjs.com/guide/) syntax.
- The `global.css` file contains the global CSS of the website, you can write your own CSS in this file.

### Building custom pages

A great example would be a blog index page where you show a list of posts and link to them. This can be achieved by accessing the site directory that is passed to every page.
The site directory can be accessed through the root object, this is available in every page and it represents the entire site structure including its metadata, so I can render a blog index page like this:

A custom template say `blog`, with lists all pages under `blog` folder.

```handlebars
<ul>
  {{#each root.blog}}
    {{#if (not (eq @key "_self"))}}
      <hgroup>
        <h4><a href="{{@key}}/">{{this.title}}</a></h4>
        <h2>{{this.author}}</h2>
      </hgroup>
    {{/if}}
  {{/each}}
</ul>
```

Then define a new page under blog folder and specify the template as `blog` which we have created as shown above.

```md
--
template: blog
title: ~/Rusty!nk/blog
--

### This is a blog index
```
