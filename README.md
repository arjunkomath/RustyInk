# RustyInk

[![crates.io](https://img.shields.io/crates/v/rustyink)](https://crates.io/crates/rustyink)
![Crates.io](https://img.shields.io/crates/d/rustyink)
[![Build & test](https://github.com/arjunkomath/RustyInk/actions/workflows/build_test.yml/badge.svg)](https://github.com/arjunkomath/RustyInk/actions/workflows/build_test.yml)
[![Publish to Pages](https://github.com/arjunkomath/RustyInk/actions/workflows/publish.yml/badge.svg)](https://github.com/arjunkomath/RustyInk/actions/workflows/publish.yml)

> **Note**
> I'm building this in public. You can follow the progress on Twitter [@arjunz](https://twitter.com/arjunz).

A sleek and minimalist static site generator written in Rust. Designed with simplicity in mind, RustyInk makes website creation a breeze.

Here is a live [DEMO](https://techulus.xyz), my blog is built using RustyInk.

## Features

- [x] Markdown support
- [x] Custom themes
- [x] SEO
- [x] Hot reloading
- [x] Custom metadata passthrough

## Installation

You can install RustyInk using Cargo:

```bash
cargo install rustyink
```

## Usage

### Create new project

You can initialise a new project using `new` command.

```bash
rustyink new <folder>
```

You can optionally specify a theme also.

```bash
rustyink new <folder> -t pico
```



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
title: ~/RustyInk/blog
--

### This is a blog index
```

## The `Settings.toml` file

The `Settings.toml` file contains the settings of the website, you can customize the website by changing the values in this file.

```toml
[dev]
port = 3000 # The port on which the dev server runs
ws_port = 3001 # The port on which the dev server websocket runs, for hot reloading

[site]
script_urls = [] # List of script urls to be included in the site
style_urls = [ # List of style urls to be included in the site
  'https://cdn.jsdelivr.net/npm/@picocss/pico@1/css/pico.min.css',
  'https://cdn.jsdelivr.net/npm/prismjs@1.29.0/themes/prism-tomorrow.min.css',
]

[meta]
title = "~/RustyInk" # The title of the website
description = "Blazing fast static site generator written in Rust" # The description of the website
og_image_url = "https://rustyink.cli.rs/images/og.png" # The og image url of the website
base_url = "https://rustyink.cli.rs" # The base url of the website, used for building sitemap

[navigation] # The navigation links of the website
links = [
  { label = "~/", url = "/" },
  { label = "GitHub", url = "https://github.com/arjunkomath/rustyink" },
  { label = "Twitter", url = "https://twitter.com/arjunz" },
  { label = "Blog", url = "/blog/" },
  { label = "About", url = "/about/" },
]

[data] # The data to be passed to every page, can be accessed using `data` object in every page
author = "Arjun Komath"
author_link = "https://twitter.com/arjunz"
```

## Handlebars Helpers

RustyInk provides a few handlebars helpers to make your life easier. This project uses [handlebars-rust](https://crates.io/crates/handlebars) and hence all the helpers provided by it are available. Apart from that, RustyInk provides the following helpers:

- `slice`: Slices an array and returns the sliced array.
- `sort-by`: Sorts an array of objects by a key.
- `format-date`: Formats a date using the given format.
- `stringify`: Converts a value to string, this is useful for debugging.

You can find examples of these helpers in the [demo project](https://github.com/techulus/blog).

## Deployment

You can build the site using the build command:

```bash
rustyink build <input-dir-path>
```

The build outputs are saved to `_site` folder. So, you can deploy the website by copying the `_site` folder to your web server. You can also use GitHub pages to host your website. Here is an example GitHub action to deploy your website to GitHub pages:

```yaml
# Simple workflow for deploying static content to GitHub Pages
name: Publish to Pages

on:
  # Runs on pushes targeting the default branch
  push:
    branches: ["main"]

  # Allows you to run this workflow manually from the Actions tab
  workflow_dispatch:

# Sets permissions of the GITHUB_TOKEN to allow deployment to GitHub Pages
permissions:
  contents: read
  pages: write
  id-token: write

# Allow only one concurrent deployment, skipping runs queued between the run in-progress and latest queued.
# However, do NOT cancel in-progress runs as we want to allow these production deployments to complete.
concurrency:
  group: "pages"
  cancel-in-progress: false

jobs:
  # Single deploy job since we're just deploying
  deploy:
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v3
      - name: Setup Pages
        uses: actions/configure-pages@v3
      - name: Install
        run: cargo install rustyink
      - name: Build
        run: rustyink build src # Replace src with your input directory
      - name: Upload artifact
        uses: actions/upload-pages-artifact@v1
        with:
          # Upload entire repository
          path: './_site'
      - name: Deploy to GitHub Pages
        id: deployment
        uses: actions/deploy-pages@v2
```

## LICENSE

You can find the license [here](https://github.com/arjunkomath/RustyInk/blob/main/LICENSE).