# AUIG V1 Styling Guide

AUIG promotes clean layouts by separating markup from custom overrides.

## Styling Precedence
1. **Default Component Style**: Baseline styles for each tier.
2. **Theme Preset**: Map semantic tones globally.
3. **Tone**: Apply color tones (`primary`, `success`, `warning`, `danger`, `info`, `neutral`).
4. **Variant**: Apply variant shapes (`soft`, `solid`, `outline`, `dark`, `light`, `minimal`).
5. **Style Block**: Advanced customized styling overrides.

## The Style Block (`style:`)
For advanced customization, use a nested `style:` block. Avoid placing styling utility classes on the element/component declaration line.

```aui
stat-card "Revenue" "$48k" success:
  style:
    bg green-50
    text green-900
    border green-200
    radius xl
    shadow lg
```

### Allowed Style Keys
Only the following keys are allowed inside a `style:` block:
- `bg` (background color class)
- `text` (text color class)
- `border` (border class)
- `radius` (border radius utility)
- `shadow` (box shadow utility)
- `padding`
- `margin`
- `width`
- `height`
- `align`
- `justify`

Any unknown keys (e.g. `txt`) will throw a compile-time compiler error suggesting the closest valid key.
