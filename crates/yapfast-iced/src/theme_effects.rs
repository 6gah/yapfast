//! Owner-theme text light and pointer effects on ordinary iced content.
//!
//! These stay beside the app's resolved package registry: Wired's kit
//! [`Theme`](crate::theme::Theme) remains the one colour/style boundary and
//! keeps its original shape.

use std::{
    hash::{DefaultHasher, Hash, Hasher},
    time::{Duration, Instant},
};

use iced::{Color, Element, Font, Point, Rectangle, Vector, advanced::Shell};
use yapfast_core::themes::{
    Performance,
    package::art::{Event, EventSlot, Foil, FoilSurface, TextRole},
};

use crate::{
    app::Message,
    piece::TextLight,
    widgets::{At, Hooks, Inner, Wrap},
};

/// A resolved role's shadow/glow/outline/emboss in the renderer-neutral kit
/// form used by selectable rich text.
pub fn text_light(ctx: crate::ctx::Ctx<'_>, role: TextRole) -> TextLight {
    let Some(look) = ctx.themes.text_look(&ctx.look.theme.id, role) else {
        return TextLight::default();
    };
    let shadow = look.shadow.map(|shadow| {
        (
            crate::themes::argb(shadow.colour),
            Vector::new(shadow.x, shadow.y),
        )
    });
    let glow = look.glow.map(|(colour, reach)| {
        let mut colour = crate::themes::argb(colour);
        // The owner renderer draws two glow copies; one third alpha makes
        // their overlap build near the glyphs without flattening the face.
        colour.a /= 3.0;
        (colour, reach)
    });
    TextLight {
        shadow,
        glow,
        outline: look
            .outline
            .map(|(width, colour)| (crate::themes::argb(colour), width)),
        emboss: look.emboss,
    }
}

/// The owner's decrypt-on-hover event for a person name. The callback form
/// lets the generic kit widget animate core's canonical text algorithm
/// without knowing about package types.
pub fn name_hover_effect(
    ctx: crate::ctx::Ctx<'_>,
    words: &str,
) -> Option<crate::elide::HoverTextEffect> {
    let id = &ctx.look.theme.id;
    let event = ctx.themes.event_effect(id, EventSlot::NameHover)?;
    let (performance, reduced) = ctx.themes.art_motion(id);
    if !motion_runs(performance, reduced, ctx.plays) {
        return None;
    }
    let mut hash = DefaultHasher::new();
    words.hash(&mut hash);
    Some(crate::elide::HoverTextEffect {
        duration_ms: event.duration,
        seed: hash.finish(),
        every: Duration::from_millis(if performance == Performance::Full {
            1000 / 60
        } else {
            1000 / 30
        }),
        render: decrypt,
    })
}

fn decrypt(words: &str, elapsed: f32, duration: f32, seed: u64) -> (String, bool) {
    yapfast_core::themes::motion::effect_text(
        yapfast_core::themes::package::Effect::Decrypt,
        words,
        elapsed,
        duration,
        seed,
    )
}

/// A simple one-line label with the same owner text-light treatment as a
/// selectable [`crate::piece::Piece`]. Complex message text uses
/// [`text_light`] directly so its links and selection remain one paragraph.
pub fn label<'a>(
    ctx: crate::ctx::Ctx<'a>,
    role: TextRole,
    words: impl Into<String>,
    size: f32,
    font: Font,
    ink: Color,
) -> Element<'a, Message> {
    let light = text_light(ctx, role);
    crate::elide::elide(words.into())
        .size(size)
        .font(font)
        .color(ink)
        .light(light)
        .shrink()
        .into()
}

#[derive(Clone)]
struct Effects {
    click: Option<Event>,
    hover: Option<Event>,
    select: Option<(Event, bool)>,
    foil: Option<Foil>,
    running: bool,
    fps: u64,
}

#[derive(Default)]
struct State {
    hovered: bool,
    pointer: Point,
    hover_started: Option<Instant>,
    press_started: Option<Instant>,
    released: Option<Instant>,
    selected: bool,
    selected_started: Option<Duration>,
    elapsed: Duration,
    last: Option<Instant>,
    running: bool,
}

impl State {
    fn tick(&mut self, now: Instant, running: bool) {
        if running
            && self.running
            && let Some(last) = self.last
        {
            self.elapsed += now.saturating_duration_since(last);
        }
        self.last = Some(now);
        self.running = running;
    }

    fn animating(&self, effects: &Effects, now: Instant) -> bool {
        effects.foil.is_some()
            || effects.hover.is_some_and(|event| {
                self.hover_started
                    .is_some_and(|at| age(now, at) < event.duration / 1000.0)
            })
            || effects.click.is_some_and(|event| {
                self.press_started.is_some()
                    && (self.released.is_none()
                        || self
                            .released
                            .is_some_and(|at| age(now, at) < event.duration / 1000.0))
            })
    }
}

impl Hooks<Message> for Effects {
    type State = State;

    fn diff(&self, state: &mut State) {
        let selected = self.select.is_some_and(|(_, selected)| selected);
        if selected != state.selected {
            state.selected = selected;
            state.selected_started = selected.then_some(state.elapsed);
        }
    }

    fn update(
        &mut self,
        state: &mut State,
        event: &iced::Event,
        at: At,
        shell: &mut Shell<'_, Message>,
        child: &mut Inner<'_, '_, Message>,
    ) {
        let now = Instant::now();
        match event {
            iced::Event::Mouse(iced::mouse::Event::CursorMoved { .. }) => {
                let position = at.cursor.position();
                let hovered = position.is_some_and(|position| at.bounds.contains(position));
                if hovered && !state.hovered {
                    state.hover_started = Some(now);
                }
                state.hovered = hovered;
                if let Some(position) = position {
                    state.pointer = position;
                }
            }
            iced::Event::Mouse(iced::mouse::Event::CursorLeft) => state.hovered = false,
            iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left))
                if state.hovered && self.click.is_some() =>
            {
                state.press_started = Some(now);
                state.released = None;
            }
            iced::Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left))
                if state.press_started.is_some() =>
            {
                state.released = Some(now);
            }
            iced::Event::Window(iced::window::Event::Unfocused) => {
                state.hovered = false;
                state.last = None;
                state.running = false;
            }
            iced::Event::Window(iced::window::Event::RedrawRequested(at)) => {
                state.tick(*at, self.running);
                if self.running && state.animating(self, *at) {
                    shell.request_redraw_at(*at + Duration::from_millis(1000 / self.fps));
                }
            }
            _ => {}
        }
        child.update(shell);
    }

    fn draw(
        &self,
        state: &State,
        renderer: &mut crate::widgets::Renderer,
        at: At,
        child: &mut dyn FnMut(&mut crate::widgets::Renderer),
    ) {
        use iced::advanced::Renderer as _;

        let bounds = at.bounds;
        renderer.with_layer(bounds, |renderer| {
            if let Some((event, selected)) = self.select
                && selected
            {
                let amount = if self.running {
                    state.last.map_or(1.0, |_now| {
                        state.selected_started.map_or(1.0, |started| {
                            (state.elapsed.saturating_sub(started).as_secs_f32() * 1000.0
                                / event.duration.max(1.0))
                            .clamp(0.0, 1.0)
                        })
                    })
                } else {
                    1.0
                };
                let mark = Rectangle::new(
                    Point::new(bounds.x + 2.0, bounds.y + bounds.height * 0.18),
                    iced::Size::new((bounds.width - 4.0).max(0.0) * amount, bounds.height * 0.64),
                );
                quad(
                    renderer,
                    mark,
                    crate::themes::argb(event.colour).scale_alpha(0.55),
                    2.0,
                );
            }

            child(renderer);

            let now = Instant::now();
            if self.running
                && let Some(event) = self.hover
                && state.hovered
                && let Some(started) = state.hover_started
            {
                let pass = (age(now, started) * 1000.0 / event.duration.max(1.0)).clamp(0.0, 1.0);
                if pass < 1.0 {
                    let width = bounds.width * 0.6;
                    let band = Rectangle::new(
                        Point::new(bounds.x - width + bounds.width * 1.6 * pass, bounds.y),
                        iced::Size::new(width, bounds.height),
                    );
                    band_gradient(renderer, band, crate::themes::argb(event.colour), 110.0);
                }
            }

            if self.running
                && let Some(event) = self.click
                && let Some(started) = state.press_started
            {
                let duration = (event.duration / 1000.0).max(0.001);
                let grow = (age(now, started) / duration).clamp(0.0, 1.0);
                let fade = state.released.map_or(1.0, |released| {
                    (1.0 - age(now, released) / duration).clamp(0.0, 1.0)
                });
                if fade > 0.0 {
                    let reach = bounds.width.max(bounds.height) * 2.2 * grow;
                    let center = if bounds.contains(state.pointer) {
                        state.pointer
                    } else {
                        bounds.center()
                    };
                    quad(
                        renderer,
                        Rectangle::new(
                            Point::new(center.x - reach / 2.0, center.y - reach / 2.0),
                            iced::Size::new(reach, reach),
                        ),
                        crate::themes::argb(event.colour).scale_alpha(fade),
                        reach / 2.0,
                    );
                }
            }

            if let Some(foil) = &self.foil {
                let position = if self.running {
                    if state.hovered && bounds.width > 0.0 {
                        ((state.pointer.x - bounds.x) / bounds.width).clamp(0.0, 1.0)
                    } else {
                        (state.elapsed.as_secs_f32() * 1000.0 / foil.period.max(1.0)).fract()
                    }
                } else {
                    0.5
                };
                let width = bounds.width * 0.9;
                let band = Rectangle::new(
                    Point::new(
                        bounds.x - bounds.width * 0.75 + bounds.width * 1.5 * position,
                        bounds.y,
                    ),
                    iced::Size::new(width, bounds.height),
                );
                let colours = foil.colours.map(crate::themes::argb);
                foil_gradient(renderer, band, colours, foil.opacity);
            }
        });
    }
}

/// Adds the owner's click ripple and hover gloss to an ordinary interactive
/// surface. Missing declarations return Wired's element unchanged.
pub fn interactive<'a>(
    content: impl Into<Element<'a, Message>>,
    ctx: crate::ctx::Ctx<'a>,
) -> Element<'a, Message> {
    decorated(content, ctx, None, None)
}

/// Adds a selected-row highlighter as well as ordinary click/hover effects.
pub fn selected_row<'a>(
    content: impl Into<Element<'a, Message>>,
    ctx: crate::ctx::Ctx<'a>,
    selected: bool,
) -> Element<'a, Message> {
    decorated(
        content,
        ctx,
        Some((EventSlot::Select, selected, true)),
        None,
    )
}

/// Adds the owner's message-row highlighter while the pointer is on the row.
/// Unlike the selected-channel mark, this is deliberately absent in a still
/// theme, matching the owner renderer's reduced-motion behaviour.
pub fn message_row<'a>(
    content: impl Into<Element<'a, Message>>,
    ctx: crate::ctx::Ctx<'a>,
    hovered: bool,
) -> Element<'a, Message> {
    decorated(content, ctx, Some((EventSlot::Row, hovered, false)), None)
}

/// Whether the animated owner row mark replaces Wired's ordinary hover wash.
pub fn message_row_active(ctx: crate::ctx::Ctx<'_>) -> bool {
    let id = &ctx.look.theme.id;
    let (performance, reduced) = ctx.themes.art_motion(id);
    ctx.themes.event_effect(id, EventSlot::Row).is_some()
        && motion_runs(performance, reduced, ctx.plays)
}

/// Adds the owner's holographic band to a card or tile, together with the
/// ordinary pointer effects declared for that surface.
pub fn foil<'a>(
    content: impl Into<Element<'a, Message>>,
    ctx: crate::ctx::Ctx<'a>,
    surface: FoilSurface,
) -> Element<'a, Message> {
    decorated(content, ctx, None, Some(surface))
}

fn decorated<'a>(
    content: impl Into<Element<'a, Message>>,
    ctx: crate::ctx::Ctx<'a>,
    highlighter: Option<(EventSlot, bool, bool)>,
    foil_surface: Option<FoilSurface>,
) -> Element<'a, Message> {
    let id = &ctx.look.theme.id;
    let click = ctx.themes.event_effect(id, EventSlot::Click);
    let hover = ctx.themes.event_effect(id, EventSlot::Hover);
    let (performance, reduced) = ctx.themes.art_motion(id);
    let running = motion_runs(performance, reduced, ctx.plays);
    let select = highlighter.and_then(|(slot, selected, static_when_still)| {
        highlighter_effect(
            ctx.themes.event_effect(id, slot),
            selected,
            running,
            static_when_still,
        )
    });
    let foil = foil_surface.and_then(|surface| {
        ctx.themes
            .foil(id)
            .filter(|foil| foil.surfaces.contains(&surface))
            .cloned()
    });
    if click.is_none() && hover.is_none() && select.is_none() && foil.is_none() {
        return content.into();
    }
    Wrap::new(
        content,
        Effects {
            click,
            hover,
            select,
            foil,
            running,
            fps: if performance == Performance::Full {
                60
            } else {
                30
            },
        },
    )
    .into()
}

fn age(now: Instant, since: Instant) -> f32 {
    now.saturating_duration_since(since).as_secs_f32()
}

fn motion_runs(performance: Performance, reduced: bool, plays: bool) -> bool {
    !reduced && performance != Performance::Light && (performance == Performance::Full || plays)
}

fn highlighter_effect(
    event: Option<Event>,
    selected: bool,
    running: bool,
    static_when_still: bool,
) -> Option<(Event, bool)> {
    (running || static_when_still)
        .then_some(event)
        .flatten()
        .map(|event| (event, selected))
}

fn quad(renderer: &mut crate::widgets::Renderer, bounds: Rectangle, colour: Color, radius: f32) {
    use iced::advanced::Renderer as _;
    renderer.fill_quad(
        iced::advanced::renderer::Quad {
            bounds,
            border: iced::Border::default().rounded(radius),
            ..Default::default()
        },
        colour,
    );
}

fn band_gradient(
    renderer: &mut crate::widgets::Renderer,
    bounds: Rectangle,
    colour: Color,
    angle: f32,
) {
    let clear = Color { a: 0.0, ..colour };
    crate::widgets::gradient_quad(
        renderer,
        bounds,
        0.0,
        angle,
        &[(0.0, clear), (0.5, colour), (1.0, clear)],
    );
}

fn foil_gradient(
    renderer: &mut crate::widgets::Renderer,
    bounds: Rectangle,
    colours: [Color; 3],
    opacity: f32,
) {
    let [a, b, c] = colours.map(|colour| colour.scale_alpha(opacity));
    crate::widgets::gradient_quad(
        renderer,
        bounds,
        0.0,
        90.0,
        &[
            (0.0, Color { a: 0.0, ..a }),
            (0.25, a),
            (0.5, b),
            (0.75, c),
            (1.0, Color { a: 0.0, ..c }),
        ],
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn foil_surface_selection_and_event_age_are_bounded() {
        assert_eq!(age(Instant::now(), Instant::now()), 0.0);
        let foil = Foil {
            surfaces: vec![FoilSurface::Card],
            colours: [0xffff_0000, 0xff00_ff00, 0xff00_00ff],
            opacity: 0.5,
            period: 4000.0,
        };
        assert!(foil.surfaces.contains(&FoilSurface::Card));
        assert!(!foil.surfaces.contains(&FoilSurface::Tile));
    }

    #[test]
    fn performance_and_reduce_motion_gate_effect_animation() {
        assert!(motion_runs(Performance::Full, false, false));
        assert!(motion_runs(Performance::Balanced, false, true));
        assert!(!motion_runs(Performance::Balanced, false, false));
        assert!(!motion_runs(Performance::Light, false, true));
        assert!(!motion_runs(Performance::Full, true, true));
    }

    #[test]
    fn only_the_selection_highlighter_survives_a_still_theme() {
        let event = Event {
            colour: 0xffff_e45c,
            duration: 260.0,
        };
        assert_eq!(
            highlighter_effect(Some(event), true, false, true),
            Some((event, true))
        );
        assert_eq!(highlighter_effect(Some(event), true, false, false), None);
        assert_eq!(
            highlighter_effect(Some(event), true, true, false),
            Some((event, true))
        );
    }
}
