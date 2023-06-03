pub const HEADER: &str = r#"<!DOCTYPE html>
<html lang="en">

  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
  
    <link rel="icon" type="image/x-icon" href="/favicon.ico">
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@picocss/pico@1/css/pico.min.css">

    <title>Hello, world!</title>
  </head>

"#;

pub fn render_body(body: &str) -> String {
    format!(
        r#"  <body class="container">
    <nav>
        <a href="/">Home</a>
    </nav>

    <article id="article">
    {}
    </article>
  </body>"#,
        body
    )
}

pub const FOOTER: &str = r#"
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
