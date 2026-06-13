# AUIG V1 Routing Guide

AUIG compiles a folder structure into clean, statically routeable routes.

## Static Routes
The folder structure inside `pages/` maps directly to static routes.

- `pages/index.aui` $\rightarrow$ `/` (compiles to `dist/index.html`)
- `pages/about.aui` $\rightarrow$ `/about` (compiles to `dist/about/index.html`)
- `pages/dashboard.aui` $\rightarrow$ `/dashboard` (compiles to `dist/dashboard/index.html`)

## Dynamic Route Templates
AUIG supports dynamic route directory naming (e.g. `[id]`). 

- `pages/users/[id].aui` $\rightarrow$ `/users/123` (compiles to `dist/users/[id]/index.html`)

> [!NOTE]
> AUIG is a static compiler. Dynamic routes serve as **static template fallbacks** for client-side routing and dev server matches; they are not dynamically data-driven on the server.

## Dev Server Routing
When running `auig dev`, the dev server automatically matches requested dynamic paths (like `/users/123`) to their static template fallbacks (`dist/users/[id]/index.html`) to facilitate frontend development.
