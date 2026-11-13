use crate::UiError;

/// An 8-bit RGBA color shared by every wl_ui backend.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UiColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

impl UiColor {
    pub const fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha: 255,
        }
    }

    pub const fn rgba(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }

    /// Parses #RRGGBB or #RRGGBBAA. Returns None for other forms.
    pub fn from_hex(value: &str) -> Option<Self> {
        let hex = value.strip_prefix('#')?;
        if !hex.is_ascii() || (hex.len() != 6 && hex.len() != 8) {
            return None;
        }
        let red = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let green = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let blue = u8::from_str_radix(&hex[4..6], 16).ok()?;
        let alpha = if hex.len() == 8 {
            u8::from_str_radix(&hex[6..8], 16).ok()?
        } else {
            255
        };
        Some(Self::rgba(red, green, blue, alpha))
    }

    pub fn to_css(self) -> String {
        format!(
            "rgba({}, {}, {}, {:.3})",
            self.red,
            self.green,
            self.blue,
            f32::from(self.alpha) / 255.0
        )
    }
}

/// Selects which palette a renderer should resolve.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum UiColorScheme {
    #[default]
    System,
    Light,
    Dark,
    HighContrast,
}

impl UiColorScheme {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::Dark => "dark",
            Self::HighContrast => "high-contrast",
        }
    }
}

/// Semantic colors. Components should use these roles instead of hard-coded colors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiPalette {
    pub window: UiColor,
    pub surface: UiColor,
    pub surface_alt: UiColor,
    pub surface_elevated: UiColor,
    pub text_primary: UiColor,
    pub text_secondary: UiColor,
    pub text_disabled: UiColor,
    pub border: UiColor,
    pub border_strong: UiColor,
    pub accent: UiColor,
    pub accent_hover: UiColor,
    pub accent_pressed: UiColor,
    pub accent_text: UiColor,
    pub selection: UiColor,
    pub selection_text: UiColor,
    pub focus: UiColor,
    pub success: UiColor,
    pub success_surface: UiColor,
    pub warning: UiColor,
    pub warning_surface: UiColor,
    pub danger: UiColor,
    pub danger_surface: UiColor,
    pub info: UiColor,
    pub info_surface: UiColor,
    pub shadow: UiColor,
}

impl UiPalette {
    pub const fn light() -> Self {
        Self {
            window: UiColor::rgb(248, 249, 251),
            surface: UiColor::rgb(255, 255, 255),
            surface_alt: UiColor::rgb(242, 244, 247),
            surface_elevated: UiColor::rgb(255, 255, 255),
            text_primary: UiColor::rgb(27, 34, 46),
            text_secondary: UiColor::rgb(87, 96, 109),
            text_disabled: UiColor::rgb(142, 148, 157),
            border: UiColor::rgb(222, 226, 232),
            border_strong: UiColor::rgb(190, 197, 207),
            accent: UiColor::rgb(0, 103, 192),
            accent_hover: UiColor::rgb(0, 88, 168),
            accent_pressed: UiColor::rgb(0, 72, 140),
            accent_text: UiColor::rgb(255, 255, 255),
            selection: UiColor::rgb(218, 238, 255),
            selection_text: UiColor::rgb(12, 49, 82),
            focus: UiColor::rgb(0, 103, 192),
            success: UiColor::rgb(16, 124, 16),
            success_surface: UiColor::rgb(225, 245, 222),
            warning: UiColor::rgb(157, 93, 0),
            warning_surface: UiColor::rgb(255, 244, 206),
            danger: UiColor::rgb(196, 43, 28),
            danger_surface: UiColor::rgb(253, 231, 230),
            info: UiColor::rgb(0, 103, 192),
            info_surface: UiColor::rgb(226, 241, 255),
            shadow: UiColor::rgba(0, 0, 0, 48),
        }
    }

    pub const fn dark() -> Self {
        Self {
            window: UiColor::rgb(24, 24, 28),
            surface: UiColor::rgb(32, 32, 36),
            surface_alt: UiColor::rgb(41, 41, 46),
            surface_elevated: UiColor::rgb(49, 49, 55),
            text_primary: UiColor::rgb(244, 244, 247),
            text_secondary: UiColor::rgb(190, 192, 198),
            text_disabled: UiColor::rgb(133, 136, 143),
            border: UiColor::rgb(69, 70, 77),
            border_strong: UiColor::rgb(96, 98, 107),
            accent: UiColor::rgb(96, 157, 234),
            accent_hover: UiColor::rgb(121, 176, 244),
            accent_pressed: UiColor::rgb(65, 130, 215),
            accent_text: UiColor::rgb(12, 22, 36),
            selection: UiColor::rgb(45, 80, 115),
            selection_text: UiColor::rgb(246, 250, 255),
            focus: UiColor::rgb(124, 181, 246),
            success: UiColor::rgb(108, 200, 120),
            success_surface: UiColor::rgb(32, 63, 39),
            warning: UiColor::rgb(243, 190, 90),
            warning_surface: UiColor::rgb(77, 55, 20),
            danger: UiColor::rgb(255, 133, 128),
            danger_surface: UiColor::rgb(78, 34, 32),
            info: UiColor::rgb(117, 183, 245),
            info_surface: UiColor::rgb(30, 51, 77),
            shadow: UiColor::rgba(0, 0, 0, 112),
        }
    }

    pub const fn high_contrast() -> Self {
        Self {
            window: UiColor::rgb(0, 0, 0),
            surface: UiColor::rgb(0, 0, 0),
            surface_alt: UiColor::rgb(18, 18, 18),
            surface_elevated: UiColor::rgb(0, 0, 0),
            text_primary: UiColor::rgb(255, 255, 255),
            text_secondary: UiColor::rgb(255, 255, 255),
            text_disabled: UiColor::rgb(190, 190, 190),
            border: UiColor::rgb(255, 255, 255),
            border_strong: UiColor::rgb(255, 255, 0),
            accent: UiColor::rgb(255, 255, 0),
            accent_hover: UiColor::rgb(255, 255, 0),
            accent_pressed: UiColor::rgb(255, 255, 0),
            accent_text: UiColor::rgb(0, 0, 0),
            selection: UiColor::rgb(255, 255, 0),
            selection_text: UiColor::rgb(0, 0, 0),
            focus: UiColor::rgb(255, 255, 0),
            success: UiColor::rgb(0, 255, 0),
            success_surface: UiColor::rgb(0, 0, 0),
            warning: UiColor::rgb(255, 255, 0),
            warning_surface: UiColor::rgb(0, 0, 0),
            danger: UiColor::rgb(255, 128, 128),
            danger_surface: UiColor::rgb(0, 0, 0),
            info: UiColor::rgb(0, 255, 255),
            info_surface: UiColor::rgb(0, 0, 0),
            shadow: UiColor::rgb(0, 0, 0),
        }
    }
}

/// Font families, type ramp, weights and baseline text rhythm.
#[derive(Clone, Debug, PartialEq)]
pub struct UiTypography {
    pub font_family: String,
    pub monospace_family: String,
    pub size_xs: f32,
    pub size_sm: f32,
    pub size_md: f32,
    pub size_lg: f32,
    pub size_xl: f32,
    pub size_display: f32,
    pub weight_regular: u16,
    pub weight_medium: u16,
    pub weight_semibold: u16,
    pub weight_bold: u16,
    pub line_height: f32,
    pub letter_spacing: f32,
}

impl Default for UiTypography {
    fn default() -> Self {
        Self {
            font_family: "Segoe UI".to_string(),
            monospace_family: "Cascadia Mono".to_string(),
            size_xs: 11.0,
            size_sm: 12.0,
            size_md: 14.0,
            size_lg: 18.0,
            size_xl: 24.0,
            size_display: 32.0,
            weight_regular: 400,
            weight_medium: 500,
            weight_semibold: 600,
            weight_bold: 700,
            line_height: 1.45,
            letter_spacing: 0.0,
        }
    }
}

/// Layout and shape tokens measured in device-independent pixels.
#[derive(Clone, Debug, PartialEq)]
pub struct UiMetrics {
    pub spacing_unit: f32,
    pub radius_xs: f32,
    pub radius_sm: f32,
    pub radius_md: f32,
    pub radius_lg: f32,
    pub radius_xl: f32,
    pub radius_pill: f32,
    pub border_width: f32,
    pub border_width_strong: f32,
    pub control_height_sm: f32,
    pub control_height_md: f32,
    pub control_height_lg: f32,
    pub content_padding: f32,
    pub navigation_width: f32,
    pub icon_size_sm: f32,
    pub icon_size_md: f32,
    pub icon_size_lg: f32,
    pub scrollbar_width: f32,
}

impl Default for UiMetrics {
    fn default() -> Self {
        Self {
            spacing_unit: 4.0,
            radius_xs: 3.0,
            radius_sm: 6.0,
            radius_md: 9.0,
            radius_lg: 14.0,
            radius_xl: 20.0,
            radius_pill: 999.0,
            border_width: 1.0,
            border_width_strong: 2.0,
            control_height_sm: 28.0,
            control_height_md: 34.0,
            control_height_lg: 42.0,
            content_padding: 20.0,
            navigation_width: 248.0,
            icon_size_sm: 16.0,
            icon_size_md: 20.0,
            icon_size_lg: 24.0,
            scrollbar_width: 8.0,
        }
    }
}

/// A shadow token expressed in device-independent pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiShadow {
    pub color: UiColor,
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur: f32,
    pub spread: f32,
}

impl UiShadow {
    pub const fn new(color: UiColor, offset_x: f32, offset_y: f32, blur: f32, spread: f32) -> Self {
        Self {
            color,
            offset_x,
            offset_y,
            blur,
            spread,
        }
    }
}

/// Motion, elevation and keyboard focus tokens.
#[derive(Clone, Debug, PartialEq)]
pub struct UiEffects {
    pub shadow_sm: UiShadow,
    pub shadow_md: UiShadow,
    pub shadow_lg: UiShadow,
    pub focus_ring_width: f32,
    pub focus_ring_offset: f32,
    pub motion_fast_ms: u32,
    pub motion_normal_ms: u32,
    pub motion_slow_ms: u32,
    pub reduced_motion: bool,
}

impl Default for UiEffects {
    fn default() -> Self {
        Self {
            shadow_sm: UiShadow::new(UiColor::rgba(0, 0, 0, 36), 0.0, 1.0, 3.0, 0.0),
            shadow_md: UiShadow::new(UiColor::rgba(0, 0, 0, 52), 0.0, 4.0, 12.0, 0.0),
            shadow_lg: UiShadow::new(UiColor::rgba(0, 0, 0, 72), 0.0, 10.0, 28.0, 0.0),
            focus_ring_width: 2.0,
            focus_ring_offset: 2.0,
            motion_fast_ms: 90,
            motion_normal_ms: 160,
            motion_slow_ms: 240,
            reduced_motion: false,
        }
    }
}

/// A complete semantic theme with light, dark and high-contrast palettes.
#[derive(Clone, Debug, PartialEq)]
pub struct UiTheme {
    name: String,
    scheme: UiColorScheme,
    light: UiPalette,
    dark: UiPalette,
    high_contrast: UiPalette,
    typography: UiTypography,
    metrics: UiMetrics,
    effects: UiEffects,
}

impl UiTheme {
    pub fn system() -> Self {
        Self {
            name: "System".to_string(),
            scheme: UiColorScheme::System,
            light: UiPalette::light(),
            dark: UiPalette::dark(),
            high_contrast: UiPalette::high_contrast(),
            typography: UiTypography::default(),
            metrics: UiMetrics::default(),
            effects: UiEffects::default(),
        }
    }

    pub fn light() -> Self {
        Self::system().scheme(UiColorScheme::Light).name("Light")
    }

    pub fn dark() -> Self {
        Self::system().scheme(UiColorScheme::Dark).name("Dark")
    }

    pub fn high_contrast() -> Self {
        Self::system()
            .scheme(UiColorScheme::HighContrast)
            .name("High contrast")
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn scheme(mut self, scheme: UiColorScheme) -> Self {
        self.scheme = scheme;
        self
    }

    pub fn light_palette(mut self, palette: UiPalette) -> Self {
        self.light = palette;
        self
    }

    pub fn dark_palette(mut self, palette: UiPalette) -> Self {
        self.dark = palette;
        self
    }

    pub fn high_contrast_palette(mut self, palette: UiPalette) -> Self {
        self.high_contrast = palette;
        self
    }

    pub fn typography(mut self, typography: UiTypography) -> Self {
        self.typography = typography;
        self
    }

    pub fn metrics(mut self, metrics: UiMetrics) -> Self {
        self.metrics = metrics;
        self
    }

    pub fn effects(mut self, effects: UiEffects) -> Self {
        self.effects = effects;
        self
    }

    pub fn color_scheme(&self) -> UiColorScheme {
        self.scheme
    }

    pub fn light_colors(&self) -> &UiPalette {
        &self.light
    }

    pub fn dark_colors(&self) -> &UiPalette {
        &self.dark
    }

    pub fn high_contrast_colors(&self) -> &UiPalette {
        &self.high_contrast
    }

    pub fn theme_name(&self) -> &str {
        &self.name
    }

    /// Returns the palette selected by this theme. System resolves to light as a
    /// portable fallback; native adapters may substitute their operating-system colors.
    pub fn palette(&self) -> &UiPalette {
        self.palette_for(self.scheme)
    }

    pub fn palette_for(&self, scheme: UiColorScheme) -> &UiPalette {
        match scheme {
            UiColorScheme::System | UiColorScheme::Light => &self.light,
            UiColorScheme::Dark => &self.dark,
            UiColorScheme::HighContrast => &self.high_contrast,
        }
    }

    pub fn typography_tokens(&self) -> &UiTypography {
        &self.typography
    }

    pub fn metrics_tokens(&self) -> &UiMetrics {
        &self.metrics
    }

    pub fn effects_tokens(&self) -> &UiEffects {
        &self.effects
    }

    pub fn validate(&self) -> Result<(), UiError> {
        if self.name.trim().is_empty()
            || !valid_font_family(&self.typography.font_family)
            || !valid_font_family(&self.typography.monospace_family)
        {
            return Err(UiError::InvalidStyle(
                "theme name or font family is invalid".to_string(),
            ));
        }

        let typography = &self.typography;
        for value in [
            typography.size_xs,
            typography.size_sm,
            typography.size_md,
            typography.size_lg,
            typography.size_xl,
            typography.size_display,
            typography.line_height,
        ] {
            if !finite_positive(value) {
                return Err(UiError::InvalidStyle(
                    "typography values must be finite and positive".to_string(),
                ));
            }
        }
        if !typography.letter_spacing.is_finite()
            || [
                typography.weight_regular,
                typography.weight_medium,
                typography.weight_semibold,
                typography.weight_bold,
            ]
            .iter()
            .any(|weight| *weight == 0 || *weight > 1000)
        {
            return Err(UiError::InvalidStyle(
                "typography weight or letter spacing is invalid".to_string(),
            ));
        }

        let metrics = &self.metrics;
        for value in [
            metrics.spacing_unit,
            metrics.radius_xs,
            metrics.radius_sm,
            metrics.radius_md,
            metrics.radius_lg,
            metrics.radius_xl,
            metrics.radius_pill,
            metrics.border_width,
            metrics.border_width_strong,
            metrics.control_height_sm,
            metrics.control_height_md,
            metrics.control_height_lg,
            metrics.content_padding,
            metrics.navigation_width,
            metrics.icon_size_sm,
            metrics.icon_size_md,
            metrics.icon_size_lg,
            metrics.scrollbar_width,
        ] {
            if !finite_non_negative(value) {
                return Err(UiError::InvalidStyle(
                    "metric values must be finite and non-negative".to_string(),
                ));
            }
        }

        let effects = &self.effects;
        if !finite_non_negative(effects.focus_ring_width)
            || !finite_non_negative(effects.focus_ring_offset)
            || !valid_shadow(effects.shadow_sm)
            || !valid_shadow(effects.shadow_md)
            || !valid_shadow(effects.shadow_lg)
        {
            return Err(UiError::InvalidStyle(
                "effect values must be finite and shadow blur radii non-negative".to_string(),
            ));
        }
        Ok(())
    }

    /// Produces theme variables for CSS-capable renderers (Tauri/WebKit).
    /// System themes follow prefers-color-scheme and forced-colors.
    pub fn css_variables(&self) -> Result<String, UiError> {
        self.validate()?;
        let mut css = String::new();
        match self.scheme {
            UiColorScheme::System => {
                css.push_str(":root{color-scheme:light;");
                append_palette_variables(&mut css, &self.light);
                css.push('}');
                css.push_str("@media(prefers-color-scheme:dark){:root{color-scheme:dark;");
                append_palette_variables(&mut css, &self.dark);
                css.push_str("}}");
            }
            UiColorScheme::Light => {
                css.push_str(":root{color-scheme:light;");
                append_palette_variables(&mut css, &self.light);
                css.push('}');
            }
            UiColorScheme::Dark => {
                css.push_str(":root{color-scheme:dark;");
                append_palette_variables(&mut css, &self.dark);
                css.push('}');
            }
            UiColorScheme::HighContrast => {
                css.push_str(":root{color-scheme:light dark;");
                append_palette_variables(&mut css, &self.high_contrast);
                css.push('}');
            }
        }

        css.push_str("@media(forced-colors:active){:root{color-scheme:light dark;--wl-color-window:Canvas;--wl-color-surface:Canvas;--wl-color-surface-alt:Canvas;--wl-color-text-primary:CanvasText;--wl-color-text-secondary:CanvasText;--wl-color-border:CanvasText;--wl-color-accent:Highlight;--wl-color-accent-text:HighlightText;--wl-color-focus:Highlight;--wl-color-selection:Highlight;--wl-color-selection-text:HighlightText;}}");
        let typography = &self.typography;
        let metrics = &self.metrics;
        let effects = &self.effects;
        css.push_str(":root{");
        css.push_str(&format!(
            "--wl-font-family:{};--wl-font-mono:{};",
            css_font_family(&typography.font_family),
            css_font_family(&typography.monospace_family)
        ));
        css.push_str(&format!(
            "--wl-font-size-xs:{}px;--wl-font-size-sm:{}px;--wl-font-size-md:{}px;--wl-font-size-lg:{}px;--wl-font-size-xl:{}px;--wl-font-size-display:{}px;",
            number(typography.size_xs),
            number(typography.size_sm),
            number(typography.size_md),
            number(typography.size_lg),
            number(typography.size_xl),
            number(typography.size_display)
        ));
        css.push_str(&format!(
            "--wl-font-weight-regular:{};--wl-font-weight-medium:{};--wl-font-weight-semibold:{};--wl-font-weight-bold:{};--wl-line-height:{};--wl-letter-spacing:{}px;",
            typography.weight_regular,
            typography.weight_medium,
            typography.weight_semibold,
            typography.weight_bold,
            number(typography.line_height),
            number(typography.letter_spacing)
        ));
        css.push_str(&format!(
            "--wl-space-unit:{}px;--wl-radius-xs:{}px;--wl-radius-sm:{}px;--wl-radius-md:{}px;--wl-radius-lg:{}px;--wl-radius-xl:{}px;--wl-radius-pill:{}px;--wl-border-width:{}px;--wl-border-width-strong:{}px;",
            number(metrics.spacing_unit),
            number(metrics.radius_xs),
            number(metrics.radius_sm),
            number(metrics.radius_md),
            number(metrics.radius_lg),
            number(metrics.radius_xl),
            number(metrics.radius_pill),
            number(metrics.border_width),
            number(metrics.border_width_strong)
        ));
        css.push_str(&format!(
            "--wl-control-height-sm:{}px;--wl-control-height-md:{}px;--wl-control-height-lg:{}px;--wl-content-padding:{}px;--wl-navigation-width:{}px;--wl-icon-size-sm:{}px;--wl-icon-size-md:{}px;--wl-icon-size-lg:{}px;--wl-scrollbar-width:{}px;",
            number(metrics.control_height_sm),
            number(metrics.control_height_md),
            number(metrics.control_height_lg),
            number(metrics.content_padding),
            number(metrics.navigation_width),
            number(metrics.icon_size_sm),
            number(metrics.icon_size_md),
            number(metrics.icon_size_lg),
            number(metrics.scrollbar_width)
        ));
        css.push_str(&format!(
            "--wl-shadow-sm:{};--wl-shadow-md:{};--wl-shadow-lg:{};--wl-focus-ring-width:{}px;--wl-focus-ring-offset:{}px;--wl-motion-fast:{}ms;--wl-motion-normal:{}ms;--wl-motion-slow:{}ms;--wl-motion-enabled:{};",
            shadow_css(effects.shadow_sm),
            shadow_css(effects.shadow_md),
            shadow_css(effects.shadow_lg),
            number(effects.focus_ring_width),
            number(effects.focus_ring_offset),
            effects.motion_fast_ms,
            effects.motion_normal_ms,
            effects.motion_slow_ms,
            if effects.reduced_motion { "0" } else { "1" }
        ));
        css.push('}');
        if self.effects.reduced_motion {
            css.push_str(":root{--wl-motion-enabled:0;--wl-motion-fast:0ms;--wl-motion-normal:0ms;--wl-motion-slow:0ms;}");
        } else {
            css.push_str("@media(prefers-reduced-motion:reduce){:root{--wl-motion-enabled:0;--wl-motion-fast:0ms;--wl-motion-normal:0ms;--wl-motion-slow:0ms;}}");
        }
        Ok(css)
    }
}

impl Default for UiTheme {
    fn default() -> Self {
        Self::system()
    }
}

/// A four-sided value, suitable for padding and margin.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct UiEdges {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl UiEdges {
    pub const fn all(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    pub const fn xy(horizontal: f32, vertical: f32) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    pub const fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    fn validate(self, name: &str, allow_negative: bool) -> Result<(), UiError> {
        if [self.top, self.right, self.bottom, self.left]
            .iter()
            .any(|value| !value.is_finite() || (!allow_negative && *value < 0.0))
        {
            let range = if allow_negative {
                "finite"
            } else {
                "finite and non-negative"
            };
            return Err(UiError::InvalidStyle(format!(
                "{name} values must be {range}"
            )));
        }
        Ok(())
    }

    fn css(self) -> String {
        format!(
            "{}px {}px {}px {}px",
            number(self.top),
            number(self.right),
            number(self.bottom),
            number(self.left)
        )
    }
}

/// A width or height. Percentages are relative to the containing layout surface.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UiLength {
    Auto,
    Px(f32),
    Percent(f32),
}

impl UiLength {
    fn validate(self, name: &str) -> Result<(), UiError> {
        match self {
            Self::Auto => Ok(()),
            Self::Px(value) if finite_non_negative(value) => Ok(()),
            Self::Percent(value) if finite_non_negative(value) => Ok(()),
            _ => Err(UiError::InvalidStyle(format!(
                "{name} is outside its valid range"
            ))),
        }
    }

    fn validate_signed(self, name: &str) -> Result<(), UiError> {
        match self {
            Self::Auto => Ok(()),
            Self::Px(value) if value.is_finite() => Ok(()),
            Self::Percent(value) if value.is_finite() => Ok(()),
            _ => Err(UiError::InvalidStyle(format!("{name} must be finite"))),
        }
    }

    fn css(self) -> String {
        match self {
            Self::Auto => "auto".to_string(),
            Self::Px(value) => format!("{}px", number(value)),
            Self::Percent(value) => format!("{}%", number(value)),
        }
    }
}

/// CSS cursor behavior requested by a style rule.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UiCursor {
    Default,
    Pointer,
    Text,
    Crosshair,
    Move,
    Wait,
    NotAllowed,
}

impl UiCursor {
    fn css(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Pointer => "pointer",
            Self::Text => "text",
            Self::Crosshair => "crosshair",
            Self::Move => "move",
            Self::Wait => "wait",
            Self::NotAllowed => "not-allowed",
        }
    }
}

/// Text alignment shared by CSS and native renderers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UiTextAlign {
    Start,
    Center,
    End,
    Justify,
}

impl UiTextAlign {
    fn css(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Center => "center",
            Self::End => "end",
            Self::Justify => "justify",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UiBorderStyle {
    None,
    Solid,
    Dashed,
    Dotted,
}

impl UiBorderStyle {
    fn css(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Solid => "solid",
            Self::Dashed => "dashed",
            Self::Dotted => "dotted",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UiTextDecoration {
    None,
    Underline,
    LineThrough,
    Overline,
}

impl UiTextDecoration {
    fn css(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Underline => "underline",
            Self::LineThrough => "line-through",
            Self::Overline => "overline",
        }
    }
}

/// A color stop in a linear gradient, positioned from 0 to 100 percent.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiGradientStop {
    pub color: UiColor,
    pub position_percent: f32,
}
impl UiGradientStop {
    pub const fn new(color: UiColor, position_percent: f32) -> Self {
        Self {
            color,
            position_percent,
        }
    }
}

/// A typed linear gradient. Radial and image paints are renderer extensions.
#[derive(Clone, Debug, PartialEq)]
pub struct UiLinearGradient {
    angle_degrees: f32,
    stops: Vec<UiGradientStop>,
}
impl UiLinearGradient {
    pub fn new(angle_degrees: f32, stops: impl IntoIterator<Item = UiGradientStop>) -> Self {
        Self {
            angle_degrees,
            stops: stops.into_iter().collect(),
        }
    }
    pub fn angle_degrees(&self) -> f32 {
        self.angle_degrees
    }
    pub fn stops(&self) -> &[UiGradientStop] {
        &self.stops
    }
    fn validate(&self) -> Result<(), UiError> {
        if !self.angle_degrees.is_finite() || !(0.0..=360.0).contains(&self.angle_degrees) {
            return Err(UiError::InvalidStyle(
                "gradient angle must be between 0 and 360 degrees".to_string(),
            ));
        }
        if !(2..=16).contains(&self.stops.len()) {
            return Err(UiError::InvalidStyle(
                "a gradient must contain between 2 and 16 stops".to_string(),
            ));
        }
        let mut previous = -1.0;
        for stop in &self.stops {
            if !finite_non_negative(stop.position_percent)
                || stop.position_percent > 100.0
                || stop.position_percent < previous
            {
                return Err(UiError::InvalidStyle(
                    "gradient stops must be ordered between 0 and 100 percent".to_string(),
                ));
            }
            previous = stop.position_percent;
        }
        Ok(())
    }
    fn css(&self) -> String {
        let stops = self
            .stops
            .iter()
            .map(|stop| format!("{} {}%", stop.color.to_css(), number(stop.position_percent)))
            .collect::<Vec<_>>()
            .join(",");
        format!("linear-gradient({}deg,{stops})", number(self.angle_degrees))
    }
}

/// Background paint shared by CSS-capable renderers.
#[derive(Clone, Debug, PartialEq)]
pub enum UiBackground {
    Solid(UiColor),
    LinearGradient(UiLinearGradient),
}
impl From<UiColor> for UiBackground {
    fn from(value: UiColor) -> Self {
        Self::Solid(value)
    }
}

/// How content is distributed along the cross axis of a grid container.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAlignContent {
    Start,
    End,
    Center,
    Stretch,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}
impl UiAlignContent {
    fn css(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::End => "end",
            Self::Center => "center",
            Self::Stretch => "stretch",
            Self::SpaceBetween => "space-between",
            Self::SpaceAround => "space-around",
            Self::SpaceEvenly => "space-evenly",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiJustifyItems {
    Start,
    End,
    Center,
    Stretch,
}
impl UiJustifyItems {
    fn css(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::End => "end",
            Self::Center => "center",
            Self::Stretch => "stretch",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiFontStyle {
    Normal,
    Italic,
    Oblique,
}
impl UiFontStyle {
    fn css(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Italic => "italic",
            Self::Oblique => "oblique",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiTextOverflow {
    Clip,
    Ellipsis,
}
impl UiTextOverflow {
    fn css(self) -> &'static str {
        match self {
            Self::Clip => "clip",
            Self::Ellipsis => "ellipsis",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiCornerRadii {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_right: f32,
    pub bottom_left: f32,
}
impl UiCornerRadii {
    pub const fn uniform(value: f32) -> Self {
        Self {
            top_left: value,
            top_right: value,
            bottom_right: value,
            bottom_left: value,
        }
    }

    pub const fn new(top_left: f32, top_right: f32, bottom_right: f32, bottom_left: f32) -> Self {
        Self {
            top_left,
            top_right,
            bottom_right,
            bottom_left,
        }
    }

    fn validate(self) -> bool {
        [
            self.top_left,
            self.top_right,
            self.bottom_right,
            self.bottom_left,
        ]
        .iter()
        .all(|value| finite_non_negative(*value))
    }

    fn css(self) -> String {
        format!(
            "{}px {}px {}px {}px",
            number(self.top_left),
            number(self.top_right),
            number(self.bottom_right),
            number(self.bottom_left)
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UiGridTrack {
    Auto,
    Px(f32),
    Fraction(f32),
    MinContent,
    MaxContent,
}
impl UiGridTrack {
    fn validate(self) -> bool {
        match self {
            Self::Auto | Self::MinContent | Self::MaxContent => true,
            Self::Px(value) => finite_non_negative(value),
            Self::Fraction(value) => finite_positive(value),
        }
    }

    fn css(self) -> String {
        match self {
            Self::Auto => "auto".to_string(),
            Self::Px(value) => format!("{}px", number(value)),
            Self::Fraction(value) => format!("{}fr", number(value)),
            Self::MinContent => "min-content".to_string(),
            Self::MaxContent => "max-content".to_string(),
        }
    }
}

/// A CSS display mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiDisplay {
    Block,
    Inline,
    InlineBlock,
    Flex,
    InlineFlex,
    Grid,
    None,
}
impl UiDisplay {
    fn css(self) -> &'static str {
        match self {
            Self::Block => "block",
            Self::Inline => "inline",
            Self::InlineBlock => "inline-block",
            Self::Flex => "flex",
            Self::InlineFlex => "inline-flex",
            Self::Grid => "grid",
            Self::None => "none",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiPosition {
    Static,
    Relative,
    Absolute,
    Fixed,
    Sticky,
}
impl UiPosition {
    fn css(self) -> &'static str {
        match self {
            Self::Static => "static",
            Self::Relative => "relative",
            Self::Absolute => "absolute",
            Self::Fixed => "fixed",
            Self::Sticky => "sticky",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiOverflow {
    Visible,
    Hidden,
    Clip,
    Auto,
    Scroll,
}
impl UiOverflow {
    fn css(self) -> &'static str {
        match self {
            Self::Visible => "visible",
            Self::Hidden => "hidden",
            Self::Clip => "clip",
            Self::Auto => "auto",
            Self::Scroll => "scroll",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiFlexDirection {
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}
impl UiFlexDirection {
    fn css(self) -> &'static str {
        match self {
            Self::Row => "row",
            Self::RowReverse => "row-reverse",
            Self::Column => "column",
            Self::ColumnReverse => "column-reverse",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiFlexWrap {
    NoWrap,
    Wrap,
    WrapReverse,
}
impl UiFlexWrap {
    fn css(self) -> &'static str {
        match self {
            Self::NoWrap => "nowrap",
            Self::Wrap => "wrap",
            Self::WrapReverse => "wrap-reverse",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiJustifyContent {
    Start,
    End,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}
impl UiJustifyContent {
    fn css(self) -> &'static str {
        match self {
            Self::Start => "flex-start",
            Self::End => "flex-end",
            Self::Center => "center",
            Self::SpaceBetween => "space-between",
            Self::SpaceAround => "space-around",
            Self::SpaceEvenly => "space-evenly",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAlignItems {
    Start,
    End,
    Center,
    Stretch,
    Baseline,
}
impl UiAlignItems {
    fn css(self) -> &'static str {
        match self {
            Self::Start => "flex-start",
            Self::End => "flex-end",
            Self::Center => "center",
            Self::Stretch => "stretch",
            Self::Baseline => "baseline",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiBoxSizing {
    ContentBox,
    BorderBox,
}
impl UiBoxSizing {
    fn css(self) -> &'static str {
        match self {
            Self::ContentBox => "content-box",
            Self::BorderBox => "border-box",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiTextTransform {
    None,
    Uppercase,
    Lowercase,
    Capitalize,
}
impl UiTextTransform {
    fn css(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Uppercase => "uppercase",
            Self::Lowercase => "lowercase",
            Self::Capitalize => "capitalize",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiWhiteSpace {
    Normal,
    NoWrap,
    Pre,
    PreWrap,
    PreLine,
}
impl UiWhiteSpace {
    fn css(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::NoWrap => "nowrap",
            Self::Pre => "pre",
            Self::PreWrap => "pre-wrap",
            Self::PreLine => "pre-line",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiObjectFit {
    Fill,
    Contain,
    Cover,
    None,
    ScaleDown,
}
impl UiObjectFit {
    fn css(self) -> &'static str {
        match self {
            Self::Fill => "fill",
            Self::Contain => "contain",
            Self::Cover => "cover",
            Self::None => "none",
            Self::ScaleDown => "scale-down",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiTransitionProperty {
    All,
    Opacity,
    Transform,
    Color,
    Background,
    Border,
    Shadow,
}
impl UiTransitionProperty {
    fn css(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Opacity => "opacity",
            Self::Transform => "transform",
            Self::Color => "color",
            Self::Background => "background-color",
            Self::Border => "border-color",
            Self::Shadow => "box-shadow",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiEasing {
    Linear,
    Ease,
    EaseIn,
    EaseOut,
    EaseInOut,
}
impl UiEasing {
    fn css(self) -> &'static str {
        match self {
            Self::Linear => "linear",
            Self::Ease => "ease",
            Self::EaseIn => "ease-in",
            Self::EaseOut => "ease-out",
            Self::EaseInOut => "ease-in-out",
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiTransition {
    pub property: UiTransitionProperty,
    pub duration_ms: u32,
    pub delay_ms: u32,
    pub easing: UiEasing,
}
impl UiTransition {
    pub const fn new(property: UiTransitionProperty, duration_ms: u32, easing: UiEasing) -> Self {
        Self {
            property,
            duration_ms,
            delay_ms: 0,
            easing,
        }
    }
    pub const fn delay(mut self, delay_ms: u32) -> Self {
        self.delay_ms = delay_ms;
        self
    }
    fn css(self) -> String {
        format!(
            "{} {}ms {} {}ms",
            self.property.css(),
            self.duration_ms,
            self.easing.css(),
            self.delay_ms
        )
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAnimationIterations {
    Once,
    Count(u16),
    Infinite,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAnimationDirection {
    Normal,
    Reverse,
    Alternate,
    AlternateReverse,
}
impl UiAnimationDirection {
    fn css(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Reverse => "reverse",
            Self::Alternate => "alternate",
            Self::AlternateReverse => "alternate-reverse",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAnimationPlayState {
    Running,
    Paused,
}

impl UiAnimationPlayState {
    fn css(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Paused => "paused",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAnimationFillMode {
    None,
    Forwards,
    Backwards,
    Both,
}
impl UiAnimationFillMode {
    fn css(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Forwards => "forwards",
            Self::Backwards => "backwards",
            Self::Both => "both",
        }
    }
}

/// Declarative animation settings. This model contains no script or callbacks.
#[derive(Clone, Debug, PartialEq)]
pub struct UiAnimation {
    name: String,
    duration_ms: u32,
    delay_ms: i32,
    iterations: UiAnimationIterations,
    direction: UiAnimationDirection,
    fill_mode: UiAnimationFillMode,
    play_state: UiAnimationPlayState,
    easing: UiEasing,
}
impl UiAnimation {
    pub fn new(name: impl Into<String>, duration_ms: u32) -> Self {
        Self {
            name: name.into(),
            duration_ms,
            delay_ms: 0,
            iterations: UiAnimationIterations::Once,
            direction: UiAnimationDirection::Normal,
            fill_mode: UiAnimationFillMode::Both,
            play_state: UiAnimationPlayState::Running,
            easing: UiEasing::Ease,
        }
    }
    pub fn delay(mut self, delay_ms: i32) -> Self { self.delay_ms = delay_ms; self }
    pub fn iterations(mut self, value: UiAnimationIterations) -> Self { self.iterations = value; self }
    pub fn direction(mut self, value: UiAnimationDirection) -> Self { self.direction = value; self }
    pub fn fill_mode(mut self, value: UiAnimationFillMode) -> Self { self.fill_mode = value; self }
    pub fn play_state(mut self, value: UiAnimationPlayState) -> Self { self.play_state = value; self }
    pub fn easing(mut self, value: UiEasing) -> Self { self.easing = value; self }
    pub fn name(&self) -> &str { &self.name }
    pub fn duration_ms(&self) -> u32 { self.duration_ms }
    pub fn delay_ms(&self) -> i32 { self.delay_ms }
    fn validate(&self) -> Result<(), UiError> {
        if !valid_identifier(&self.name) {
            return Err(UiError::InvalidStyle("animation name must be a valid identifier".to_string()));
        }
        if self.duration_ms > 3_600_000 || self.delay_ms.unsigned_abs() > 3_600_000 {
            return Err(UiError::InvalidStyle("animation duration and delay must not exceed one hour".to_string()));
        }
        if let UiAnimationIterations::Count(count) = self.iterations {
            if count == 0 || count > 1000 {
                return Err(UiError::InvalidStyle("animation iteration count must be 1 to 1000 or infinite".to_string()));
            }
        }
        Ok(())
    }
    fn css(&self) -> String {
        let iterations = match self.iterations {
            UiAnimationIterations::Once => "1".to_string(),
            UiAnimationIterations::Count(count) => count.to_string(),
            UiAnimationIterations::Infinite => "infinite".to_string(),
        };
        format!(
            "{} calc({}ms * var(--wl-motion-enabled,1)) {} calc({}ms * var(--wl-motion-enabled,1)) {} {} {} {}",
            self.name, self.duration_ms, self.easing.css(), self.delay_ms,
            iterations, self.direction.css(), self.fill_mode.css(), self.play_state.css()
        )
    }
}

/// One typed keyframe; offsets are percentages from 0 through 100.
#[derive(Clone, Debug, PartialEq)]
pub struct UiKeyframe {
    offset_percent: f32,
    style: UiStyle,
}
impl UiKeyframe {
    pub fn new(offset_percent: f32, style: UiStyle) -> Self { Self { offset_percent, style } }
    pub fn offset_percent(&self) -> f32 { self.offset_percent }
    pub fn style(&self) -> &UiStyle { &self.style }
}

/// Named, bounded keyframe definitions for CSS-capable renderers.
#[derive(Clone, Debug, PartialEq)]
pub struct UiKeyframes {
    name: String,
    frames: Vec<UiKeyframe>,
}
impl UiKeyframes {
    pub fn new(name: impl Into<String>, frames: impl IntoIterator<Item = UiKeyframe>) -> Self {
        Self { name: name.into(), frames: frames.into_iter().collect() }
    }
    pub fn name(&self) -> &str { &self.name }
    pub fn frames(&self) -> &[UiKeyframe] { &self.frames }
    fn validate(&self) -> Result<(), UiError> {
        if !valid_identifier(&self.name) {
            return Err(UiError::InvalidStyle("keyframe name must be a valid identifier".to_string()));
        }
        if !(2..=64).contains(&self.frames.len()) {
            return Err(UiError::InvalidStyle("keyframes must contain 2 to 64 frames".to_string()));
        }
        let mut previous = -1.0_f32;
        for frame in &self.frames {
            if !frame.offset_percent.is_finite()
                || !(0.0..=100.0).contains(&frame.offset_percent)
                || frame.offset_percent <= previous
            {
                return Err(UiError::InvalidStyle("keyframe offsets must be strictly increasing from 0 to 100".to_string()));
            }
            frame.style.validate()?;
            if !frame.style.transitions.is_empty() || !frame.style.animations.is_empty() {
                return Err(UiError::InvalidStyle("keyframe declarations cannot nest transitions or animations".to_string()));
            }
            previous = frame.offset_percent;
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiTransform {
    pub translate_x: f32,
    pub translate_y: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub rotate_degrees: f32,
}
impl UiTransform {
    pub const fn identity() -> Self {
        Self {
            translate_x: 0.0,
            translate_y: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            rotate_degrees: 0.0,
        }
    }
    pub const fn translate(mut self, x: f32, y: f32) -> Self {
        self.translate_x = x;
        self.translate_y = y;
        self
    }
    pub const fn scale(mut self, x: f32, y: f32) -> Self {
        self.scale_x = x;
        self.scale_y = y;
        self
    }
    pub const fn rotate(mut self, degrees: f32) -> Self {
        self.rotate_degrees = degrees;
        self
    }
    fn validate(self) -> bool {
        [
            self.translate_x,
            self.translate_y,
            self.scale_x,
            self.scale_y,
            self.rotate_degrees,
        ]
        .iter()
        .all(|value| value.is_finite())
    }
    fn css(self) -> String {
        format!(
            "translate({}px,{}px) rotate({}deg) scale({},{})",
            number(self.translate_x),
            number(self.translate_y),
            number(self.rotate_degrees),
            number(self.scale_x),
            number(self.scale_y)
        )
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UiMediaCondition {
    MinWidth(f32),
    MaxWidth(f32),
    LightScheme,
    DarkScheme,
    ReducedMotion,
    ForcedColors,
}
impl UiMediaCondition {
    fn validate(self) -> Result<(), UiError> {
        match self {
            Self::MinWidth(value) | Self::MaxWidth(value) if finite_non_negative(value) => Ok(()),
            Self::MinWidth(_) | Self::MaxWidth(_) => Err(UiError::InvalidStyle(
                "media width must be finite and non-negative".to_string(),
            )),
            _ => Ok(()),
        }
    }
    fn css(self) -> String {
        match self {
            Self::MinWidth(value) => format!("(min-width:{}px)", number(value)),
            Self::MaxWidth(value) => format!("(max-width:{}px)", number(value)),
            Self::LightScheme => "(prefers-color-scheme:light)".to_string(),
            Self::DarkScheme => "(prefers-color-scheme:dark)".to_string(),
            Self::ReducedMotion => "(prefers-reduced-motion:reduce)".to_string(),
            Self::ForcedColors => "(forced-colors:active)".to_string(),
        }
    }
}

/// A reusable style declaration. Unset properties inherit from the theme or renderer defaults.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct UiStyle {
    background: Option<UiBackground>,
    foreground: Option<UiColor>,
    border_color: Option<UiColor>,
    font_family: Option<String>,
    font_size: Option<f32>,
    font_weight: Option<u16>,
    line_height: Option<f32>,
    letter_spacing: Option<f32>,
    padding: Option<UiEdges>,
    margin: Option<UiEdges>,
    gap: Option<f32>,
    width: Option<UiLength>,
    height: Option<UiLength>,
    min_width: Option<UiLength>,
    min_height: Option<UiLength>,
    max_width: Option<UiLength>,
    max_height: Option<UiLength>,
    border_width: Option<f32>,
    border_radius: Option<f32>,
    border_style: Option<UiBorderStyle>,
    opacity: Option<f32>,
    shadow: Option<UiShadow>,
    cursor: Option<UiCursor>,
    text_align: Option<UiTextAlign>,
    text_decoration: Option<UiTextDecoration>,
    outline_color: Option<UiColor>,
    outline_width: Option<f32>,
    outline_offset: Option<f32>,
    display: Option<UiDisplay>,
    position: Option<UiPosition>,
    overflow: Option<UiOverflow>,
    flex_direction: Option<UiFlexDirection>,
    flex_wrap: Option<UiFlexWrap>,
    justify_content: Option<UiJustifyContent>,
    align_items: Option<UiAlignItems>,
    box_sizing: Option<UiBoxSizing>,
    text_transform: Option<UiTextTransform>,
    white_space: Option<UiWhiteSpace>,
    object_fit: Option<UiObjectFit>,
    flex_grow: Option<f32>,
    flex_shrink: Option<f32>,
    z_index: Option<i32>,
    top: Option<UiLength>,
    right: Option<UiLength>,
    bottom: Option<UiLength>,
    left: Option<UiLength>,
    transform: Option<UiTransform>,
    transitions: Vec<UiTransition>,
    animations: Vec<UiAnimation>,
    grid_columns: Option<Vec<UiGridTrack>>,
    grid_rows: Option<Vec<UiGridTrack>>,
    column_gap: Option<f32>,
    row_gap: Option<f32>,
    align_content: Option<UiAlignContent>,
    justify_items: Option<UiJustifyItems>,
    font_style: Option<UiFontStyle>,
    text_overflow: Option<UiTextOverflow>,
    aspect_ratio: Option<f32>,
    corner_radii: Option<UiCornerRadii>,
    text_shadow: Option<UiShadow>,
}

impl UiStyle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn background(mut self, color: UiColor) -> Self {
        self.background = Some(UiBackground::Solid(color));
        self
    }

    pub fn background_paint(mut self, background: UiBackground) -> Self {
        self.background = Some(background);
        self
    }

    pub fn linear_gradient(mut self, gradient: UiLinearGradient) -> Self {
        self.background = Some(UiBackground::LinearGradient(gradient));
        self
    }

    pub fn foreground(mut self, color: UiColor) -> Self {
        self.foreground = Some(color);
        self
    }

    pub fn border_color(mut self, color: UiColor) -> Self {
        self.border_color = Some(color);
        self
    }

    pub fn font_family(mut self, family: impl Into<String>) -> Self {
        self.font_family = Some(family.into());
        self
    }

    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = Some(size);
        self
    }

    pub fn font_weight(mut self, weight: u16) -> Self {
        self.font_weight = Some(weight);
        self
    }

    pub fn line_height(mut self, value: f32) -> Self {
        self.line_height = Some(value);
        self
    }

    pub fn letter_spacing(mut self, value: f32) -> Self {
        self.letter_spacing = Some(value);
        self
    }

    pub fn padding(mut self, value: UiEdges) -> Self {
        self.padding = Some(value);
        self
    }

    pub fn margin(mut self, value: UiEdges) -> Self {
        self.margin = Some(value);
        self
    }

    pub fn gap(mut self, value: f32) -> Self {
        self.gap = Some(value);
        self
    }

    pub fn width(mut self, value: UiLength) -> Self {
        self.width = Some(value);
        self
    }

    pub fn height(mut self, value: UiLength) -> Self {
        self.height = Some(value);
        self
    }

    pub fn min_width(mut self, value: UiLength) -> Self {
        self.min_width = Some(value);
        self
    }

    pub fn min_height(mut self, value: UiLength) -> Self {
        self.min_height = Some(value);
        self
    }

    pub fn max_width(mut self, value: UiLength) -> Self {
        self.max_width = Some(value);
        self
    }

    pub fn max_height(mut self, value: UiLength) -> Self {
        self.max_height = Some(value);
        self
    }

    pub fn border(mut self, width: f32, style: UiBorderStyle, color: UiColor) -> Self {
        self.border_width = Some(width);
        self.border_style = Some(style);
        self.border_color = Some(color);
        self
    }

    pub fn border_radius(mut self, value: f32) -> Self {
        self.border_radius = Some(value);
        self
    }

    pub fn opacity(mut self, value: f32) -> Self {
        self.opacity = Some(value);
        self
    }

    pub fn shadow(mut self, value: UiShadow) -> Self {
        self.shadow = Some(value);
        self
    }

    pub fn cursor(mut self, value: UiCursor) -> Self {
        self.cursor = Some(value);
        self
    }

    pub fn text_align(mut self, value: UiTextAlign) -> Self {
        self.text_align = Some(value);
        self
    }

    pub fn text_decoration(mut self, value: UiTextDecoration) -> Self {
        self.text_decoration = Some(value);
        self
    }

    pub fn display(mut self, value: UiDisplay) -> Self {
        self.display = Some(value);
        self
    }
    pub fn position(mut self, value: UiPosition) -> Self {
        self.position = Some(value);
        self
    }
    pub fn overflow(mut self, value: UiOverflow) -> Self {
        self.overflow = Some(value);
        self
    }
    pub fn flex_direction(mut self, value: UiFlexDirection) -> Self {
        self.flex_direction = Some(value);
        self
    }
    pub fn flex_wrap(mut self, value: UiFlexWrap) -> Self {
        self.flex_wrap = Some(value);
        self
    }
    pub fn justify_content(mut self, value: UiJustifyContent) -> Self {
        self.justify_content = Some(value);
        self
    }
    pub fn align_items(mut self, value: UiAlignItems) -> Self {
        self.align_items = Some(value);
        self
    }
    pub fn box_sizing(mut self, value: UiBoxSizing) -> Self {
        self.box_sizing = Some(value);
        self
    }
    pub fn text_transform(mut self, value: UiTextTransform) -> Self {
        self.text_transform = Some(value);
        self
    }
    pub fn white_space(mut self, value: UiWhiteSpace) -> Self {
        self.white_space = Some(value);
        self
    }
    pub fn object_fit(mut self, value: UiObjectFit) -> Self {
        self.object_fit = Some(value);
        self
    }
    pub fn flex_grow(mut self, value: f32) -> Self {
        self.flex_grow = Some(value);
        self
    }
    pub fn flex_shrink(mut self, value: f32) -> Self {
        self.flex_shrink = Some(value);
        self
    }
    pub fn z_index(mut self, value: i32) -> Self {
        self.z_index = Some(value);
        self
    }
    pub fn top(mut self, value: UiLength) -> Self {
        self.top = Some(value);
        self
    }
    pub fn right(mut self, value: UiLength) -> Self {
        self.right = Some(value);
        self
    }
    pub fn bottom(mut self, value: UiLength) -> Self {
        self.bottom = Some(value);
        self
    }
    pub fn left(mut self, value: UiLength) -> Self {
        self.left = Some(value);
        self
    }
    pub fn transform(mut self, value: UiTransform) -> Self {
        self.transform = Some(value);
        self
    }
    pub fn transition(mut self, value: UiTransition) -> Self {
        self.transitions.push(value);
        self
    }

    pub fn animation(mut self, value: UiAnimation) -> Self {
        self.animations.push(value);
        self
    }
    pub fn grid_template(
        mut self,
        columns: impl IntoIterator<Item = UiGridTrack>,
        rows: impl IntoIterator<Item = UiGridTrack>,
    ) -> Self {
        self.grid_columns = Some(columns.into_iter().collect());
        self.grid_rows = Some(rows.into_iter().collect());
        self
    }

    pub fn grid_columns(mut self, columns: impl IntoIterator<Item = UiGridTrack>) -> Self {
        self.grid_columns = Some(columns.into_iter().collect());
        self
    }

    pub fn grid_rows(mut self, rows: impl IntoIterator<Item = UiGridTrack>) -> Self {
        self.grid_rows = Some(rows.into_iter().collect());
        self
    }

    pub fn grid_gap(mut self, row: f32, column: f32) -> Self {
        self.row_gap = Some(row);
        self.column_gap = Some(column);
        self
    }

    pub fn align_content(mut self, value: UiAlignContent) -> Self {
        self.align_content = Some(value);
        self
    }

    pub fn justify_items(mut self, value: UiJustifyItems) -> Self {
        self.justify_items = Some(value);
        self
    }

    pub fn font_style(mut self, value: UiFontStyle) -> Self {
        self.font_style = Some(value);
        self
    }

    pub fn text_overflow(mut self, value: UiTextOverflow) -> Self {
        self.text_overflow = Some(value);
        self
    }

    pub fn aspect_ratio(mut self, value: f32) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn corner_radii(mut self, value: UiCornerRadii) -> Self {
        self.corner_radii = Some(value);
        self
    }

    pub fn text_shadow(mut self, value: UiShadow) -> Self {
        self.text_shadow = Some(value);
        self
    }

    pub fn background_color(&self) -> Option<UiColor> {
        match &self.background {
            Some(UiBackground::Solid(color)) => Some(*color),
            _ => None,
        }
    }
    pub fn foreground_color(&self) -> Option<UiColor> {
        self.foreground
    }
    pub fn border_color_value(&self) -> Option<UiColor> {
        self.border_color
    }
    pub fn font_family_value(&self) -> Option<&str> {
        self.font_family.as_deref()
    }
    pub fn font_size_value(&self) -> Option<f32> {
        self.font_size
    }
    pub fn font_weight_value(&self) -> Option<u16> {
        self.font_weight
    }
    pub fn border_width_value(&self) -> Option<f32> {
        self.border_width
    }
    pub fn padding_value(&self) -> Option<UiEdges> {
        self.padding
    }

    /// Returns the configured background paint, including gradients.
    pub fn background_value(&self) -> Option<&UiBackground> {
        self.background.as_ref()
    }
    pub fn margin_value(&self) -> Option<UiEdges> {
        self.margin
    }
    pub fn gap_value(&self) -> Option<f32> {
        self.gap
    }
    pub fn width_value(&self) -> Option<UiLength> {
        self.width
    }
    pub fn height_value(&self) -> Option<UiLength> {
        self.height
    }
    pub fn min_width_value(&self) -> Option<UiLength> {
        self.min_width
    }
    pub fn min_height_value(&self) -> Option<UiLength> {
        self.min_height
    }
    pub fn max_width_value(&self) -> Option<UiLength> {
        self.max_width
    }
    pub fn max_height_value(&self) -> Option<UiLength> {
        self.max_height
    }
    pub fn border_style_value(&self) -> Option<UiBorderStyle> {
        self.border_style
    }
    pub fn border_radius_value(&self) -> Option<f32> {
        self.border_radius
    }
    pub fn corner_radii_value(&self) -> Option<UiCornerRadii> {
        self.corner_radii
    }
    pub fn opacity_value(&self) -> Option<f32> {
        self.opacity
    }
    pub fn shadow_value(&self) -> Option<UiShadow> {
        self.shadow
    }
    pub fn text_shadow_value(&self) -> Option<UiShadow> {
        self.text_shadow
    }
    pub fn cursor_value(&self) -> Option<UiCursor> {
        self.cursor
    }
    pub fn text_align_value(&self) -> Option<UiTextAlign> {
        self.text_align
    }
    pub fn text_decoration_value(&self) -> Option<UiTextDecoration> {
        self.text_decoration
    }
    pub fn outline_color_value(&self) -> Option<UiColor> {
        self.outline_color
    }
    pub fn outline_width_value(&self) -> Option<f32> {
        self.outline_width
    }
    pub fn outline_offset_value(&self) -> Option<f32> {
        self.outline_offset
    }
    pub fn display_value(&self) -> Option<UiDisplay> {
        self.display
    }
    pub fn position_value(&self) -> Option<UiPosition> {
        self.position
    }
    pub fn overflow_value(&self) -> Option<UiOverflow> {
        self.overflow
    }
    pub fn flex_direction_value(&self) -> Option<UiFlexDirection> {
        self.flex_direction
    }
    pub fn flex_wrap_value(&self) -> Option<UiFlexWrap> {
        self.flex_wrap
    }
    pub fn justify_content_value(&self) -> Option<UiJustifyContent> {
        self.justify_content
    }
    pub fn align_items_value(&self) -> Option<UiAlignItems> {
        self.align_items
    }
    pub fn box_sizing_value(&self) -> Option<UiBoxSizing> {
        self.box_sizing
    }
    pub fn text_transform_value(&self) -> Option<UiTextTransform> {
        self.text_transform
    }
    pub fn white_space_value(&self) -> Option<UiWhiteSpace> {
        self.white_space
    }
    pub fn object_fit_value(&self) -> Option<UiObjectFit> {
        self.object_fit
    }
    pub fn flex_grow_value(&self) -> Option<f32> {
        self.flex_grow
    }
    pub fn flex_shrink_value(&self) -> Option<f32> {
        self.flex_shrink
    }
    pub fn z_index_value(&self) -> Option<i32> {
        self.z_index
    }
    pub fn top_value(&self) -> Option<UiLength> {
        self.top
    }
    pub fn right_value(&self) -> Option<UiLength> {
        self.right
    }
    pub fn bottom_value(&self) -> Option<UiLength> {
        self.bottom
    }
    pub fn left_value(&self) -> Option<UiLength> {
        self.left
    }
    pub fn transform_value(&self) -> Option<UiTransform> {
        self.transform
    }
    pub fn transitions_value(&self) -> &[UiTransition] {
        &self.transitions
    }
    pub fn animations_value(&self) -> &[UiAnimation] {
        &self.animations
    }
    pub fn grid_columns_value(&self) -> Option<&[UiGridTrack]> {
        self.grid_columns.as_deref()
    }
    pub fn grid_rows_value(&self) -> Option<&[UiGridTrack]> {
        self.grid_rows.as_deref()
    }
    pub fn row_gap_value(&self) -> Option<f32> {
        self.row_gap
    }
    pub fn column_gap_value(&self) -> Option<f32> {
        self.column_gap
    }
    pub fn align_content_value(&self) -> Option<UiAlignContent> {
        self.align_content
    }
    pub fn justify_items_value(&self) -> Option<UiJustifyItems> {
        self.justify_items
    }
    pub fn font_style_value(&self) -> Option<UiFontStyle> {
        self.font_style
    }
    pub fn text_overflow_value(&self) -> Option<UiTextOverflow> {
        self.text_overflow
    }
    pub fn aspect_ratio_value(&self) -> Option<f32> {
        self.aspect_ratio
    }

    pub fn outline(mut self, width: f32, offset: f32, color: UiColor) -> Self {
        self.outline_width = Some(width);
        self.outline_offset = Some(offset);
        self.outline_color = Some(color);
        self
    }

    pub fn validate(&self) -> Result<(), UiError> {
        if self
            .font_family
            .as_ref()
            .is_some_and(|family| !valid_font_family(family))
        {
            return Err(UiError::InvalidStyle(
                "font family contains unsupported CSS characters".to_string(),
            ));
        }
        if let Some(UiBackground::LinearGradient(gradient)) = &self.background {
            gradient.validate()?;
        }
        for (name, value) in [
            ("font size", self.font_size),
            ("line height", self.line_height),
            ("border width", self.border_width),
            ("border radius", self.border_radius),
            ("gap", self.gap),
            ("outline width", self.outline_width),
        ] {
            if value.is_some_and(|value| !finite_non_negative(value)) {
                return Err(UiError::InvalidStyle(format!(
                    "{name} must be finite and non-negative"
                )));
            }
        }
        if self.outline_offset.is_some_and(|value| !value.is_finite()) {
            return Err(UiError::InvalidStyle(
                "outline offset must be finite".to_string(),
            ));
        }
        if self.font_size.is_some_and(|value| value == 0.0)
            || self.line_height.is_some_and(|value| value == 0.0)
        {
            return Err(UiError::InvalidStyle(
                "font size and line height must be greater than zero".to_string(),
            ));
        }
        if self
            .font_weight
            .is_some_and(|value| value == 0 || value > 1000)
        {
            return Err(UiError::InvalidStyle(
                "font weight must be between 1 and 1000".to_string(),
            ));
        }
        if self.letter_spacing.is_some_and(|value| !value.is_finite()) {
            return Err(UiError::InvalidStyle(
                "letter spacing must be finite".to_string(),
            ));
        }
        if self
            .opacity
            .is_some_and(|value| !value.is_finite() || !(0.0..=1.0).contains(&value))
        {
            return Err(UiError::InvalidStyle(
                "opacity must be between 0 and 1".to_string(),
            ));
        }
        for (name, value) in [
            ("flex grow", self.flex_grow),
            ("flex shrink", self.flex_shrink),
        ] {
            if value.is_some_and(|value| !finite_non_negative(value)) {
                return Err(UiError::InvalidStyle(format!(
                    "{name} must be finite and non-negative"
                )));
            }
        }
        if self.animations.len() > 8 {
            return Err(UiError::InvalidStyle("a component cannot have more than 8 simultaneous animations".to_string()));
        }
        for animation in &self.animations {
            animation.validate()?;
        }
        if self.transform.is_some_and(|value| !value.validate()) {
            return Err(UiError::InvalidStyle(
                "transform values must be finite".to_string(),
            ));
        }
        for (name, tracks) in [
            ("grid columns", self.grid_columns.as_ref()),
            ("grid rows", self.grid_rows.as_ref()),
        ] {
            if let Some(tracks) = tracks {
                if !(1..=12).contains(&tracks.len())
                    || !tracks.iter().copied().all(UiGridTrack::validate)
                {
                    return Err(UiError::InvalidStyle(format!(
                        "{name} must contain 1 to 12 valid tracks"
                    )));
                }
            }
        }
        for (name, value) in [("column gap", self.column_gap), ("row gap", self.row_gap)] {
            if value.is_some_and(|value| !finite_non_negative(value)) {
                return Err(UiError::InvalidStyle(format!(
                    "{name} must be finite and non-negative"
                )));
            }
        }
        if self
            .aspect_ratio
            .is_some_and(|value| !finite_positive(value))
        {
            return Err(UiError::InvalidStyle(
                "aspect ratio must be finite and positive".to_string(),
            ));
        }
        if self.corner_radii.is_some_and(|value| !value.validate()) {
            return Err(UiError::InvalidStyle(
                "corner radii must be finite and non-negative".to_string(),
            ));
        }
        if self.text_shadow.is_some_and(|value| !valid_text_shadow(value)) {
            return Err(UiError::InvalidStyle(
                "text shadow offsets must be finite, blur non-negative, and spread zero".to_string(),
            ));
        }
        if let Some(value) = self.padding {
            value.validate("padding", false)?;
        }
        if let Some(value) = self.margin {
            value.validate("margin", true)?;
        }
        for (name, value) in [
            ("top", self.top),
            ("right", self.right),
            ("bottom", self.bottom),
            ("left", self.left),
        ] {
            if let Some(value) = value {
                value.validate_signed(name)?;
            }
        }
        for (name, value) in [
            ("width", self.width),
            ("height", self.height),
            ("min-width", self.min_width),
            ("min-height", self.min_height),
            ("max-width", self.max_width),
            ("max-height", self.max_height),
        ] {
            if let Some(value) = value {
                value.validate(name)?;
            }
        }
        if let Some(value) = self.shadow {
            if !valid_shadow(value) {
                return Err(UiError::InvalidStyle(
                    "shadow values must be finite and blur non-negative".to_string(),
                ));
            }
        }
        Ok(())
    }

    fn css_declarations(&self) -> Result<String, UiError> {
        self.validate()?;
        let mut css = String::new();
        macro_rules! push {
            ($name:literal, $value:expr) => {{
                css.push_str($name);
                css.push(':');
                append_css_value(&mut css, $value);
                css.push(';');
            }};
        }
        if let Some(value) = &self.background {
            match value {
                UiBackground::Solid(color) => push!("background-color", color.to_css()),
                UiBackground::LinearGradient(gradient) => {
                    gradient.validate()?;
                    push!("background-image", gradient.css());
                }
            }
        }
        if let Some(value) = self.foreground {
            push!("color", value.to_css());
        }
        if let Some(value) = self.border_color {
            push!("border-color", value.to_css());
        }
        if let Some(value) = &self.font_family {
            push!("font-family", css_font_family(value));
        }
        if let Some(value) = self.font_size {
            push!("font-size", format!("{}px", number(value)));
        }
        if let Some(value) = self.font_weight {
            push!("font-weight", value.to_string());
        }
        if let Some(value) = self.line_height {
            push!("line-height", number(value));
        }
        if let Some(value) = self.letter_spacing {
            push!("letter-spacing", format!("{}px", number(value)));
        }
        if let Some(value) = self.padding {
            push!("padding", value.css());
        }
        if let Some(value) = self.margin {
            push!("margin", value.css());
        }
        if let Some(value) = self.gap {
            push!("gap", format!("{}px", number(value)));
        }
        if let Some(value) = self.width {
            push!("width", value.css());
        }
        if let Some(value) = self.height {
            push!("height", value.css());
        }
        if let Some(value) = self.min_width {
            push!("min-width", value.css());
        }
        if let Some(value) = self.min_height {
            push!("min-height", value.css());
        }
        if let Some(value) = self.max_width {
            push!("max-width", value.css());
        }
        if let Some(value) = self.max_height {
            push!("max-height", value.css());
        }
        if let Some(value) = self.border_width {
            push!("border-width", format!("{}px", number(value)));
        }
        if let Some(value) = self.border_style {
            push!("border-style", value.css());
        }
        if let Some(value) = self.border_radius {
            push!("border-radius", format!("{}px", number(value)));
        }
        if let Some(value) = self.opacity {
            push!("opacity", number(value));
        }
        if let Some(value) = self.shadow {
            push!("box-shadow", shadow_css(value));
        }
        if let Some(value) = self.cursor {
            push!("cursor", value.css());
        }
        if let Some(value) = self.text_align {
            push!("text-align", value.css());
        }
        if let Some(value) = self.text_decoration {
            push!("text-decoration", value.css());
        }
        if self.outline_width.is_some() || self.outline_color.is_some() {
            let width = self.outline_width.unwrap_or(1.0);
            let color = self.outline_color.unwrap_or(UiColor::rgb(0, 103, 192));
            push!(
                "outline",
                format!("{}px solid {}", number(width), color.to_css())
            );
        }
        if let Some(value) = self.outline_offset {
            push!("outline-offset", format!("{}px", number(value)));
        }
        if let Some(value) = self.display {
            push!("display", value.css());
        }
        if let Some(value) = self.position {
            push!("position", value.css());
        }
        if let Some(value) = self.overflow {
            push!("overflow", value.css());
        }
        if let Some(value) = self.flex_direction {
            push!("flex-direction", value.css());
        }
        if let Some(value) = self.flex_wrap {
            push!("flex-wrap", value.css());
        }
        if let Some(value) = self.justify_content {
            push!("justify-content", value.css());
        }
        if let Some(value) = self.align_items {
            push!("align-items", value.css());
        }
        if let Some(value) = self.box_sizing {
            push!("box-sizing", value.css());
        }
        if let Some(value) = self.text_transform {
            push!("text-transform", value.css());
        }
        if let Some(value) = self.white_space {
            push!("white-space", value.css());
        }
        if let Some(value) = self.object_fit {
            push!("object-fit", value.css());
        }
        if let Some(value) = self.flex_grow {
            push!("flex-grow", number(value));
        }
        if let Some(value) = self.flex_shrink {
            push!("flex-shrink", number(value));
        }
        if let Some(value) = self.z_index {
            push!("z-index", value.to_string());
        }
        if let Some(value) = self.top {
            push!("top", value.css());
        }
        if let Some(value) = self.right {
            push!("right", value.css());
        }
        if let Some(value) = self.bottom {
            push!("bottom", value.css());
        }
        if let Some(value) = self.left {
            push!("left", value.css());
        }
        if let Some(value) = self.transform {
            push!("transform", value.css());
        }
        if !self.transitions.is_empty() {
            push!(
                "transition",
                self.transitions
                    .iter()
                    .copied()
                    .map(UiTransition::css)
                    .collect::<Vec<_>>()
                    .join(",")
            );
        }
        if !self.animations.is_empty() {
            push!(
                "animation",
                self.animations.iter().map(UiAnimation::css).collect::<Vec<_>>().join(",")
            );
        }
        if let Some(value) = &self.grid_columns {
            push!(
                "grid-template-columns",
                value
                    .iter()
                    .copied()
                    .map(UiGridTrack::css)
                    .collect::<Vec<_>>()
                    .join(" ")
            );
        }
        if let Some(value) = &self.grid_rows {
            push!(
                "grid-template-rows",
                value
                    .iter()
                    .copied()
                    .map(UiGridTrack::css)
                    .collect::<Vec<_>>()
                    .join(" ")
            );
        }
        if let Some(value) = self.column_gap {
            push!("column-gap", format!("{}px", number(value)));
        }
        if let Some(value) = self.row_gap {
            push!("row-gap", format!("{}px", number(value)));
        }
        if let Some(value) = self.align_content {
            push!("align-content", value.css());
        }
        if let Some(value) = self.justify_items {
            push!("justify-items", value.css());
        }
        if let Some(value) = self.font_style {
            push!("font-style", value.css());
        }
        if let Some(value) = self.text_overflow {
            push!("text-overflow", value.css());
        }
        if let Some(value) = self.aspect_ratio {
            push!("aspect-ratio", number(value));
        }
        if let Some(value) = self.corner_radii {
            push!("border-radius", value.css());
        }
        if let Some(value) = self.text_shadow {
            push!(
                "text-shadow",
                format!(
                    "{}px {}px {}px {}",
                    number(value.offset_x),
                    number(value.offset_y),
                    number(value.blur),
                    value.color.to_css()
                )
            );
        }
        Ok(css)
    }
}

/// Semantic element kinds understood by style selectors and CSS renderers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiComponent {
    Window,
    Surface,
    Card,
    Text,
    Heading,
    Button,
    TextInput,
    Select,
    Checkbox,
    Radio,
    Switch,
    Slider,
    Navigation,
    NavigationItem,
    Dialog,
    Menu,
    MenuItem,
    Tooltip,
    Badge,
    Progress,
    Divider,
    List,
    ListItem,
    Image,
    Icon,
    Custom(String),
}

impl UiComponent {
    fn as_str(&self) -> &str {
        match self {
            Self::Window => "window",
            Self::Surface => "surface",
            Self::Card => "card",
            Self::Text => "text",
            Self::Heading => "heading",
            Self::Button => "button",
            Self::TextInput => "text-input",
            Self::Select => "select",
            Self::Checkbox => "checkbox",
            Self::Radio => "radio",
            Self::Switch => "switch",
            Self::Slider => "slider",
            Self::Navigation => "navigation",
            Self::NavigationItem => "navigation-item",
            Self::Dialog => "dialog",
            Self::Menu => "menu",
            Self::MenuItem => "menu-item",
            Self::Tooltip => "tooltip",
            Self::Badge => "badge",
            Self::Progress => "progress",
            Self::Divider => "divider",
            Self::List => "list",
            Self::ListItem => "list-item",
            Self::Image => "image",
            Self::Icon => "icon",
            Self::Custom(value) => value,
        }
    }
}

/// Interactive states can be targeted without depending on a renderer's native type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiVisualState {
    Hover,
    Pressed,
    Focused,
    Disabled,
    Selected,
    Checked,
    Invalid,
    Loading,
}

impl UiVisualState {
    fn css(self) -> &'static str {
        match self {
            Self::Hover => ":hover",
            Self::Pressed => ":active",
            Self::Focused => ":focus-visible",
            Self::Disabled => "[aria-disabled=\"true\"]",
            Self::Selected => "[aria-selected=\"true\"]",
            Self::Checked => "[aria-checked=\"true\"]",
            Self::Invalid => ":invalid",
            Self::Loading => "[aria-busy=\"true\"]",
        }
    }
}

/// A CSS-like selector that remains meaningful to native renderers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiSelector {
    component: UiComponent,
    class: Option<String>,
    id: Option<String>,
    state: Option<UiVisualState>,
}

impl UiSelector {
    pub fn new(component: UiComponent) -> Self {
        Self {
            component,
            class: None,
            id: None,
            state: None,
        }
    }

    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.class = Some(class.into());
        self
    }

    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn state(mut self, state: UiVisualState) -> Self {
        self.state = Some(state);
        self
    }

    pub fn component(&self) -> &UiComponent {
        &self.component
    }

    pub fn class_name(&self) -> Option<&str> {
        self.class.as_deref()
    }

    pub fn id_name(&self) -> Option<&str> {
        self.id.as_deref()
    }

    pub fn visual_state(&self) -> Option<UiVisualState> {
        self.state
    }

    fn css(&self) -> Result<String, UiError> {
        let component = self.component.as_str();
        if !valid_identifier(component) {
            return Err(UiError::InvalidStyle(
                "component selector must be a valid identifier".to_string(),
            ));
        }
        let mut selector = format!("[data-wl-component=\"{}\"]", component);
        if let Some(class) = &self.class {
            if !valid_identifier(class) {
                return Err(UiError::InvalidStyle(
                    "style class must be a valid CSS identifier".to_string(),
                ));
            }
            selector.push('.');
            selector.push_str(class);
        }
        if let Some(id) = &self.id {
            if !valid_identifier(id) {
                return Err(UiError::InvalidStyle(
                    "style ID must be a valid CSS identifier".to_string(),
                ));
            }
            selector.push('#');
            selector.push_str(id);
        }
        if let Some(state) = self.state {
            selector.push_str(state.css());
        }
        Ok(selector)
    }
}

/// One selector and its declarations.
#[derive(Clone, Debug, PartialEq)]
pub struct UiStyleRule {
    selector: UiSelector,
    style: UiStyle,
    media: Option<UiMediaCondition>,
}

impl UiStyleRule {
    pub fn new(selector: UiSelector, style: UiStyle) -> Self {
        Self {
            selector,
            style,
            media: None,
        }
    }

    pub fn when(mut self, condition: UiMediaCondition) -> Self {
        self.media = Some(condition);
        self
    }

    pub fn media(&self) -> Option<UiMediaCondition> {
        self.media
    }

    pub fn selector(&self) -> &UiSelector {
        &self.selector
    }

    pub fn style(&self) -> &UiStyle {
        &self.style
    }
}

/// Ordered stylesheet. Later matching rules are emitted later and therefore win the CSS cascade.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct UiStyleSheet {
    rules: Vec<UiStyleRule>,
    keyframes: Vec<UiKeyframes>,
}

impl UiStyleSheet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn rule(mut self, selector: UiSelector, style: UiStyle) -> Self {
        self.rules.push(UiStyleRule::new(selector, style));
        self
    }

    pub fn push(&mut self, selector: UiSelector, style: UiStyle) {
        self.rules.push(UiStyleRule::new(selector, style));
    }

    pub fn media_rule(
        mut self,
        selector: UiSelector,
        style: UiStyle,
        condition: UiMediaCondition,
    ) -> Self {
        self.rules
            .push(UiStyleRule::new(selector, style).when(condition));
        self
    }

    pub fn keyframes(mut self, keyframes: UiKeyframes) -> Self {
        self.keyframes.push(keyframes);
        self
    }

    pub fn rules(&self) -> &[UiStyleRule] {
        &self.rules
    }

    pub fn keyframes_value(&self) -> &[UiKeyframes] {
        &self.keyframes
    }

    pub fn validate(&self) -> Result<(), UiError> {
        if self.keyframes.len() > 64 {
            return Err(UiError::InvalidStyle("a stylesheet cannot define more than 64 keyframe sets".to_string()));
        }
        for (index, keyframes) in self.keyframes.iter().enumerate() {
            keyframes.validate()?;
            if self.keyframes[..index].iter().any(|previous| previous.name == keyframes.name) {
                return Err(UiError::InvalidStyle("keyframe names must be unique within a stylesheet".to_string()));
            }
        }
        if self.rules.len() > 4096 {
            return Err(UiError::InvalidStyle("a stylesheet cannot contain more than 4096 rules".to_string()));
        }
        for rule in &self.rules {
            rule.selector.css()?;
            rule.style.validate()?;
            for animation in &rule.style.animations {
                if !self.keyframes.iter().any(|keyframes| keyframes.name == animation.name) {
                    return Err(UiError::InvalidStyle(format!(
                        "animation '{}' has no matching keyframe definition",
                        animation.name
                    )));
                }
            }
            if let Some(media) = rule.media {
                media.validate()?;
            }
        }
        Ok(())
    }

    /// Converts the shared rule model to CSS. Native backends may map the same tokens
    /// to native controls instead of loading CSS.
    pub fn to_css(&self) -> Result<String, UiError> {
        self.validate()?;
        let mut css = String::new();
        for keyframes in &self.keyframes {
            css.push_str("@keyframes ");
            css.push_str(&keyframes.name);
            css.push('{');
            for frame in &keyframes.frames {
                css.push_str(&format!("{}%{{", number(frame.offset_percent)));
                css.push_str(&frame.style.css_declarations()?);
                css.push('}');
            }
            css.push('}');
        }
        if !self.keyframes.is_empty() {
            css.push_str("@media(prefers-reduced-motion:reduce){[data-wl-component]{animation-duration:0.01ms!important;animation-delay:0ms!important;animation-iteration-count:1!important;}}");
        }
        for rule in &self.rules {
            let declarations = rule.style.css_declarations()?;
            if !declarations.is_empty() {
                let mut rule_css = String::new();
                rule_css.push_str(&rule.selector.css()?);
                rule_css.push('{');
                rule_css.push_str(&declarations);
                rule_css.push('}');
                if let Some(media) = rule.media {
                    css.push_str(&format!("@media {}{{{rule_css}}}", media.css()));
                } else {
                    css.push_str(&rule_css);
                }
            }
        }
        Ok(css)
    }
}

fn append_palette_variables(css: &mut String, palette: &UiPalette) {
    css.push_str(&format!(
        "--wl-color-window:{};--wl-color-surface:{};--wl-color-surface-alt:{};--wl-color-surface-elevated:{};",
        palette.window.to_css(),
        palette.surface.to_css(),
        palette.surface_alt.to_css(),
        palette.surface_elevated.to_css()
    ));
    css.push_str(&format!(
        "--wl-color-text-primary:{};--wl-color-text-secondary:{};--wl-color-text-disabled:{};--wl-color-border:{};--wl-color-border-strong:{};",
        palette.text_primary.to_css(),
        palette.text_secondary.to_css(),
        palette.text_disabled.to_css(),
        palette.border.to_css(),
        palette.border_strong.to_css()
    ));
    css.push_str(&format!(
        "--wl-color-accent:{};--wl-color-accent-hover:{};--wl-color-accent-pressed:{};--wl-color-accent-text:{};--wl-color-selection:{};--wl-color-selection-text:{};--wl-color-focus:{};",
        palette.accent.to_css(),
        palette.accent_hover.to_css(),
        palette.accent_pressed.to_css(),
        palette.accent_text.to_css(),
        palette.selection.to_css(),
        palette.selection_text.to_css(),
        palette.focus.to_css()
    ));
    css.push_str(&format!(
        "--wl-color-success:{};--wl-color-success-surface:{};--wl-color-warning:{};--wl-color-warning-surface:{};--wl-color-danger:{};--wl-color-danger-surface:{};--wl-color-info:{};--wl-color-info-surface:{};--wl-color-shadow:{};",
        palette.success.to_css(),
        palette.success_surface.to_css(),
        palette.warning.to_css(),
        palette.warning_surface.to_css(),
        palette.danger.to_css(),
        palette.danger_surface.to_css(),
        palette.info.to_css(),
        palette.info_surface.to_css(),
        palette.shadow.to_css()
    ));
}

fn valid_shadow(shadow: UiShadow) -> bool {
    // CSS permits signed offsets and spread (inset/contracting shadows); only
    // the blur radius is constrained to be non-negative.
    shadow.offset_x.is_finite()
        && shadow.offset_y.is_finite()
        && finite_non_negative(shadow.blur)
        && shadow.spread.is_finite()
}

fn valid_text_shadow(shadow: UiShadow) -> bool {
    shadow.offset_x.is_finite()
        && shadow.offset_y.is_finite()
        && finite_non_negative(shadow.blur)
        && shadow.spread == 0.0
}

fn append_css_value(css: &mut String, value: impl AsRef<str>) {
    css.push_str(value.as_ref());
}

fn shadow_css(shadow: UiShadow) -> String {
    format!(
        "{}px {}px {}px {}px {}",
        number(shadow.offset_x),
        number(shadow.offset_y),
        number(shadow.blur),
        number(shadow.spread),
        shadow.color.to_css()
    )
}

fn valid_font_family(value: &str) -> bool {
    !value.trim().is_empty()
        && value.chars().all(|character| {
            character.is_alphanumeric() || matches!(character, ' ' | '\t' | '-' | '_' | ',')
        })
}

fn css_font_family(value: &str) -> String {
    let families = value
        .split(',')
        .map(str::trim)
        .filter(|family| !family.is_empty())
        .map(|family| format!("\"{family}\""))
        .collect::<Vec<_>>()
        .join(",");
    format!("{families},sans-serif")
}

fn valid_identifier(value: &str) -> bool {
    let mut characters = value.chars();
    match characters.next() {
        Some(first) if first.is_ascii_alphabetic() || first == '_' => {}
        _ => return false,
    }
    characters
        .all(|character| character.is_ascii_alphanumeric() || character == '_' || character == '-')
}

fn finite_positive(value: f32) -> bool {
    value.is_finite() && value > 0.0
}

fn finite_non_negative(value: f32) -> bool {
    value.is_finite() && value >= 0.0
}

fn number(value: f32) -> String {
    let rendered = format!("{value:.3}");
    rendered
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_rgb_and_rgba_hex_colors() {
        assert_eq!(
            UiColor::from_hex("#123456"),
            Some(UiColor::rgb(0x12, 0x34, 0x56))
        );
        assert_eq!(
            UiColor::from_hex("#12345680"),
            Some(UiColor::rgba(0x12, 0x34, 0x56, 0x80))
        );
        assert_eq!(UiColor::from_hex("123456"), None);
        assert_eq!(UiColor::from_hex("#12345"), None);
        assert_eq!(UiColor::from_hex("#é23456"), None);
    }

    #[test]
    fn system_theme_emits_light_dark_and_forced_color_tokens() {
        let css = UiTheme::system().css_variables().unwrap();
        assert!(css.contains("@media(prefers-color-scheme:dark)"));
        assert!(css.contains("@media(forced-colors:active)"));
        assert!(css.contains("--wl-color-accent"));
        assert!(css.contains("--wl-font-family"));
        assert!(css.contains("@media(forced-colors:active)"));
        let reduced = UiTheme::system()
            .effects(UiEffects {
                reduced_motion: true,
                ..UiEffects::default()
            })
            .css_variables()
            .unwrap();
        assert!(reduced.contains("--wl-motion-fast:0ms"));
        assert!(!reduced.contains("@media(prefers-reduced-motion:no-preference)"));
    }

    #[test]
    fn stylesheet_emits_stable_semantic_selectors_and_typed_values() {
        let styles = UiStyleSheet::new().rule(
            UiSelector::new(UiComponent::Button)
                .class("primary")
                .state(UiVisualState::Hover),
            UiStyle::new()
                .background(UiColor::rgb(1, 2, 3))
                .padding(UiEdges::xy(12.0, 8.0))
                .border_radius(6.0)
                .font_weight(600),
        );
        let css = styles.to_css().unwrap();
        assert!(css.contains("[data-wl-component=\"button\"].primary:hover"));
        assert!(css.contains("background-color:rgba(1, 2, 3, 1.000)"));
        assert!(css.contains("padding:8px 12px 8px 12px"));
        assert!(css.contains("font-weight:600"));
        assert!(css.contains("border-radius:6px"));
    }

    #[test]
    fn typed_style_values_can_be_read_without_css_parsing() {
        let style = UiStyle::new()
            .display(UiDisplay::Grid)
            .font_size(16.0)
            .font_style(UiFontStyle::Italic)
            .grid_columns([UiGridTrack::Fraction(1.0), UiGridTrack::Px(240.0)])
            .corner_radii(UiCornerRadii::uniform(8.0))
            .opacity(0.9)
            .transition(UiTransition::new(
                UiTransitionProperty::Opacity,
                120,
                UiEasing::EaseOut,
            ));
        assert_eq!(style.display_value(), Some(UiDisplay::Grid));
        assert_eq!(style.font_size_value(), Some(16.0));
        assert_eq!(style.font_style_value(), Some(UiFontStyle::Italic));
        assert_eq!(
            style.grid_columns_value(),
            Some(&[UiGridTrack::Fraction(1.0), UiGridTrack::Px(240.0)][..])
        );
        assert_eq!(
            style.corner_radii_value(),
            Some(UiCornerRadii::uniform(8.0))
        );
        assert_eq!(style.opacity_value(), Some(0.9));
        assert_eq!(style.transitions_value().len(), 1);

        let selector = UiSelector::new(UiComponent::Button)
            .class("primary")
            .id("save")
            .state(UiVisualState::Focused);
        assert_eq!(selector.class_name(), Some("primary"));
        assert_eq!(selector.id_name(), Some("save"));
        assert_eq!(selector.visual_state(), Some(UiVisualState::Focused));
    }

    #[test]
    fn grid_and_text_layout_values_emit_validated_css() {
        let style = UiStyle::new()
            .display(UiDisplay::Grid)
            .grid_template(
                [UiGridTrack::Fraction(1.0), UiGridTrack::Px(280.0)],
                [UiGridTrack::Auto, UiGridTrack::Fraction(1.0)],
            )
            .grid_gap(12.0, 16.0)
            .align_content(UiAlignContent::Start)
            .justify_items(UiJustifyItems::Stretch)
            .font_style(UiFontStyle::Italic)
            .text_overflow(UiTextOverflow::Ellipsis)
            .aspect_ratio(1.5)
            .corner_radii(UiCornerRadii::new(8.0, 8.0, 12.0, 12.0))
            .text_shadow(UiShadow::new(UiColor::rgb(0, 0, 0), 0.0, 1.0, 2.0, 0.0));
        let css = UiStyleSheet::new()
            .rule(UiSelector::new(UiComponent::Card), style)
            .to_css()
            .unwrap();

        assert!(css.contains("grid-template-columns:1fr 280px"));
        assert!(css.contains("grid-template-rows:auto 1fr"));
        assert!(css.contains("row-gap:12px"));
        assert!(css.contains("column-gap:16px"));
        assert!(css.contains("border-radius:8px 8px 12px 12px"));
        assert!(css.contains("font-style:italic"));
        assert!(css.contains("text-overflow:ellipsis"));

        let invalid = UiStyleSheet::new().rule(
            UiSelector::new(UiComponent::Card),
            UiStyle::new().grid_columns([UiGridTrack::Fraction(0.0)]),
        );
        assert!(invalid.to_css().is_err());
    }

    #[test]
    fn rejects_invalid_selectors_and_css_injection_strings() {
        let invalid_selector = UiStyleSheet::new().rule(
            UiSelector::new(UiComponent::Button).class("ok}body"),
            UiStyle::new().opacity(0.5),
        );
        assert!(invalid_selector.to_css().is_err());

        let invalid_font = UiStyleSheet::new().rule(
            UiSelector::new(UiComponent::Text),
            UiStyle::new().font_family("Segoe UI; color:red"),
        );
        assert!(invalid_font.to_css().is_err());
    }

    #[test]
    fn gradient_layout_and_responsive_styles_emit_typed_css() {
        let gradient = UiLinearGradient::new(
            90.0,
            [
                UiGradientStop::new(UiColor::rgb(0, 0, 0), 0.0),
                UiGradientStop::new(UiColor::rgb(255, 255, 255), 100.0),
            ],
        );
        let sheet = UiStyleSheet::new().media_rule(
            UiSelector::new(UiComponent::Card).class("hero"),
            UiStyle::new()
                .linear_gradient(gradient)
                .display(UiDisplay::Flex)
                .flex_direction(UiFlexDirection::Column)
                .justify_content(UiJustifyContent::SpaceBetween)
                .overflow(UiOverflow::Hidden)
                .transform(UiTransform::identity().translate(2.0, 3.0))
                .transition(UiTransition::new(
                    UiTransitionProperty::Opacity,
                    120,
                    UiEasing::EaseOut,
                )),
            UiMediaCondition::MinWidth(720.0),
        );
        let css = sheet.to_css().unwrap();
        assert!(css.contains("@media (min-width:720px)"));
        assert!(css.contains("linear-gradient(90deg"));
        assert!(css.contains("display:flex"));
        assert!(css.contains("flex-direction:column"));
        assert!(css.contains("transition:opacity 120ms ease-out 0ms"));
    }

    #[test]
    fn rejects_invalid_gradients_but_supports_mirrored_transforms() {
        let invalid_gradient = UiLinearGradient::new(
            400.0,
            [
                UiGradientStop::new(UiColor::rgb(0, 0, 0), 0.0),
                UiGradientStop::new(UiColor::rgb(255, 255, 255), 100.0),
            ],
        );
        assert!(UiStyleSheet::new()
            .rule(
                UiSelector::new(UiComponent::Surface),
                UiStyle::new().linear_gradient(invalid_gradient),
            )
            .to_css()
            .is_err());

        let mirrored = UiStyleSheet::new().rule(
            UiSelector::new(UiComponent::Surface),
            UiStyle::new().transform(UiTransform::identity().scale(-1.0, 1.0)),
        );
        assert!(mirrored.to_css().unwrap().contains("scale(-1,1)"));
    }

    #[test]
    fn shadow_offsets_and_spread_may_be_negative_but_blur_may_not() {
        let shadow = UiShadow::new(UiColor::rgb(0, 0, 0), -2.0, 3.0, 4.0, -1.0);
        let text_shadow = UiShadow::new(UiColor::rgb(0, 0, 0), -2.0, 3.0, 4.0, 0.0);
        let css = UiStyleSheet::new()
            .rule(
                UiSelector::new(UiComponent::Card),
                UiStyle::new().shadow(shadow).text_shadow(text_shadow),
            )
            .to_css()
            .unwrap();
        assert!(css.contains("box-shadow:-2px 3px 4px -1px"));
        assert!(css.contains("text-shadow:-2px 3px 4px rgba("));

        let invalid_text_shadow = UiStyleSheet::new().rule(
            UiSelector::new(UiComponent::Text),
            UiStyle::new().text_shadow(UiShadow::new(
                UiColor::rgb(0, 0, 0),
                0.0,
                0.0,
                1.0,
                1.0,
            )),
        );
        assert!(invalid_text_shadow.validate().is_err());

        let invalid = UiStyleSheet::new().rule(
            UiSelector::new(UiComponent::Card),
            UiStyle::new().shadow(UiShadow::new(UiColor::rgb(0, 0, 0), 0.0, 0.0, -1.0, 0.0)),
        );
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn supports_negative_margins_and_percentages_over_one_hundred() {
        let style = UiStyleSheet::new().rule(
            UiSelector::new(UiComponent::Card),
            UiStyle::new()
                .margin(UiEdges::new(-8.0, 0.0, 4.0, -12.0))
                .width(UiLength::Percent(125.0)),
        );
        let css = style.to_css().unwrap();
        assert!(css.contains("margin:-8px 0px 4px -12px"));
        assert!(css.contains("width:125%"));

        let invalid_padding = UiStyleSheet::new().rule(
            UiSelector::new(UiComponent::Card),
            UiStyle::new().padding(UiEdges::all(-1.0)),
        );
        assert!(invalid_padding.validate().is_err());
    }

    #[test]
    fn supports_signed_position_insets_and_outline_offsets() {
        let css = UiStyleSheet::new()
            .rule(
                UiSelector::new(UiComponent::Card),
                UiStyle::new()
                    .position(UiPosition::Absolute)
                    .top(UiLength::Px(-12.0))
                    .left(UiLength::Percent(-5.0))
                    .outline(2.0, -3.0, UiColor::rgb(1, 2, 3)),
            )
            .to_css()
            .unwrap();
        assert!(css.contains("top:-12px"));
        assert!(css.contains("left:-5%"));
        assert!(css.contains("outline-offset:-3px"));

        let invalid = UiStyleSheet::new().rule(
            UiSelector::new(UiComponent::Card),
            UiStyle::new().top(UiLength::Px(f32::INFINITY)),
        );
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn rejects_invalid_numeric_styles() {
        let invalid = UiStyleSheet::new().rule(
            UiSelector::new(UiComponent::Card),
            UiStyle::new().opacity(1.5),
        );
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn emits_typed_keyframes_and_animation_declarations() {
        let keyframes = UiKeyframes::new(
            "fade_in",
            [
                UiKeyframe::new(0.0, UiStyle::new().opacity(0.0)),
                UiKeyframe::new(
                    100.0,
                    UiStyle::new().opacity(1.0).transform(UiTransform::identity().translate(0.0, 0.0)),
                ),
            ],
        );
        let stylesheet = UiStyleSheet::new()
            .keyframes(keyframes)
            .rule(
                UiSelector::new(UiComponent::Card).class("entering"),
                UiStyle::new().animation(
                    UiAnimation::new("fade_in", 240)
                        .delay(-20)
                        .iterations(UiAnimationIterations::Count(2))
                        .direction(UiAnimationDirection::Alternate)
                        .fill_mode(UiAnimationFillMode::Both)
                        .play_state(UiAnimationPlayState::Paused)
                        .easing(UiEasing::EaseOut),
                ),
            );
        let css = stylesheet.to_css().unwrap();
        assert!(css.contains("@keyframes fade_in{0%{opacity:0;}"));
        assert!(css.contains("100%{opacity:1;"));
        assert!(css.contains("animation:fade_in calc(240ms * var(--wl-motion-enabled,1)) ease-out calc(-20ms * var(--wl-motion-enabled,1)) 2 alternate both paused;"));
        assert!(css.contains("prefers-reduced-motion:reduce"));
    }

    #[test]
    fn rejects_untrusted_or_unbounded_animation_definitions() {
        let unsafe_name = UiStyleSheet::new().keyframes(UiKeyframes::new(
            "fade;body",
            [
                UiKeyframe::new(0.0, UiStyle::new().opacity(0.0)),
                UiKeyframe::new(100.0, UiStyle::new().opacity(1.0)),
            ],
        ));
        assert!(unsafe_name.validate().is_err());

        let missing_definition = UiStyleSheet::new().rule(
            UiSelector::new(UiComponent::Card),
            UiStyle::new().animation(UiAnimation::new("missing", 100)),
        );
        assert!(missing_definition.validate().is_err());

        let invalid_iterations = UiStyleSheet::new()
            .keyframes(UiKeyframes::new(
                "pulse",
                [
                    UiKeyframe::new(0.0, UiStyle::new().opacity(0.2)),
                    UiKeyframe::new(100.0, UiStyle::new().opacity(1.0)),
                ],
            ))
            .rule(
                UiSelector::new(UiComponent::Card),
                UiStyle::new().animation(
                    UiAnimation::new("pulse", 100).iterations(UiAnimationIterations::Count(1001)),
                ),
            );
        assert!(invalid_iterations.validate().is_err());
    }

    #[test]
    fn theme_presets_validate_and_resolve_expected_palettes() {
        assert!(UiTheme::light().validate().is_ok());
        assert!(UiTheme::dark().validate().is_ok());
        assert!(UiTheme::high_contrast().validate().is_ok());
        assert_eq!(UiTheme::dark().palette().window, UiPalette::dark().window);
    }
}
