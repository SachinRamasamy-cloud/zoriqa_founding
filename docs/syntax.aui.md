# AUIG V1 Syntax Guide

AUIG features a clean, readable syntax inspired by Python's indentation rules, designed to be easy for beginners while remaining powerful for advanced layouts.

## Core Syntax Rules

### 1. Easy-by-Default Elements
To create basic elements, specify the element name followed by its main content (positional string argument).

```aui
h1 "Hello World"
p "Build websites faster"
```

### 2. Semantic Flags
Flags represent style meanings. Always place positional arguments first, followed by semantic flags.

```aui
btn "Get Started" primary
badge "New" success
```

### 3. Named Properties
Named properties are used exclusively for defining behavior (e.g. links, image sources, placeholders).

```aui
link "Home" to "/"
image "Logo" src "/logo.png"
input "Email" placeholder "Enter email address"
```

### 4. Child Directives
Use indented blocks (ending with `:`) or curly braces (`{ }`) to specify nested child content.

```aui
view:
  navbar "AUIG" dark
  center:
    h1 "Welcome"
    p "This is nested content."
```
