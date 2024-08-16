use cosmic::cosmic_theme::Component;
use cosmic::iced_core;
use cosmic::{theme::TRANSPARENT_COMPONENT, widget::button::Appearance, Theme};
use iced_core::{Background, Color};

#[derive(Default)]
#[allow(dead_code)]
pub enum Button {
    AppletIcon,
    Custom {
        active: Box<dyn Fn(bool, &cosmic::Theme) -> Appearance>,
        disabled: Box<dyn Fn(&cosmic::Theme) -> Appearance>,
        hovered: Box<dyn Fn(bool, &cosmic::Theme) -> Appearance>,
        pressed: Box<dyn Fn(bool, &cosmic::Theme) -> Appearance>,
    },
    AppletMenu,
    Destructive,
    HeaderBar,
    Icon,
    IconVertical,
    Image,
    Link,
    MenuItem,
    MenuRoot,
    #[default]
    Standard,
    Suggested,
    Text,
    Transparent,
}

pub fn appearance(
    theme: &cosmic::Theme,
    focused: bool,
    selected: bool,
    style: &Button,
    color: impl Fn(&Component) -> (Color, Option<Color>, Option<Color>),
) -> Appearance {
    let cosmic = theme.cosmic();
    let mut corner_radii = &cosmic.corner_radii.radius_xl;
    let mut appearance = Appearance::new();

    match style {
        Button::Standard
        | Button::Text
        | Button::Suggested
        | Button::Destructive
        | Button::Transparent => {
            let style_component = match style {
                Button::Standard => &cosmic.button,
                Button::Text => &cosmic.text_button,
                Button::Suggested => &cosmic.accent_button,
                Button::Destructive => &cosmic.destructive_button,
                Button::Transparent => &TRANSPARENT_COMPONENT,
                _ => return appearance,
            };

            let (background, text, icon) = color(style_component);
            appearance.background = Some(Background::Color(background));
            if !matches!(style, Button::Standard) {
                appearance.text_color = text;
                appearance.icon_color = icon;
            }
        }

        Button::Icon | Button::IconVertical | Button::HeaderBar => {
            if matches!(style, Button::IconVertical) {
                corner_radii = &cosmic.corner_radii.radius_m;
                if selected {
                    appearance.overlay = Some(Background::Color(Color::from(
                        cosmic.icon_button.selected_state_color(),
                    )));
                }
            }
            if focused || selected {
                appearance.icon_color = Some(cosmic.accent.base.into());
                appearance.text_color = Some(cosmic.accent.base.into());
            }
            let (background, _text, _icon) = color(&cosmic.icon_button);
            appearance.background = Some(Background::Color(background));
        }

        Button::Image => {
            appearance.background = None;
            appearance.text_color = Some(cosmic.accent.base.into());
            appearance.icon_color = Some(cosmic.accent.base.into());

            corner_radii = &cosmic.corner_radii.radius_s;
            appearance.border_radius = (*corner_radii).into();

            if focused || selected {
                appearance.border_width = 2.0;
                appearance.border_color = cosmic.accent.base.into();
            }

            return appearance;
        }

        Button::Link => {
            appearance.background = None;
            appearance.icon_color = Some(cosmic.accent.base.into());
            appearance.text_color = Some(cosmic.accent.base.into());
            corner_radii = &cosmic.corner_radii.radius_0;
        }

        Button::Custom { .. } => (),
        Button::AppletMenu => {
            let (background, _, _) = color(&cosmic.text_button);
            appearance.background = Some(Background::Color(background));

            appearance.icon_color = Some(cosmic.background.on.into());
            appearance.text_color = Some(cosmic.background.on.into());
            corner_radii = &cosmic.corner_radii.radius_0;
        }
        Button::AppletIcon => {
            let (background, _, _) = color(&cosmic.text_button);
            appearance.background = Some(Background::Color(background));

            appearance.icon_color = Some(cosmic.background.on.into());
            appearance.text_color = Some(cosmic.background.on.into());
        }
        Button::MenuRoot => {
            appearance.background = None;
            appearance.icon_color = None;
            appearance.text_color = None;
        }
        Button::MenuItem => {
            let (background, _, _) = color(&cosmic.background.component);
            appearance.background = Some(Background::Color(background));

            appearance.icon_color = Some(cosmic.background.on.into());
            appearance.text_color = Some(cosmic.background.on.into());
            corner_radii = &cosmic.corner_radii.radius_s;
        }
    }

    appearance.border_radius = (*corner_radii).into();

    if focused {
        appearance.outline_width = 1.0;
        appearance.outline_color = cosmic.accent.base.into();
        appearance.border_width = 2.0;
        appearance.border_color = Color::TRANSPARENT;
    }

    appearance
}
// not my code

pub fn active(theme: &Theme, focused: bool, selected: bool, style: Button) -> Appearance {
    return appearance(theme, focused, selected, &style, |component| {
        let text_color = if matches!(
            style,
            Button::Icon | Button::IconVertical | Button::HeaderBar
        ) && selected
        {
            Some(theme.cosmic().accent_color().into())
        } else {
            Some(component.on.into())
        };

        (component.base.into(), text_color, text_color)
    });
}

pub fn hovered(theme: &Theme, focused: bool, selected: bool, style: Button) -> Appearance {
    return appearance(theme, focused, selected, &style, |component| {
        let text_color = if matches!(
            style,
            Button::Icon | Button::IconVertical | Button::HeaderBar
        ) && selected
        {
            Some(theme.cosmic().accent_color().into())
        } else {
            Some(component.on.into())
        };

        (component.hover.into(), text_color, text_color)
    });
}
pub fn pressed(theme: &Theme, focused: bool, selected: bool, style: Button) -> Appearance {
    return appearance(theme, focused, selected, &style, |component| {
        let text_color = if matches!(
            style,
            Button::Icon | Button::IconVertical | Button::HeaderBar
        ) && selected
        {
            Some(theme.cosmic().accent_color().into())
        } else {
            Some(component.on.into())
        };

        (component.pressed.into(), text_color, text_color)
    });
}

pub fn selection_background(theme: &Theme) -> Background {
    Background::Color(theme.cosmic().primary.base.into())
}
