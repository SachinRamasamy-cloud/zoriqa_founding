use crate::router::RouteRecord;

pub fn generate_spa_index(app_title: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>{}</title>
    <link rel="stylesheet" href="/auig.css" />
  </head>
  <body>
    <div id="auig-root"></div>
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
        let title_escaped = r.title.as_deref().unwrap_or("AUIG App").replace('"', "\\\"");
        let html_escaped = r.html_fragment
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "");
        routes_json.push_str(&format!(
            "  {{\n    path: \"{}\",\n    title: \"{}\",\n    html: \"{}\"\n  }},\n",
            r.route_path, title_escaped, html_escaped
        ));
    }
    routes_json.push_str("]");

    format!(
        r##"// AUIG SPA Router Runtime
window.__AUIG_ROUTES__ = {};

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
  for (const route of window.__AUIG_ROUTES__) {{
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
  const route404 = window.__AUIG_ROUTES__.find(r => r.path === "/404");
  if (route404) {{
    document.getElementById("auig-root").innerHTML = route404.html;
    document.title = route404.title || "404 Not Found";
  }} else {{
    document.getElementById("auig-root").innerHTML = `
      <main class="aui-center">
        <h1 class="aui-heading aui-h1 aui-bold aui-large">404</h1>
        <p class="aui-text">Page not found</p>
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

  document.getElementById("auig-root").innerHTML = route.html;
  document.title = route.title || "AUIG App";
}}

document.addEventListener("click", function (event) {{
  const link = event.target.closest("a[data-auig-link]");
  if (!link) return;

  const href = link.getAttribute("href");
  if (!href || href.startsWith("http") || href.startsWith("#")) {{
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
