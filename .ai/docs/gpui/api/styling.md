# Styling API

## Colors

### Color Types

```rust
pub type Hsla = palette::Hsla<f32>;
pub type Rgba = palette::Rgba<f32>;
```

### Color Functions

```rust
pub fn rgb(hex: u32) -> Hsla
pub fn rgba(hex: u32) -> Hsla
pub fn hsl(h: f32, s: f32, l: f32) -> Hsla
pub fn hsla(h: f32, s: f32, l: f32, a: f32) -> Hsla
```

### Predefined Colors

```rust
pub fn red() -> Hsla
pub fn green() -> Hsla
pub fn blue() -> Hsla
pub fn yellow() -> Hsla
pub fn cyan() -> Hsla
pub fn magenta() -> Hsla
pub fn black() -> Hsla
pub fn white() -> Hsla
pub fn transparent() -> Hsla
```

### Color Examples

```rust
use gpui::{rgb, rgba, hsl, hsla, red, blue};

div()
    .bg(rgb(0xff0000))           // Red background
    .text_color(blue())          // Blue text
    .border_color(rgba(0x00ff00ff)) // Green border with alpha
    .hover(|style| {
        style.bg(hsl(0.5, 1.0, 0.5)) // Cyan on hover
    })
```

## Layout System

### Flexbox

```rust
div()
    .flex()                      // Enable flexbox
    .flex_col()                  // Column direction
    .flex_row()                  // Row direction (default)
    .flex_wrap()                 // Allow wrapping
    .justify_center()            // Center main axis
    .justify_start()             // Start main axis
    .justify_end()               // End main axis
    .justify_between()           // Space between
    .justify_around()            // Space around
    .items_center()              // Center cross axis
    .items_start()               // Start cross axis
    .items_end()                 // End cross axis
    .items_stretch()             // Stretch cross axis
```

### Grid Layout

```rust
div()
    .grid()                      // Enable grid
    .grid_cols(3)                // 3 columns
    .grid_rows(2)                // 2 rows
    .gap_4()                     // Gap between items
    .col_span(2)                 // Span 2 columns
    .row_span(1)                 // Span 1 row
```

### Positioning

```rust
div()
    .relative()                  // Relative positioning
    .absolute()                  // Absolute positioning
    .fixed()                     // Fixed positioning
    .top(px(10.))               // Top offset
    .right(px(10.))             // Right offset
    .bottom(px(10.))            // Bottom offset
    .left(px(10.))              // Left offset
    .z_index(10)                // Z-index
```

## Sizing

### Width and Height

```rust
div()
    .w(px(100.))                // Fixed width
    .h(px(50.))                 // Fixed height
    .w_full()                   // Full width (100%)
    .h_full()                   // Full height (100%)
    .w_auto()                   // Auto width
    .h_auto()                   // Auto height
    .min_w(px(50.))             // Minimum width
    .max_w(px(200.))            // Maximum width
    .min_h(px(30.))             // Minimum height
    .max_h(px(100.))            // Maximum height
```

### Size Utilities

```rust
div()
    .size(px(100.))             // Square (100x100)
    .size_full()                // Full size
    .aspect_ratio(16.0 / 9.0)   // Aspect ratio
```

## Spacing

### Padding

```rust
div()
    .p(px(16.))                 // All sides
    .px(px(16.))                // Horizontal
    .py(px(8.))                 // Vertical
    .pt(px(8.))                 // Top
    .pr(px(16.))                // Right
    .pb(px(8.))                 // Bottom
    .pl(px(16.))                // Left
    .p_0()                      // No padding
    .p_1()                      // 4px padding
    .p_2()                      // 8px padding
    .p_4()                      // 16px padding
    .p_8()                      // 32px padding
```

### Margin

```rust
div()
    .m(px(16.))                 // All sides
    .mx(px(16.))                // Horizontal
    .my(px(8.))                 // Vertical
    .mt(px(8.))                 // Top
    .mr(px(16.))                // Right
    .mb(px(8.))                 // Bottom
    .ml(px(16.))                // Left
    .mx_auto()                  // Center horizontally
    .m_0()                      // No margin
    .m_1()                      // 4px margin
    .m_2()                      // 8px margin
    .m_4()                      // 16px margin
```

### Gap

```rust
div()
    .gap(px(16.))               // Gap between children
    .gap_x(px(16.))             // Horizontal gap
    .gap_y(px(8.))              // Vertical gap
    .gap_0()                    // No gap
    .gap_1()                    // 4px gap
    .gap_2()                    // 8px gap
    .gap_4()                    // 16px gap
```

## Borders

### Border Width

```rust
div()
    .border(px(1.))             // All sides
    .border_t(px(1.))           // Top
    .border_r(px(1.))           // Right
    .border_b(px(1.))           // Bottom
    .border_l(px(1.))           // Left
    .border_0()                 // No border
    .border_1()                 // 1px border
    .border_2()                 // 2px border
    .border_4()                 // 4px border
```

### Border Style

```rust
div()
    .border_solid()             // Solid border
    .border_dashed()            // Dashed border
    .border_dotted()            // Dotted border
    .border_none()              // No border
```

### Border Radius

```rust
div()
    .rounded(px(8.))            // All corners
    .rounded_tl(px(8.))         // Top-left
    .rounded_tr(px(8.))         // Top-right
    .rounded_bl(px(8.))         // Bottom-left
    .rounded_br(px(8.))         // Bottom-right
    .rounded_none()             // No rounding
    .rounded_sm()               // Small (2px)
    .rounded_md()               // Medium (6px)
    .rounded_lg()               // Large (8px)
    .rounded_xl()               // Extra large (12px)
    .rounded_full()             // Fully rounded
```

## Typography

### Text Size

```rust
text("Hello")
    .size(px(16.))              // Custom size
    .text_xs()                  // 12px
    .text_sm()                  // 14px
    .text_base()                // 16px
    .text_lg()                  // 18px
    .text_xl()                  // 20px
    .text_2xl()                 // 24px
    .text_3xl()                 // 30px
```

### Font Weight

```rust
text("Hello")
    .weight(FontWeight::Thin)
    .weight(FontWeight::Light)
    .weight(FontWeight::Normal)
    .weight(FontWeight::Medium)
    .weight(FontWeight::SemiBold)
    .weight(FontWeight::Bold)
    .weight(FontWeight::ExtraBold)
    .weight(FontWeight::Black)
```

### Text Style

```rust
text("Hello")
    .italic()                   // Italic text
    .underline()                // Underlined text
    .line_through()             // Strikethrough text
    .uppercase()                // Uppercase
    .lowercase()                // Lowercase
    .capitalize()               // Capitalize first letter
```

### Text Alignment

```rust
div()
    .text_left()                // Left align
    .text_center()              // Center align
    .text_right()               // Right align
    .text_justify()             // Justify
```

## Effects

### Shadows

```rust
div()
    .shadow_sm()                // Small shadow
    .shadow_md()                // Medium shadow
    .shadow_lg()                // Large shadow
    .shadow_xl()                // Extra large shadow
    .shadow_none()              // No shadow
```

### Opacity

```rust
div()
    .opacity(0.5)               // 50% opacity
    .opacity_0()                // Invisible
    .opacity_25()               // 25% opacity
    .opacity_50()               // 50% opacity
    .opacity_75()               // 75% opacity
    .opacity_100()              // Fully opaque
```

### Transforms

```rust
div()
    .scale(1.1)                 // Scale 110%
    .rotate(45.0)               // Rotate 45 degrees
    .translate_x(px(10.))       // Move right 10px
    .translate_y(px(-5.))       // Move up 5px
```

## Responsive Design

### Conditional Styling

```rust
div()
    .when(is_mobile, |div| {
        div.flex_col().p_2()
    })
    .when(!is_mobile, |div| {
        div.flex_row().p_8()
    })
```

### State-Based Styling

```rust
div()
    .bg(rgb(0xffffff))
    .hover(|style| {
        style.bg(rgb(0xf3f4f6))
    })
    .active(|style| {
        style.bg(rgb(0xe5e7eb))
    })
    .focus(|style| {
        style.border_color(rgb(0x3b82f6))
    })
```

## Units

### Pixel Units

```rust
use gpui::{px, Pixels};

div()
    .w(px(100.))                // 100 pixels
    .h(Pixels(50.0))            // 50 pixels
```

### Relative Units

```rust
div()
    .w_full()                   // 100% width
    .h_full()                   // 100% height
    .w(relative(0.5))           // 50% width
    .h(relative(0.25))          // 25% height
```
