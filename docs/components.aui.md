# AUIG V1 Components Guide

AUIG provides a semantic Design Kit that translates simple component calls into robust, semantic HTML structures.

## Core Elements (Tier 1)
These are always available globally in the compiler:
- **Structure**: `view`, `section`, `row`, `column`, `box`, `list`, `item`
- **Text**: `h1`, `h2`, `h3`, `p`, `text`
- **Interactions**: `link`, `action`, `button`
- **Forms**: `form`, `input`, `textarea`, `select`
- **Tables**: `table`, `thead`, `tbody`, `tr`, `td`

## Standard Design Kit Components (Tier 2)
Import these at the top of your file using `import "auig/ui"`.

### Navbar
Represents the top navigation bar. Renders semantically as `<nav>`.
```aui
navbar "AUIG" dark
```

### Footer
Represents the bottom footer. Renders semantically as `<footer>`.
```aui
footer "© 2026 AUIG"
```

### Hero
Main showcase section. Renders semantically as `<section>`.
```aui
hero "Build Websites Faster":
  subtitle "Python-like syntax."
  action "Get Started" to "/start"
```

### Stat Card
Shows a key-value metric card. Renders semantically as `<div>` with styling.
```aui
stat-card "Users" "24.5k" success
```

### Pricing Card
Represents plan pricing details. Renders list items inside standard `<ul><li>` structures.
```aui
pricing-card "Enterprise" "$99":
  item "Unlimited projects"
  item "24/7 priority support"
```

### Alert
Feedback alerts. Renders as `<div role="alert">`.
```aui
alert "Payment successful" success
```

### Badge
Small category indicator. Renders as `<span>`.
```aui
badge "Beta" warning
```
