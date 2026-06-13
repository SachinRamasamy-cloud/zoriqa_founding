# AUIG V1 Unified Master Documentation

Welcome to the AUIG V1 Unified Master Documentation. This guide provides an end-to-end explanation of the AUIG static layout compiler architecture, core syntax, layouts, routing mechanics, Tier 1 and Tier 2 components, and its advanced JIT utility-based styling system.

---

## 1. Core Architecture & Pipeline

### Compiler Pipeline

AUIG V1 uses an AST-driven compiler architecture.

```
.aui Page Source
→ Lexer
→ Parser
→ Typed AST
→ Import Resolver
→ Theme Resolver
→ Layout Slot Resolver
→ Design Kit AST Transformer
→ Schema Validator
→ HTML Generator
→ JIT CSS Collector
→ CSS Generator
→ dist/
```

No Tier 2 component is expanded through raw string templates. Components such as `navbar`, `hero`, `stat-card`, and `pricing-card` are parsed into AST nodes and transformed structurally into lower-level semantic AST nodes before HTML generation by the **Design Kit AST Transformer**.

---

## 2. Syntax, Layouts & Routing

### Syntax Guidelines
AUIG features a Python-like indentation-sensitive syntax that is easy for beginners yet highly structured.
* **Easy-by-Default Elements**: Element names followed by a positional string literal argument.
  ```aui
  h1 "Build Websites Faster"
  ```
* **Semantic Flags**: Represent tone or style intent. Place positional arguments first, then flags.
  ```aui
  btn "Get Started" primary solid
  ```
* **Named Properties**: Used for behavioral elements (links, sources, placeholders, roles).
  ```aui
  link "Documentation" to "/docs"
  ```
* **Nested Children**: Defined using indented blocks (ending with `:`) or curly braces (`{ }`).
  ```aui
  column gap-medium:
    h2 "Welcome"
    p "This is nested child content."
  ```

### Theme Block Syntax
Themes are defined globally using the `theme` directive, either unnamed or named. Variables are key-value mappings of design tokens:
```aui
theme:
  primary blue
  success green
  danger red
  radius xl
```
Or with a named theme:
```aui
theme "startup":
  primary blue
  success green
  radius xl
```
> [!NOTE]
> **Theme Activation**: If only one theme is declared, AUIG uses it globally. Named themes are reserved for future multi-theme selection unless explicitly activated.

### Layout Slot Mechanics
Layouts allow you to define persistent templates (e.g., headers, footers) shared across multiple pages.
* **Defining a Layout**: Declared using the `layout` keyword. A layout must contain **exactly one** `slot` element.
  ```aui
  layout Main:
    navbar "AUIG" dark
    slot
    footer "© 2026 AUIG Project"
  ```
* **Page Layout Linking**: Pages specify their layout in the page declaration.
  ```aui
  page Home layout Main:
    title "AUIG - Welcome"
    view:
      hero "Build Websites Faster" primary
  ```
* **Strict Slot Rules**:
  * **At Least One Slot**: Failing to include a `slot` throws `AUIG Error: layout "LayoutName" must include one slot.`
  * **At Most One Slot**: Including multiple slots throws `AUIG Error: layout "LayoutName" cannot contain multiple slots.`

### Statically Routeable Directories
The pages directory structure maps directly to static routes in the output:
* `pages/index.aui` $\rightarrow$ `/` (compiles to `dist/index.html`)
* `pages/about.aui` $\rightarrow$ `/about` (compiles to `dist/about/index.html`)
* `pages/users/[id].aui` $\rightarrow$ `/users/[id]` (compiles to `dist/users/[id]/index.html`)

> [!IMPORTANT]
> **Dynamic Routing Fallbacks**: Dynamic route files are compiled as static fallback templates. During development, `/users/123` can be matched to `/users/[id]/index.html`. No automatic data loading or route params are available yet.

---

## 3. Tier 1: Core Layout & Content Elements

Tier 1 elements are primitive tags built directly into the compiler. They map directly to standard semantic HTML tags and can take JIT utility flags.

### Elements Reference Table

| AUI Tag | Compiled HTML Tag | Default Behavior / CSS Base Rules |
| :--- | :--- | :--- |
| `view` | `<main>` | Full viewport height: `min-height: 100vh; display: flex; flex-direction: column;` |
| `section` | `<section>` | Margin and vertical spacing. |
| `row` | `<div>` | Horizontal flex-box: `display: flex; flex-direction: row; flex-wrap: wrap;` |
| `column` / `col` | `<div>` | Vertical flex-box: `display: flex; flex-direction: column;` |
| `center` | `<div>` | Centered flex layout: `display: flex; flex-direction: column; align-items: center; justify-content: center; text-align: center;` |
| `box` | `<div>` | standard block wrapper: `display: block;` |
| `card` | `<div>` | Box container with border radius and custom class `.aui-card`. |
| `heading` / `h1` | `<h1>` | Title text size `64px` (`.aui-large`) or `38px` on mobile. |
| `h2` | `<h2>` | Subtitle text size `36px` (`.aui-h2`) or `28px` on mobile. |
| `h3` | `<h3>` | Small header. |
| `text` / `p` | `<p>` | Paragraph text: `font-size: 16px; line-height: 1.7;` |
| `subtitle` | `<p>` | Styled subtitle text. |
| `desc` | `<p>` | Card descriptions. |
| `message` | `<p>` | Feedback messages. |
| `icon` | `<span>` | inline-block icon wrapper: `.aui-icon { display: inline-block; }`. Bypasses JIT utility checks. |
| `btn` / `button` | `<button>` (or `<a>` if `to` is present) | Interactive button with hover translation and scale transitions. |
| `link` / `a` | `<a>` | Hyperlink with hover state color adjustments. |
| `span` | `<span>` | Generic inline container. |
| `list` | `<ul>` | Unordered list wrapper. |
| `item` | `<li>` | Nested list item. |
| `slot` | Renders children | Layout placeholder. Bypasses JIT utility checks. |
| `form` | `<form>` | Standard form wrapper. |
| `input` | `<input>` | Styled textual input fields. |
| `textarea` | `<textarea>` | Multi-line textual input fields. |
| `select` | `<select>` | Dropdown selection field. |
| `table` | `<table>` | Standard layout table. |
| `thead` | `<thead>` | Table head container. |
| `tbody` | `<tbody>` | Table body container. |
| `tr` | `<tr>` | Table row container. |
| `td` | `<td>` | Table cell element. |

> [!NOTE]
> Some Tier 2 components generate internal semantic HTML tags (such as `<aside>`, `<dialog>`, `<ol>`, or `<figure>`) that are not directly exposed as Tier 1 user-facing AUIG tags.

---

## 4. Tier 2: Built-in Design Kit Components

Tier 2 components are high-level design presets compiled down into semantic Tier 1 primitive structures. To use them, import `auig/ui` or their specific design kit module (e.g., `import "auig/navbar"`).

### Design Kit Registry Schema

| Component Name | Required Positionals | Allowed Tones | Allowed Variants | Allowed Children | Example Usage |
| :--- | :---: | :---: | :---: | :--- | :--- |
| `navbar` | 1 | Yes | Yes | `link`, `action`, `button` | `navbar "My Brand" dark` |
| `footer` | 1 | Yes | Yes | `row`, `column`, `link`, `p` | `footer "© 2026 Corp" dark` |
| `hero` | 1 | Yes | Yes | `subtitle`, `action`, `button`, `badge`, `style` | `hero "Welcome Title" primary` |
| `stat-card` | 2 | Yes | Yes | `badge`, `style` | `stat-card "Users" "24k" success` |
| `feature-card` | 1 | Yes | Yes | `icon`, `desc`, `badge`, `style` | `feature-card "Speeds" success` |
| `profile-card` | 2 | Yes | Yes | None | `profile-card "Jane" "Lead Designer"` |
| `pricing-card` | 2 | Yes | Yes | `desc`, `action`, `button`, `item`, `style` | `pricing-card "Basic" "$19"` |
| `alert` | 1 | Yes | Yes | `message`, `style` | `alert "Disk alert" danger` |
| `badge` | 1 | Yes | Yes | None | `badge "Beta" warning` |
| `sidebar` | 0 | Yes | Yes | `link`, `style` | `sidebar:` |
| `tabs` | 0 | Yes | Yes | `button`, `style` | `tabs:` |
| `modal` | 1 | Yes | Yes | `h1`, `h2`, `h3`, `p`, `row`, `column`, `button`, `action`, `form`, `input`, `style` | `modal "Are you sure?"` |
| `timeline` | 0 | Yes | Yes | `item` | `timeline:` |
| `faq` | 0 | Yes | Yes | `h3`, `p` | `faq:` |
| `testimonial` | 2 | Yes | Yes | None | `testimonial "Great!" "Alice"` |
| `gallery` | 0 | Yes | Yes | `image` | `gallery:` |
| `dashboard-card` | 2 | Yes | Yes | None | `dashboard-card "Views" "12k"` |

---

### Component HTML Output Structure Reference

#### 1. Navbar (`navbar`)
* **Default values**: Background `gray-950`, Text `white`, Border `gray-800`.
* **Compiled HTML Output**:
```html
<nav class="aui-navbar aui-bg-gray-950 aui-text-white aui-p-4 aui-flex aui-flex-row aui-items-center aui-justify-between aui-w-full">
  <h2 class="aui-heading aui-h2 aui-bold aui-text-inherit">AUIG</h2>
  <div class="aui-row aui-gap-medium aui-items-center">
    <a href="/docs" class="aui-link aui-text-inherit">Docs</a>
    <a href="/start" class="aui-button aui-secondary">Get Started</a>
  </div>
</nav>
```

#### 2. Footer (`footer`)
* **Default values**: Background `gray-950`, Text `white`, Border `gray-800`.
* **Compiled HTML Output**:
```html
<footer class="aui-footer aui-bg-gray-950 aui-text-white aui-border-t aui-border-gray-800 aui-w-full">
  <div class="aui-section aui-py-12 aui-px-8">
    <!-- Nested child links & rows -->
    <div class="aui-row aui-justify-between aui-items-center aui-border-t aui-border-gray-800 aui-pt-8 aui-mt-8">
      <p class="aui-text aui-muted aui-small">© 2026 AUIG Project</p>
    </div>
  </div>
</footer>
```

#### 3. Hero (`hero`)
* **Default values**: Background `blue-600`, Text `white`.
* **Compiled HTML Output**:
```html
<section class="aui-section aui-bg-blue-600 aui-text-white aui-py-20 aui-px-8 aui-rounded-2xl aui-my-8">
  <div class="aui-center">
    <h1 class="aui-heading aui-h1 aui-bold aui-large">Main Showcase</h1>
    <p class="aui-text aui-medium aui-text-inherit aui-opacity-90">Subtitle text goes here</p>
    <div class="aui-row aui-gap-medium aui-mt-6 aui-justify-center aui-items-center">
      <a href="/docs" class="aui-button aui-secondary">Read Docs</a>
    </div>
  </div>
</section>
```

#### 4. Stat Card (`stat-card`)
* **Default values**: Background `white`, Text `gray-900`, Border `gray-200`, Shadow `shadow-sm`, Radius `xl`.
* **Compiled HTML Output**:
```html
<div class="aui-card aui-bg-white aui-text-gray-900 aui-border aui-border-gray-200 aui-shadow-sm aui-rounded-xl aui-p-6 aui-text-center aui-w-full">
  <p class="aui-text aui-small aui-muted aui-mb-2">Total Users</p>
  <h2 class="aui-heading aui-h2 aui-bold aui-large">24.5k</h2>
</div>
```

#### 5. Feature Card (`feature-card`)
* **Default values**: Background `white`, Text `gray-900`, Border `gray-100`, Shadow `shadow-md`, Radius `2xl`.
* **Compiled HTML Output**:
```html
<div class="aui-card aui-bg-white aui-text-gray-900 aui-border aui-border-gray-100 aui-shadow-md aui-rounded-2xl aui-p-6 aui-text-center aui-flex aui-flex-col aui-gap-small">
  <span class="aui-icon aui-p-3 aui-rounded-full aui-bg-blue-50 aui-text-blue-600 aui-inline-block aui-w-12 aui-h-12">zap</span>
  <h2 class="aui-heading aui-h2 aui-bold aui-medium aui-mb-2">Fast Speeds</h2>
  <p class="aui-text aui-small aui-muted">Compiles in less than a millisecond.</p>
</div>
```

#### 6. Profile Card (`profile-card`)
* **Default values**: Background `white`, Text `gray-900`, Border `gray-200`, Shadow `shadow-md`, Radius `2xl`.
* **Compiled HTML Output**:
```html
<div class="aui-box aui-bg-white aui-text-gray-900 aui-rounded-2xl aui-p-6 aui-text-center aui-flex aui-flex-col aui-gap-small aui-border aui-border-gray-200 aui-shadow-md">
  <h3 class="aui-heading aui-h3 aui-bold">Alice Smith</h3>
  <p class="aui-text aui-muted aui-small">Lead Engineer</p>
</div>
```

#### 7. Pricing Card (`pricing-card`)
* **Default values**: Background `white`, Text `gray-900`, Border `gray-200`, Shadow `shadow-lg`, Radius `2xl`.
* **Compiled HTML Output**:
```html
<div class="aui-card aui-bg-white aui-text-gray-900 aui-border aui-border-gray-200 aui-shadow-lg aui-rounded-2xl aui-p-8 aui-flex aui-flex-col aui-gap-large aui-relative">
  <!-- If 'popular' flag is active, adds scale-105 transform and label badge -->
  <div class="aui-column aui-gap-small">
    <h2 class="aui-heading aui-h2 aui-bold aui-medium">Enterprise</h2>
    <p class="aui-text aui-muted aui-small">For large organizations</p>
  </div>
  <div class="aui-row aui-items-baseline aui-gap-small">
    <span class="aui-text aui-large aui-bold">$99</span>
    <span class="aui-text aui-muted aui-small">/month</span>
  </div>
  <ul class="aui-list aui-flex aui-flex-col aui-gap-small">
    <li class="aui-item aui-text aui-small aui-flex aui-flex-row aui-items-center aui-gap-small">✔ Unlimited projects</li>
  </ul>
  <a href="#" class="aui-button aui-primary aui-w-full aui-text-center">Select Plan</a>
</div>
```

#### 8. Alert (`alert`)
* **Default values**: Background `blue-50`, Text `blue-800`, Border `blue-200`, Radius `xl`.
* **Compiled HTML Output**:
```html
<div role="alert" class="aui-row aui-bg-blue-50 aui-text-blue-800 aui-border aui-border-blue-200 aui-rounded-xl aui-p-4 aui-items-center aui-gap-medium aui-w-full">
  <div class="aui-column aui-gap-small">
    <h2 class="aui-heading aui-h2 aui-bold aui-small">Operation Succeeded</h2>
    <p class="aui-text aui-small aui-text-inherit">Changes compiled successfully.</p>
  </div>
</div>
```

#### 9. Badge (`badge`)
* **Default values**: Background `gray-100`, Text `gray-800`, Radius `full`.
* **Compiled HTML Output**:
```html
<span class="aui-span aui-bg-green-100 aui-text-green-800 aui-rounded-full aui-px-3 aui-py-1 aui-text-xs aui-bold aui-inline-block aui-w-auto">
  <span class="aui-span aui-text-inherit">Success Badge</span>
</span>
```

#### 10. Sidebar (`sidebar`)
* **Default values**: Background `gray-950`, Text `white`, Border `gray-800`.
* **Compiled HTML Output**:
```html
<aside class="aui-aside aui-bg-gray-950 aui-text-white aui-w-64 aui-h-full aui-p-6 aui-flex aui-flex-col aui-gap-medium">
  <!-- Sidebar inner navigation links -->
</aside>
```

#### 11. Tabs (`tabs`)
* **Default values**: Background `transparent`, Text `gray-900`, Border `gray-200`, Radius `md`.
* **Compiled HTML Output**:
```html
<div class="aui-box aui-bg-transparent aui-text-gray-900 aui-flex aui-flex-row aui-border-b aui-gap-small aui-p-2">
  <!-- Tab trigger elements -->
</div>
```

#### 12. Modal (`modal`)
* **Default values**: Background `white`, Text `gray-900`, Border `transparent`, Shadow `shadow-xl`, Radius `2xl`.
* **Compiled HTML Output**:
```html
<dialog class="aui-dialog aui-bg-white aui-text-gray-900 aui-rounded-2xl aui-p-8 aui-max-w-lg aui-mx-auto aui-my-20 aui-flex aui-flex-col aui-gap-medium aui-shadow-xl">
  <h3 class="aui-heading aui-h3 aui-bold">Confirm Action</h3>
  <!-- Modal overlay/dialog children contents -->
</dialog>
```

#### 13. Timeline (`timeline`)
* **Default values**: Background `transparent`, Text `gray-900`.
* **Compiled HTML Output**:
```html
<ol class="aui-ol aui-bg-transparent aui-text-gray-900 aui-border-l aui-border-gray-200 aui-ml-4 aui-pl-6 aui-flex aui-flex-col aui-gap-large">
  <!-- List items representing timeline checkpoints -->
</ol>
```

#### 14. FAQ (`faq`)
* **Default values**: Background `transparent`, Text `gray-900`.
* **Compiled HTML Output**:
```html
<div class="aui-box aui-bg-transparent aui-text-gray-900 aui-flex aui-flex-col aui-gap-medium aui-w-full">
  <!-- FAQ question headers and response panels -->
</div>
```

#### 15. Testimonial (`testimonial`)
* **Default values**: Background `gray-50`, Text `gray-900`, Border `gray-200`, Shadow `shadow-sm`, Radius `xl`.
* **Compiled HTML Output**:
```html
<figure class="aui-figure aui-bg-gray-50 aui-text-gray-900 aui-rounded-xl aui-p-6 aui-italic aui-flex aui-flex-col aui-gap-small aui-border aui-border-gray-200 aui-shadow-sm">
  <p class="aui-text aui-text-inherit">"AUIG is incredibly fast!"</p>
  <p class="aui-text aui-bold aui-small aui-muted">— Jane Doe</p>
</figure>
```

#### 16. Gallery (`gallery`)
* **Default values**: Background `transparent`, Text `gray-900`.
* **Compiled HTML Output**:
```html
<div class="aui-box aui-bg-transparent aui-text-gray-900 aui-grid aui-grid-cols-3 aui-gap-medium aui-w-full">
  <!-- Nested structural grid imagery -->
</div>
```

#### 17. Dashboard Card (`dashboard-card`)
* **Default values**: Background `white`, Text `gray-900`, Border `gray-200`, Shadow `shadow-sm`, Radius `xl`.
* **Compiled HTML Output**:
```html
<div class="aui-box aui-bg-white aui-text-gray-900 aui-border aui-border-gray-200 aui-shadow-sm aui-rounded-xl aui-p-6 aui-flex aui-flex-col aui-gap-small">
  <p class="aui-text aui-muted aui-small">Monthly Revenue</p>
  <h2 class="aui-heading aui-bold">$48,000</h2>
</div>
```

---

## 5. Styling System & Utility Reference

### Style Resolution Precedence Execution Flow
When calculating styled outputs, the engine merges rules sequentially. Each phase applies styling properties over the previous one:
```
Default Component Presets
  → Theme presets (Global theme tokens)
  → Tone presets (e.g. primary, danger background & text colors)
  → Variant modifiers (e.g. soft, solid, outline adjustments)
  → Style Override Block (Custom nesting keys)
```

> [!NOTE]
> **Legacy Inline Properties**: Inline component properties (e.g. `bg "red-500"`) are still accepted for backward compatibility, but new AUIG code should use nested `style:` blocks.

---

### Tone Preset Mappings
Tone flags map to specific background, text, and border classes:

| Tone Flag | Background Class (`bg`) | Text Color Class (`text`) | Border Color Class (`border`) | Raw Hex Value Mappings |
| :--- | :--- | :--- | :--- | :--- |
| `primary` | `blue-600` | `white` | `blue-700` | `#2563eb` / `#ffffff` / `#1d4ed8` |
| `success` | `green-50` | `green-900` | `green-200` | `#f0fdf4` / `#14532d` / `#bbf7d0` |
| `warning` | `yellow-50` | `yellow-800` | `yellow-200` | `#fefce8` / `#854d0e` / `#fef08a` |
| `danger` / `error`| `red-50` | `red-800` | `red-200` | `#fef2f2` / `#991b1b` / `#fecaca` |
| `info` | `blue-50` | `blue-800` | `blue-200` | `#eff6ff` / `#1e40af` / `#bfdbfe` |
| `dark` | `gray-950` | `white` | `gray-800` | `#030712` / `#ffffff` / `#1f2937` |
| `light` / `neutral`| `gray-50` | `gray-800` | `gray-200` | `#f9fafb` / `#1f2937` / `#e5e7eb` |

---

### Variant Shapes
Variants modify shapes, backgrounds, borders, or text weights on top of the resolved tone:
* `solid`: Background becomes solid saturated color, text becomes white.
* `soft`: Background becomes light theme hue, text color becomes dark saturated theme color.
* `outline`: Background becomes transparent, borders and text apply saturated theme colors.
* `dark`: Forces background to `gray-950`, text to `white`, and borders to `gray-800`.
* `light`: Forces background to `gray-50`, text to `gray-800`, and borders to `gray-200`.
* `minimal`: Background and borders become transparent, text retains theme color.

---

### Style Override Block (`style:`)
The `style:` block provides customized layout styling. **It must be nested inside components/elements.**

#### Valid Styling Keys
The compiler enforces a strict list of allowed style keys:
* `bg` (Background color class, e.g. `blue-600` or `white`)
* `text` (Text color class, e.g. `gray-900`)
* `border` (Border color class, e.g. `gray-200`)
* `radius` (Border radius size: `sm`, `md`, `lg`, `xl`, `2xl`, `full`)
* `shadow` (Box shadow weight: `sm`, `md`, `lg`, `none`)
* `padding` / `margin` (Spacing rules)
* `width` / `height` (Dimensions)
* `align` / `justify` (Flex parameters)

#### Recommended Syntax
```aui
stat-card "Revenue" "$48k" success:
  style:
    bg green-100
    text green-900
    border green-300
    radius xl
    shadow md
```

---

### JIT Utility Vocabulary Reference

Any flag applied to an element/component (or compiled as a helper class) must belong to the following strict vocabulary.

#### 1. Color Utilities (`bg-`, `text-`, `border-`)
* **Colors**: `gray`, `blue`, `red`, `green`, `yellow`, `indigo`, `purple`.
* **Shades**: `50`, `100`, `150`, `200`, `300`, `400`, `500`, `600`, `700`, `800`, `900`, `950`.
* **Special Cases**: `bg-white`, `text-white`, `border-white`, `bg-black`, `text-black`, `border-black`, `bg-transparent`, `text-inherit`.

#### 2. Spacing Utilities (`p-`, `px-`, `py-`, `pt-`, `pb-`, `m-`, `mx-`, `my-`, `mt-`, `mb-`, `ml-`, `mr-`)
* **Supported Spacing Values**:
  * `0` $\rightarrow$ `0px`
  * `1` $\rightarrow$ `4px`
  * `2` $\rightarrow$ `8px`
  * `3` $\rightarrow$ `12px`
  * `4` $\rightarrow$ `16px`
  * `6` $\rightarrow$ `24px`
  * `8` $\rightarrow$ `32px`
  * `12` $\rightarrow$ `48px`
  * `16` $\rightarrow$ `64px`
  * `20` $\rightarrow$ `80px`

#### 3. Sizing & Dimension Utilities
* `w-full` $\rightarrow$ `width: 100%;`
* `w-auto` $\rightarrow$ `width: auto;`
* `h-full` $\rightarrow$ `height: 100%;`
* `h-auto` $\rightarrow$ `height: auto;`
* `w-12` $\rightarrow$ `width: 48px;`
* `h-12` $\rightarrow$ `height: 48px;`
* `w-64` $\rightarrow$ `width: 256px;`
* `max-w-lg` $\rightarrow$ `max-width: 512px;`

#### 4. Flexbox Layout & Alignment Utilities
* `items-center` $\rightarrow$ `align-items: center;`
* `items-start` $\rightarrow$ `align-items: flex-start;`
* `items-end` $\rightarrow$ `align-items: flex-end;`
* `items-baseline` $\rightarrow$ `align-items: baseline;`
* `items-stretch` $\rightarrow$ `align-items: stretch;`
* `justify-between` $\rightarrow$ `justify-content: space-between;`
* `justify-center` $\rightarrow$ `justify-content: center;`
* `justify-start` $\rightarrow$ `justify-content: flex-start;`
* `justify-end` $\rightarrow$ `justify-content: flex-end;`
* `flex-row` $\rightarrow$ `flex-direction: row;`
* `flex-col` $\rightarrow$ `flex-direction: column;`
* `flex` $\rightarrow$ `display: flex;`
* `grid` $\rightarrow$ `display: grid;`
* `grid-cols-3` $\rightarrow$ `grid-template-columns: repeat(3, minmax(0, 1fr));`

#### 5. Gap Spacing Utilities
* `gap-small` $\rightarrow$ `gap: 8px;`
* `gap-medium` $\rightarrow$ `gap: 16px;`
* `gap-large` $\rightarrow$ `gap: 24px;`

#### 6. Typography Size & Text Align Utilities
* `text-xs` $\rightarrow$ `font-size: 12px;`
* `small` $\rightarrow$ `font-size: 14px;`
* `medium` $\rightarrow$ `font-size: 20px;`
* `large` $\rightarrow$ `font-size: 64px;`
* `bold` $\rightarrow$ `font-weight: 700;`
* `italic` $\rightarrow$ `font-style: italic;`
* `text-left` $\rightarrow$ `text-align: left;`
* `text-center` $\rightarrow$ `text-align: center;`
* `text-right` $\rightarrow$ `text-align: right;`

#### 7. Borders, Radius & Shadows
* `border` $\rightarrow$ `border: 1px solid;`
* `border-2` $\rightarrow$ `border: 2px solid;`
* `border-t` $\rightarrow$ `border-top: 1px solid;`
* `border-b` $\rightarrow$ `border-bottom: 1px solid;`
* `border-l` $\rightarrow$ `border-left: 1px solid;`
* `rounded-sm` $\rightarrow$ `border-radius: 4px;`
* `rounded-md` $\rightarrow$ `border-radius: 8px;`
* `rounded-lg` $\rightarrow$ `border-radius: 12px;`
* `rounded-xl` $\rightarrow$ `border-radius: 16px;`
* `rounded-2xl` $\rightarrow$ `border-radius: 20px;`
* `rounded-full` $\rightarrow$ `border-radius: 9999px;`
* `shadow-sm` $\rightarrow$ box shadow level 1
* `shadow-md` $\rightarrow$ box shadow level 2
* `shadow-lg` $\rightarrow$ box shadow level 3
* `shadow-xl` $\rightarrow$ box shadow level 4 (extra large)

#### 8. Positioning & Display
* `inline-block` $\rightarrow$ `display: inline-block;`
* `fixed-top` $\rightarrow$ sticky top positioning: `position: fixed; top: 0; left: 0; right: 0; z-index: 1000;`
* `relative` $\rightarrow$ `position: relative;`
* `mx-auto` $\rightarrow$ center sizing: `margin-left: auto; margin-right: auto;`
* `opacity-80` $\rightarrow$ `opacity: 0.8;`
* `opacity-90` $\rightarrow$ `opacity: 0.9;`
* `scale-105` $\rightarrow$ scale transform hover transitions.

---

### Spelling Suggestions & Validation Errors

If a developer writes an invalid style override key or JIT utility flag, the AUIG compiler catches the error at compile-time and suggests the closest matching vocabulary option using Levenshtein distance.

#### 1. Unknown Style Property Error
If you type `txt` instead of `text` in a style block:
```text
AUIG-E107: Unknown style property "txt"

Did you mean:
  text

File:
  pages/about.aui:12
```

#### 2. Unknown Utility Error
If you write a typo in a class name inside a style block:
```text
AUIG-E108: Unknown utility "bg-greeen-50"

Did you mean:
  bg-green-50

File:
  pages/docs.aui:18
```
