use crate::router::RouteRecord;

pub fn escape_js_string(value: &str) -> String {
    let mut escaped = String::new();
    for c in value.chars() {
        match c {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\'' => escaped.push_str("\\'"),
            '`' => escaped.push_str("\\`"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\u{2028}' => escaped.push_str("\\u2028"),
            '\u{2029}' => escaped.push_str("\\u2029"),
            _ => escaped.push(c),
        }
    }
    escaped.replace("</script>", "<\\/script>")
}


pub fn generate_spa_index(app_title: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>{}</title>
    <link rel="stylesheet" href="/zoriqa.css" />
  </head>
  <body>
    <div id="zoriqa-root"></div>
    <script src="/app.js"></script>
  </body>
</html>"#,
        app_title
    )
}

pub fn generate_spa_runtime(routes: &[RouteRecord]) -> String {
    let mut routes_json = String::new();
    routes_json.push_str("[\n");
    for r in routes {
        let title_escaped = escape_js_string(r.title.as_deref().unwrap_or("Zoriqa App"));
        let html_escaped = escape_js_string(&r.html_fragment);
        routes_json.push_str(&format!(
            "  {{\n    path: \"{}\",\n    title: \"{}\",\n    html: \"{}\"\n  }},\n",
            r.route_path, title_escaped, html_escaped
        ));
    }
    routes_json.push_str("]");

    format!(
        r##"// Zoriqa SPA Router Runtime
window.__ZORIQA_ROUTES__ = {};

function matchPath(pattern, pathname) {{
  const patternSegments = pattern.split('/').filter(Boolean);
  const pathSegments = pathname.split('/').filter(Boolean);

  if (patternSegments.length !== pathSegments.length) {{
    if (pattern === "/" && pathname === "/") {{
      return {{ params: {{}} }};
    }}
    return null;
  }}

  const params = {{}};
  for (let i = 0; i < patternSegments.length; i++) {{
    const pSeg = patternSegments[i];
    const rSeg = pathSegments[i];

    if (pSeg.startsWith(':')) {{
      const paramName = pSeg.slice(1);
      params[paramName] = rSeg;
    }} else if (pSeg !== rSeg) {{
      return null;
    }}
  }}

  return {{ params }};
}}

function matchRoute(pathname) {{
  for (const route of window.__ZORIQA_ROUTES__) {{
    const match = matchPath(route.path, pathname);
    if (match) {{
      return {{
        ...route,
        params: match.params
      }};
    }}
  }}
  return null;
}}

function renderNotFound() {{
  const route404 = window.__ZORIQA_ROUTES__.find(r => r.path === "/404");
  if (route404) {{
    document.getElementById("zoriqa-root").innerHTML = route404.html;
    document.title = route404.title || "404 Not Found";
  }} else {{
    document.getElementById("zoriqa-root").innerHTML = `
      <main class="zq-center">
        <h1 class="zq-heading zq-h1 zq-bold zq-large">404</h1>
        <p class="zq-text">Page not found</p>
      </main>
    `;
    document.title = "404 Not Found";
  }}
}}

function navigate(path) {{
  const route = matchRoute(path);

  if (!route) {{
    renderNotFound();
    return;
  }}

  document.getElementById("zoriqa-root").innerHTML = route.html;
  document.title = route.title || "Zoriqa App";
}}

document.addEventListener("click", function (event) {{
  if (event.defaultPrevented) return;
  if (event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return;
  if (event.button !== 0) return;

  const link = event.target.closest("a[data-zoriqa-link]");
  if (!link) return;

  const target = link.getAttribute("target");
  if (target && target !== "_self") return;
  if (link.hasAttribute("download")) return;

  const href = link.getAttribute("href");
  if (!href || href.startsWith("http") || href.startsWith("#") || href.startsWith("mailto:") || href.startsWith("tel:")) {{
    return;
  }}

  event.preventDefault();
  history.pushState({{}}, "", href);
  navigate(href);
}});

window.addEventListener("popstate", function () {{
  navigate(location.pathname);
}});

// Initial route render
document.addEventListener("DOMContentLoaded", function () {{
  navigate(location.pathname);
}});
"##,
        routes_json
    )
}
