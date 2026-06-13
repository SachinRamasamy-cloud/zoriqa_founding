# AUIG V1 Layouts Guide

Layouts in AUIG allow you to share common structures (like navbars and footers) across multiple pages.

## Defining a Layout
Use the `layout` keyword to define a layout. Every layout must contain **exactly one** `slot` element, which is the placeholder where the page's contents will be injected.

```aui
layout Main:
  navbar "AUIG" dark

  slot

  footer "© 2026 AUIG"
```

## Using a Layout in a Page
Link a page to a layout using the `layout` keyword in the page header:

```aui
page Home layout Main:
  title "Home Page"
  
  view:
    hero "Welcome to AUIG":
      subtitle "Simple and semantic layouts."
```

## Strict Slot Rules
To prevent unexpected render errors:
1. **At Least One Slot**: If a layout doesn't contain a `slot` element, the compiler throws:
   `AUIG Error: layout "Main" must include one slot.`
2. **At Most One Slot**: If a layout contains multiple `slot` elements, the compiler throws:
   `AUIG Error: layout "Main" cannot contain multiple slots.`
