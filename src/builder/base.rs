use super::settings::Link;

pub const HEADER: &str = r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
  
    <link rel="icon" type="image/x-icon" href="/favicon.ico">
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@picocss/pico@1/css/pico.min.css">

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
    let mut nav_links = String::from(format!(r#"<li><a href="{}">{}</a></li>"#, "/", "Home"));

    for link in links {
        nav_links
            .push_str(format!(r#"<li><a href="{}">{}</a></li>"#, link.url, link.label).as_str());
    }

    nav_links
}

pub fn render_article(body: &str) -> String {
    format!(
        r#"<article>
        <header>Page title</header>
    {}
    </article>"#,
        body
    )
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
