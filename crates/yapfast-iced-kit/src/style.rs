//! The small style helpers every view uses. The colours are the theme's
//! ([`crate::theme::Theme`]); the few drawn the same in every theme are
//! here.

use iced::{
    Background, Border, Color, border,
    widget::{container, scrollable},
};

use crate::theme::Theme;

/// What is drawn in the same colour whatever the theme:
/// Discord's grey for a member list's headings, its wash for a member row
/// under the pointer (`#97979f1f`) or whose card is open (`#97979f33`),
/// the chips of server tags, and the crown and the APP tag.
pub const HEADING: u32 = 0x81828a;
pub const MEMBER_HOVER: Color = grey(0x1f);
pub const MEMBER_OPEN: Color = grey(0x33);
pub const CHIP: Color = grey(0x33);
pub const CROWN: u32 = 0xf0b232;
pub const BLURPLE: u32 = 0x5865f2;
/// Black at 70%: the ground of marks and notes over pictures.
pub const SMOKE: Color = Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 0xb3 as f32 / 255.0,
};

const fn grey(alpha: u8) -> Color {
    Color {
        r: 0x97 as f32 / 255.0,
        g: 0x97 as f32 / 255.0,
        b: 0x9f as f32 / 255.0,
        a: alpha as f32 / 255.0,
    }
}

/// A colour from 0xRRGGBB.
pub fn hex(rgb: u32) -> Color {
    Color::from_rgb8((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}

/// `color` with its alpha scaled by `alpha`.
pub fn faded(color: Color, alpha: f32) -> Color {
    Color {
        a: color.a * alpha,
        ..color
    }
}

/// `over` laid on the opaque colour `under`.
pub fn blend(under: Color, over: Color) -> Color {
    let mix = |a: f32, b: f32| a * (1.0 - over.a) + b * over.a;
    Color::from_rgb(
        mix(under.r, over.r),
        mix(under.g, over.g),
        mix(under.b, over.b),
    )
}

/// A container filled with one colour.
pub fn fill(color: Color) -> container::Style {
    container::Style {
        background: Some(Background::Color(color)),
        ..Default::default()
    }
}

/// A container filled with `color`, its corners rounded by `radius`.
pub fn rounded(color: Color, radius: impl Into<border::Radius>) -> container::Style {
    container::Style {
        background: Some(color.into()),
        border: Border::default().rounded(radius),
        ..Default::default()
    }
}

/// A 1 px outline in `color`, its corners rounded by `radius`.
pub fn edge(color: Color, radius: impl Into<border::Radius>) -> Border {
    Border {
        color,
        width: 1.0,
        radius: radius.into(),
    }
}

/// No outline, its corners rounded by `radius`: the border of a button's,
/// a field's or a quad's style.
pub fn round(radius: impl Into<border::Radius>) -> Border {
    Border::default().rounded(radius)
}

/// A container filled with `ground` in a 1 px `outline`, its corners
/// rounded by `radius`: a card, a bar, a field.
pub fn outlined(
    ground: Color,
    outline: Color,
    radius: impl Into<border::Radius>,
) -> container::Style {
    let radius = radius.into();
    rounded(ground, radius).border(edge(outline, radius))
}

/// Each corner its own radius: top left, top right, bottom right, bottom left.
pub fn corners([top_left, top_right, bottom_right, bottom_left]: [f32; 4]) -> border::Radius {
    border::Radius {
        top_left,
        top_right,
        bottom_right,
        bottom_left,
    }
}

/// A card's shadow, over the window or a dimmed one: the sign-in card, a
/// dialog.
pub fn card_shadow() -> iced::Shadow {
    iced::Shadow {
        color: Color {
            a: 0.4,
            ..Color::BLACK
        },
        offset: iced::Vector::new(0.0, 12.0),
        blur_radius: 32.0,
    }
}

/// The shadow of a window over a card: the Display Name Style's, over the
/// profile editor.
pub fn window_shadow() -> iced::Shadow {
    iced::Shadow {
        color: Color {
            a: 0.7,
            ..Color::BLACK
        },
        offset: iced::Vector::new(0.0, 16.0),
        blur_radius: 48.0,
    }
}

/// The raised panels' shadow: menus, the picker, suggestions; and the
/// settings' cards.
pub fn raised_shadow() -> iced::Shadow {
    iced::Shadow {
        color: Color {
            a: 0.5,
            ..Color::BLACK
        },
        offset: iced::Vector::new(0.0, 8.0),
        blur_radius: 24.0,
    }
}

/// A raised panel: `ground` in a 1 px `edge` with `radius` corners, over
/// the raised shadow, as menus, suggestions, the picker and cards are.
pub fn panel(ground: Color, edge: Color, radius: f32) -> container::Style {
    container::Style {
        background: Some(ground.into()),
        border: self::edge(edge, radius),
        shadow: raised_shadow(),
        ..Default::default()
    }
}

/// [`panel`] through an installed theme's named surface tokens.
pub fn surface_panel(
    theme: &Theme,
    surface: &str,
    ground: Color,
    edge: Color,
    radius: f32,
) -> container::Style {
    container::Style {
        background: Some(ground.into()),
        border: theme.surface_border(surface, radius, 1.0, edge),
        shadow: theme.surface_shadow(surface, raised_shadow()),
        ..Default::default()
    }
}

/// The shadow of a bar floating over the timeline: Jump to present, the
/// unread bar.
pub fn bar_shadow() -> iced::Shadow {
    iced::Shadow {
        color: Color::from_rgba8(0, 0, 0, 0x60 as f32 / 255.0),
        offset: iced::Vector::new(0.0, 4.0),
        blur_radius: 12.0,
    }
}

/// The shadow of a message's hover bar, over its row's top edge.
pub fn hover_bar_shadow() -> iced::Shadow {
    iced::Shadow {
        color: Color::from_rgba8(0, 0, 0, 0x59 as f32 / 255.0),
        offset: iced::Vector::new(0.0, 2.0),
        blur_radius: 10.0,
    }
}

/// A thin scroller on no rail: faint on a dark theme, darker on a light
/// one, and plainer under the pointer.
pub fn scrollbar(
    theme: &Theme,
) -> impl Fn(&iced::Theme, scrollable::Status) -> scrollable::Style + use<> {
    // iced fixes the default vertical scroller's layout width at 10 px. A
    // theme's narrower owner width is therefore drawn as a transparent inset
    // while the stock hit area stays intact. The package format caps both
    // widths at 10 px, so this never needs to grow outside iced's laid-out
    // scroller. Wired's few horizontal scrollers explicitly use a 4 px layout;
    // leave their geometry alone because a style callback cannot inspect it.
    const SCROLLER_WIDTH: f32 = 10.0;
    let dark = theme.dark;
    let radius = theme.surface_radius("scrollbar", 4.0);
    let (width, hover_width) = theme.surface_widths("scrollbar");
    let package_scroller = theme.package_paint("scrollbar");
    let package_track = theme.package_paint("scrollbar-track");
    move |iced_theme, status| {
        let mut style = scrollable::default(iced_theme, status);
        let pointed = matches!(
            status,
            scrollable::Status::Hovered {
                is_vertical_scrollbar_hovered: true,
                ..
            } | scrollable::Status::Dragged { .. }
        );
        let scroller = match (dark, pointed) {
            (true, false) => Color::from_rgba8(0xff, 0xff, 0xff, 0x14 as f32 / 255.0),
            (true, true) => Color::from_rgba8(0xff, 0xff, 0xff, 0xc9 as f32 / 255.0),
            (false, false) => Color::from_rgba8(0, 0, 0, 0x73 as f32 / 255.0),
            (false, true) => Color::from_rgba8(0, 0, 0, 0x99 as f32 / 255.0),
        };
        let inset = (SCROLLER_WIDTH
            - if pointed { hover_width } else { width }
                .unwrap_or(SCROLLER_WIDTH)
                .clamp(0.0, SCROLLER_WIDTH))
            / 2.0;
        for (rail, inset) in [
            (&mut style.vertical_rail, inset),
            (&mut style.horizontal_rail, 0.0),
        ] {
            rail.background = pointed.then_some(package_track).flatten();
            rail.border = Border::default();
            rail.scroller.background = package_scroller.unwrap_or_else(|| scroller.into());
            rail.scroller.border = Border {
                color: Color::TRANSPARENT,
                width: inset,
                // The transparent border consumes `inset` from each edge;
                // adding it here leaves the visible thumb at `radius`.
                radius: (radius + inset).into(),
            };
        }
        style
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use iced::{Background, Color};

    use super::*;
    use crate::theme::{Metrics, Shape};

    #[test]
    fn an_owner_scrollbar_uses_its_paints_and_rest_and_hover_widths() {
        let theme = Theme::new("kit-scrollbar-owner", true).worked_out();
        let scroller = Background::Color(Color::from_rgb8(0x12, 0x34, 0x56));
        let track = Background::Color(Color::from_rgb8(0x65, 0x43, 0x21));
        theme.register_metrics(Metrics {
            paints: HashMap::from([
                ("scrollbar".to_owned(), scroller),
                ("scrollbar-track".to_owned(), track),
            ]),
            shapes: HashMap::from([(
                "scrollbar".to_owned(),
                Shape {
                    radius: Some(2.0),
                    width: Some(4.0),
                    hover_width: Some(8.0),
                    ..Shape::default()
                },
            )]),
            ..Metrics::default()
        });

        let iced = iced::Theme::Dark;
        let style = scrollbar(&theme);
        let rest = style(
            &iced,
            scrollable::Status::Active {
                is_horizontal_scrollbar_disabled: false,
                is_vertical_scrollbar_disabled: false,
            },
        );
        assert_eq!(rest.vertical_rail.background, None);
        assert_eq!(rest.vertical_rail.scroller.background, scroller);
        assert_eq!(rest.vertical_rail.scroller.border.width, 3.0);

        let hover = style(
            &iced,
            scrollable::Status::Hovered {
                is_horizontal_scrollbar_hovered: false,
                is_vertical_scrollbar_hovered: true,
                is_horizontal_scrollbar_disabled: false,
                is_vertical_scrollbar_disabled: false,
            },
        );
        assert_eq!(hover.vertical_rail.background, Some(track));
        assert_eq!(hover.vertical_rail.scroller.background, scroller);
        assert_eq!(hover.vertical_rail.scroller.border.width, 1.0);
    }
}
