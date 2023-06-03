use super::settings::{Link, PageMetadata};

pub const HEADER: &str = r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />

    <meta name="description" content="{{description}}" />
  
    <link rel="icon" type="image/x-icon" href="/favicon.ico">

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
  </body>

  <footer class="container">
    <small>
      Built with <a href="https://github.com/arjunkomath/rustic-ink" target="_blank" rel="noopener noreferrer">RusticInk</a> •
      <a target="_blank" rel="noopener noreferrer" href="https://github.com/arjunkomath/rustic-ink">
        Source code
      </a>
    </small>
  </footer>
</html>
"#;
