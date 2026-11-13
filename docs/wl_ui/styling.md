# wl_ui styling and design system

The styling API lives in `wl_ui_core` and is re-exported from `wl_ui`. It is
renderer-neutral by design: themes describe visual intent, selectors identify
semantic components, and style values are typed instead of accepting arbitrary
CSS strings.

## 1. Design system

A theme is composed of a semantic palette, typography, metrics and effects:

```rust
use wl_ui::{UiEffects, UiTheme, UiTypography, UiMetrics};

let theme = UiTheme::dark()
    .name("Midnight")
    .typography(UiTypography {
        font_family: "Segoe UI Variable".into(),
        monospace_family: "Cascadia Mono".into(),
        ..UiTypography::default()
    })
    .metrics(UiMetrics {
        spacing_unit: 4.0,
        radius_sm: 6.0,
        radius_md: 10.0,
        radius_lg: 14.0,
        content_padding: 24.0,
        ..UiMetrics::default()
    })
    .effects(UiEffects {
        reduced_motion: false,
        ..UiEffects::default()
    });
```

Presets: `UiTheme::system()`, `UiTheme::light()`, `UiTheme::dark()` and
`UiTheme::high_contrast()`. Select a scheme with
`UiColorScheme::{System, Light, Dark, HighContrast}`.

### Palette roles

`UiPalette` provides window, surface, alternate/elevated surface, primary/
secondary/disabled text, border/strong border, accent/hover/pressed/accent
text, selection/selection text, focus, success/warning/danger/info foreground
and surface colors, and shadow. Override a palette by copying a preset:

```rust
use wl_ui::{UiColor, UiPalette, UiTheme};

let light = UiPalette {
    accent: UiColor::rgb(0, 112, 210),
    accent_hover: UiColor::rgb(0, 92, 180),
    ..UiPalette::light()
};

let theme = UiTheme::system().light_palette(light);
```

Colors are RGBA bytes: `UiColor::rgb(r, g, b)`,
`UiColor::rgba(r, g, b, a)`, or `UiColor::from_hex("#RRGGBB")` /
`UiColor::from_hex("#RRGGBBAA")`.

### Typography

`UiTypography` contains regular and monospace font families, six text sizes,
four weight tokens, line height and letter spacing. Values are expressed in
device-independent pixels, except line height, which is a multiplier.

### Metrics

`UiMetrics` contains the spacing base, six corner-radius presets, border
widths, small/medium/large control heights, content padding, navigation width,
icon sizes and scrollbar width.

### Effects and motion

`UiEffects` defines three `UiShadow` levels, focus-ring width/offset, three
motion durations and a `reduced_motion` preference. Renderers must respect
reduced motion rather than substituting an animation with a different one.

## 2. Component selectors and states

A selector starts with a semantic component and can add a CSS-compatible class,
ID and interaction state:

```rust
use wl_ui::{UiComponent, UiSelector, UiVisualState};

let primary_hover = UiSelector::new(UiComponent::Button)
    .class("primary")
    .state(UiVisualState::Hover);
```

Built-in component names are `Window`, `Surface`, `Card`, `Text`,
`Heading`, `Button`, `TextInput`, `Select`, `Checkbox`, `Radio`,
`Switch`, `Slider`, `Navigation`, `NavigationItem`, `Dialog`, `Menu`,
`MenuItem`, `Tooltip`, `Badge`, `Progress`, `Divider`, `List`,
`ListItem`, `Image` and `Icon`. Use
`UiComponent::Custom("my-widget".into())` for an application-defined
component. Custom names must be valid identifiers.

States: `Hover`, `Pressed`, `Focused`, `Disabled`, `Selected`,
`Checked`, `Invalid` and `Loading`. They map to CSS pseudo-classes or ARIA
state attributes for CSS renderers.

Selectors deliberately do not accept raw CSS selector text, combinators,
arbitrary attributes or JavaScript. This keeps generated styles predictable
and prevents an application string from becoming executable stylesheet
syntax.

## 3. Style declarations

`UiStyle` is a builder. Unset properties inherit from the theme or the
selected renderer's defaults. Builder methods consume and return the style value.
Renderers can inspect every configured property through the corresponding
read-only `*_value()` accessor (for list-backed values such as grid tracks and
transitions, the accessor returns a borrowed slice). `UiSelector` likewise
exposes `component()`, `class_name()`, `id_name()` and `visual_state()`.
This lets native adapters map the typed model directly without parsing CSS.

### Colors and paint

- `background(UiColor)`, `background_paint(UiBackground)`
- `linear_gradient(UiLinearGradient)`
- `foreground(UiColor)`, `border_color(UiColor)`
- `shadow(UiShadow)`, `text_shadow(UiShadow)`

Box-shadow offsets and spread may be negative (for directional and contracting
shadows); blur radii must be finite and non-negative. Text shadows use finite
signed X/Y offsets and a non-negative blur; their spread field must be zero
because CSS `text-shadow` has no spread component.

Gradients are typed linear gradients with 2–16 ordered stops. A gradient angle
must be finite and between 0 and 360 degrees; stop positions must be from 0 to
100 percent. Image URLs, external CSS imports and arbitrary paint expressions
are intentionally not accepted by this API.

### Typography and text

- `font_family`, `font_size`, `font_weight`, `font_style`
- `line_height`, `letter_spacing`, `text_align`
- `text_decoration`, `text_transform`, `white_space`, `text_overflow`
- `text_shadow(UiShadow)` for a validated text-shadow value

### Box model and sizing

- `padding(UiEdges)`, `margin(UiEdges)`, `gap`
- `width`, `height`, `min_width`, `min_height`, `max_width`, `max_height`
- `aspect_ratio`, `box_sizing`
- `border(width, UiBorderStyle, color)`, `border_radius`,
  `corner_radii(UiCornerRadii)`
- `outline(width, offset, color)`, `opacity`

`UiEdges::all(value)`, `UiEdges::xy(horizontal, vertical)` and
`UiEdges::new(top, right, bottom, left)` are available for spacing. Lengths
are `UiLength::Auto`, `UiLength::Px(value)` or
`UiLength::Percent(value)`. Position insets (`top`, `right`, `bottom`,
`left`) accept finite negative pixel and percentage values; outline offsets
may also be negative. Width/height dimensions remain non-negative.

### Layout

`UiDisplay` supports block, inline, inline-block, flex, inline-flex, grid and
none. `UiPosition` supports static, relative, absolute, fixed and sticky.
`UiOverflow` supports visible, hidden, clip, auto and scroll.

Flex properties include direction, wrapping, justify-content, align-items,
grow and shrink.

Grid tracks use `UiGridTrack::Auto`, `Px(value)`, `Fraction(value)`,
`MinContent` and `MaxContent`. A style can set both axes at once with
`grid_template(columns, rows)`, or set either axis with `grid_columns` /
`grid_rows`. Each axis accepts 1–12 tracks; fractional values must be
positive and pixel values non-negative. `grid_gap(row, column)`,
`align_content` and `justify_items` control the grid spacing/alignment.

```rust
use wl_ui::{
    UiAlignContent, UiCornerRadii, UiDisplay, UiGridTrack, UiJustifyItems, UiStyle,
};

let panel = UiStyle::new()
    .display(UiDisplay::Grid)
    .grid_template(
        [UiGridTrack::Fraction(1.0), UiGridTrack::Px(280.0)],
        [UiGridTrack::Auto, UiGridTrack::Fraction(1.0)],
    )
    .grid_gap(12.0, 16.0)
    .align_content(UiAlignContent::Start)
    .justify_items(UiJustifyItems::Stretch)
    .corner_radii(UiCornerRadii::new(8.0, 8.0, 12.0, 12.0));
```

Box sizing, z-index and top/right/bottom/left offsets are available. Image
content can use `UiObjectFit`. Text styling supports `UiFontStyle` and
`UiTextOverflow::{Clip, Ellipsis}`; the latter emits the CSS
`text-overflow` property (the frontend remains responsible for clipping and
overflow layout).

### Motion and transforms

- `transform(UiTransform)` supports translate, scale and rotate.
- `transition(UiTransition)` takes a typed property, duration, easing and
  optional delay.
- Easing: linear, ease, ease-in, ease-out and ease-in-out.
- Transition properties: all, opacity, transform, color, background, border
  and shadow.

Values are checked for finite ranges. Opacity must be 0–1, font weights
1–1000, font sizes and line heights must be positive. Margins may be negative;
padding, dimensions, gaps, outline widths and radii remain non-negative.
Position insets and outline offsets may be negative. Percentage dimensions
may exceed 100% for intentional overflow or positioning layouts. Transform
scales may be negative to mirror content; all transform components must be
finite.

### Declarative keyframe animations

Animations are described with typed values rather than raw CSS or script.
A stylesheet must contain the matching keyframe definition before a component
can reference it:

```rust
use wl_ui::{
    UiAnimation, UiAnimationDirection, UiAnimationFillMode,
    UiAnimationIterations, UiComponent, UiKeyframe, UiKeyframes,
    UiSelector, UiStyle, UiStyleSheet,
};

let fade = UiKeyframes::new(
    "fade_in",
    [
        UiKeyframe::new(0.0, UiStyle::new().opacity(0.0)),
        UiKeyframe::new(100.0, UiStyle::new().opacity(1.0)),
    ],
);

let styles = UiStyleSheet::new()
    .keyframes(fade)
    .rule(
        UiSelector::new(UiComponent::Card).class("entering"),
        UiStyle::new().animation(
            UiAnimation::new("fade_in", 240)
                .iterations(UiAnimationIterations::Once)
                .direction(UiAnimationDirection::Normal)
                .fill_mode(UiAnimationFillMode::Both),
        ),
    );
```

The animation API also supports signed start delays, bounded repeat counts or
infinite repetition, direction, fill mode, easing and running/paused state.
Animation/keyframe names are validated identifiers (maximum 64 characters);
each keyframe set contains 2–64 strictly ordered frames; a stylesheet permits
at most 64 keyframe sets and each component at most 8 simultaneous animations.
Durations and delays are capped at one hour and repeat counts at 1000. Nested
animations/transitions inside keyframe declarations are rejected.

The generated CSS uses the shared `--wl-motion-enabled` token and includes
reduced-motion overrides for system preferences. A theme with
`UiEffects::reduced_motion = true` also disables repeated motion. No
JavaScript, callback, external stylesheet, arbitrary selector or raw CSS value
is accepted by these APIs.

`wl_ui::animate` re-exports the portable `wl_animate` crate for host-driven
native animation. It provides caller-driven tweens and keyframe sampling; the
host supplies elapsed time and owns its frame loop. CSS-capable renderers such
as Tauri use the validated CSS keyframes and the browser's native animation
engine. Native renderers must explicitly implement animation capability or
report it unsupported; the shared model never pretends an unimplemented
backend renders animation.

## 4. Stylesheets and responsive rules

Rules are ordered: later matching rules are emitted later and win in the CSS
cascade.

```rust
use wl_ui::{
    UiColor, UiComponent, UiEdges, UiMediaCondition, UiSelector, UiStyle,
    UiStyleSheet,
};

let stylesheet = UiStyleSheet::new()
    .rule(
        UiSelector::new(UiComponent::Button).class("primary"),
        UiStyle::new()
            .background(UiColor::rgb(0, 103, 192))
            .foreground(UiColor::rgb(255, 255, 255))
            .padding(UiEdges::xy(16.0, 10.0))
            .border_radius(8.0),
    )
    .media_rule(
        UiSelector::new(UiComponent::Card).class("settings"),
        UiStyle::new().padding(UiEdges::all(28.0)),
        UiMediaCondition::MinWidth(900.0),
    );
```

`UiMediaCondition` supports minimum/maximum width, light/dark system scheme,
reduced motion and forced colors. `UiStyleSheet::validate()` checks selectors,
media conditions and declaration values. `UiStyleSheet::to_css()` generates
the renderer CSS.

## 5. Theme CSS variables

`UiTheme::css_variables()` exports the design tokens for CSS-capable
renderers. It includes semantic colors, font families/sizes/weights, spacing,
radii, control heights, focus rings, shadows and motion durations. System
themes respond to `prefers-color-scheme`; forced-colors and reduced-motion
overrides are emitted even when a fixed light/dark palette is selected.

Generated variables use the `--wl-` namespace, for example:

- `--wl-color-window`, `--wl-color-surface`,
  `--wl-color-text-primary`, `--wl-color-accent`
- `--wl-font-family`, `--wl-font-size-md`, `--wl-font-weight-semibold`
- `--wl-space-unit`, `--wl-radius-md`, `--wl-content-padding`
- `--wl-shadow-sm`, `--wl-focus-ring-width`,
  `--wl-motion-normal`

## 6. Tauri/WebKit markup

The Tauri adapter installs the generated CSS in a style element with ID
`wl-ui-theme` and publishes it in
`window.__WL_UI_BOOTSTRAP__.theme.css`. Frontends opt into component rules
with the matching data attribute and class:

```html
<button data-wl-component="button" class="primary">Save</button>
<section data-wl-component="card" class="settings">...</section>
```

The app owns its HTML, JS, accessibility attributes, commands, permissions,
plugins and capabilities. Use semantic HTML and ARIA state attributes so
state selectors can match. The adapter doesn't manufacture a frontend or
replace the host's CSS architecture; it supplies the reusable design tokens
and generated wl_ui rules.

## 7. Backend translation and unsupported styles

The data model is shared; rendering capability is not identical:

- **Windows 10 Win32:** maps solid window/navigation/content backgrounds,
  semantic text color and the theme font to native GDI/Win32 resources.
  Gradients, CSS selectors/states, flex/grid, blur and per-element animation
  are not rendered by the current Win32 shell.
- **Windows 11 WinUI 3:** applies the theme palette, fonts, page surface,
  border, radius and spacing to the native shell. The Windows Reactor controls
  are re-exported from `wl_ui::windows11` on Windows so an app can build a
  richer native component tree. CSS is not injected into WinUI.
- **Tauri 2:** emits the complete supported CSS variable and selector model.
  The frontend must provide matching `data-wl-component` attributes and
  classes.

No backend silently substitutes a different renderer or claims that an
unsupported CSS property was applied.
