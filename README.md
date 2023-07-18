# Hyperide

Macros for generating HTML inside Rust. Think of it a bit like leptos, yew, or any other crate that
provides HTML in Rust, but without 99% of the functionality. You write HTML like syntax, and you get
a `String` back.

```rust

```

```html
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
  </head>
  <body>
    <h1>Hello, world!</h1>
    <p>This is in a closed paragraph.</p>
    <!-- "wildcard close tag ⬆️" -->
    <p><strong>Foo: </strong>bar</p>
  </body>
</html>
```
