use super::settings::{Link, PageMetadata};

pub const CODE_HIGHIGHTING_STYLES: &str = r#"
  <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.8.0/styles/atom-one-dark.min.css" integrity="sha512-Jk4AqjWsdSzSWCSuQTfYRIF84Rq/eV0G2+tu07byYwHcbTGfdmLrHjUSwvzp5HvbiqK4ibmNwdcG49Y5RGYPTg==" crossorigin="anonymous" referrerpolicy="no-referrer" />
"#;

pub const CODE_HIGHIGHTING_SCRIPTS: &str = r#"
  <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.8.0/highlight.min.js" integrity="sha512-rdhY3cbXURo13l/WU9VlaRyaIYeJ/KBakckXIvJNAQde8DgpOmE+eZf7ha4vdqVjTtwQt69bD2wH2LXob/LB7Q==" crossorigin="anonymous" referrerpolicy="no-referrer"></script>
  <script type="text/javascript">hljs.highlightAll();</script>
"#;

pub const HEADER: &str = r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />

    <meta name="description" content="{{description}}" />
  
    <link rel="icon" type="image/x-icon" href="/favicon.ico">

    %%CODE_HIGHIGHTING_STYLES%%

    <style>
      %%STYLES%%
    </style>

    <title>{{title}}</title>
  </head>

  <body>
    <main class="container">

    <nav>
        <ul>
          %%LINKS%%
        </ul>
    </nav>

    <hgroup>
      <h1>{{title}}</h1>
      <h2>{{description}}</h2>
    </hgroup>
"#;

pub fn render_links(links: &Vec<Link>) -> String {
    let mut nav_links = String::new();

    for link in links {
        nav_links
            .push_str(format!(r#"<li><a href="{}">{}</a></li>"#, link.url, link.label).as_str());
    }

    nav_links
}

pub fn render_article(body: &str, metadata: Option<PageMetadata>) -> String {
    if let Some(metadata) = metadata {
        format!(
            r#"<article>
            <header><h2>{}</h2></header>
    {}
            <footer><small>Author: {}</small></footer>
    </article>
    "#,
            metadata.title, body, metadata.author
        )
    } else {
        format!(
            r#"<article>
    {}
    </article>"#,
            body
        )
    }
}

pub const FOOTER: &str = r#"
    </main>
    
    %%CODE_HIGHIGHTING_SCRIPTS%%
  </body>

  <footer class="container">
    <small>
      Built with <a href="https://rustyink.techulus.xyz" target="_blank" rel="noopener noreferrer">Rusty!nk</a> •
      <a target="_blank" rel="noopener noreferrer" href="https://github.com/arjunkomath/rustyink">
        Source code
      </a>
    </small>
  </footer>
</html>
"#;
