//! `dump`: what the window shows, as text an agent reads instead of a
//! screenshot. The control file's `dump [PATH]` writes it
//! (`control_file::control`), and `scripts/ui-session do 'dump'` prints it.
//!
//! A line is one element in view: the named part the theme names it by, or
//! else the widget's kind, then its text, where it is and the colours the
//! theme gives it. Lines are indented by what holds them, and a long list
//! shows its first few and then how many followed.
//!
//! The elements come from iced's headless simulator over the window's own
//! view, and the parts from the draw that simulator makes, which
//! [`inspect::collected`] records; `--theme-dev`'s inspector shares the
//! same tagged parts.
//!
//! It sits behind the `devtools` feature, which is what brings iced_test
//! into the window: a build without it answers that the build has no
//! `dump`, and carries neither. `scripts/ui-session` builds with it.

use std::path::PathBuf;

use crate::app::App;

#[cfg(feature = "devtools")]
use std::fmt::Write as _;

#[cfg(feature = "devtools")]
use iced::{Background, Color, Point, Rectangle, Settings};
#[cfg(feature = "devtools")]
use iced_test::{Simulator, selector::Candidate};
#[cfg(feature = "devtools")]
use yapfast_iced_kit::{
    inspect::{self, Drawn},
    parts::PartLook,
};

/// The longest text a line carries; longer texts are cut.
#[cfg(feature = "devtools")]
const TEXT: usize = 64;

/// How many lines alike in a row show, and how few in a row count as a
/// list to collapse.
#[cfg(feature = "devtools")]
const KEEP: usize = 4;
#[cfg(feature = "devtools")]
const RUN: usize = 8;

/// The deepest a line is indented, whatever holds it.
#[cfg(feature = "devtools")]
const DEEPEST: usize = 12;

/// Writes the window as text to `path`, or beside the control file when
/// none is given.
#[cfg(feature = "devtools")]
pub fn write(app: &App, path: Option<PathBuf>) -> Result<PathBuf, String> {
    let path = path.unwrap_or_else(crate::control_file::dump_path);
    std::fs::write(&path, text(app)).map_err(|error| format!("{}: {error}", path.display()))?;
    Ok(path)
}

/// This build has no simulator, so nothing to read the window with.
#[cfg(not(feature = "devtools"))]
pub fn write(_: &App, _: Option<PathBuf>) -> Result<PathBuf, String> {
    Err("this build has no `dump`: build it with --features devtools".to_owned())
}

/// The window as text.
#[cfg(feature = "devtools")]
pub fn text(app: &App) -> String {
    let window = Rectangle::with_size(app.shell.frame);
    let mut simulator = Simulator::with_size(Settings::default(), app.shell.frame, app.view());
    let mut rows: Vec<Row> = Vec::new();
    // What holds the element being walked, by the space it takes.
    let mut outer: Vec<Rectangle> = Vec::new();
    // This only walks the window's elements, so the find finds nothing.
    drop(simulator.find(|candidate: Candidate<'_>| {
        let Some(bounds) = candidate.visible_bounds() else {
            return None::<()>;
        };
        if !window.intersects(&bounds) {
            return None::<()>;
        }
        while outer
            .last()
            .is_some_and(|open| !open.contains(bounds.center()))
        {
            outer.pop();
        }
        let (kind, text) = describe(&candidate);
        if !text.is_empty() {
            rows.push(Row {
                depth: outer.len().min(DEEPEST),
                kind,
                text,
                bounds,
            });
        }
        outer.push(bounds);
        None::<()>
    }));
    // The parts this frame draws: their names, states and colours, and where
    // they are, which only the draw knows.
    let (_, parts) = inspect::collected(|| drop(simulator.snapshot(&iced::Theme::Dark)));
    let mut lines: Vec<Line> = Vec::new();
    for row in &rows {
        lines.push(line(row, innermost(&parts, row.bounds.center())));
    }
    for part in &parts {
        // A part with text in it says so on that text's line.
        if rows
            .iter()
            .any(|row| part.bounds.contains(row.bounds.center()))
        {
            continue;
        }
        let depth = parts
            .iter()
            .filter(|outer| {
                outer.bounds.contains(part.bounds.center())
                    && area(outer.bounds) > area(part.bounds)
            })
            .count();
        lines.push(part_line(depth, part));
    }
    // Top to bottom, then left to right, as the window reads.
    lines.sort_by(|a, b| a.at.partial_cmp(&b.at).unwrap_or(std::cmp::Ordering::Equal));
    render(&lines)
}

/// An element of the window that reads something.
#[cfg(feature = "devtools")]
struct Row {
    depth: usize,
    kind: &'static str,
    text: String,
    bounds: Rectangle,
}

/// A line of the dump: where it's indented from, what it is, and the rest.
#[cfg(feature = "devtools")]
struct Line {
    depth: usize,
    head: String,
    rest: String,
    /// Where the element is, for the dump's order.
    at: Point,
}

/// That element's line: its text, its place, and the part's states and
/// colours where a part holds it.
#[cfg(feature = "devtools")]
fn line(row: &Row, part: Option<&Drawn>) -> Line {
    let mut words: Vec<String> = Vec::new();
    if !row.text.is_empty() {
        words.push(format!("\"{}\"", elide(&one_line(&row.text), TEXT)));
    }
    words.push(place(row.bounds));
    let mut head = row.kind.to_owned();
    if let Some(part) = part {
        head = part.name.to_owned();
        words.extend(states(part.states));
        words.extend(colours(part.look.as_ref()));
    }
    Line {
        depth: row.depth,
        head,
        rest: words.join(" "),
        at: Point::new(row.bounds.x, row.bounds.y),
    }
}

/// A part with no text in it: a tile, an avatar, a divider.
#[cfg(feature = "devtools")]
fn part_line(depth: usize, part: &Drawn) -> Line {
    let mut words = vec![place(part.bounds)];
    words.extend(states(part.states));
    words.extend(colours(part.look.as_ref()));
    Line {
        depth: depth.min(DEEPEST),
        head: part.name.to_owned(),
        rest: words.join(" "),
        at: Point::new(part.bounds.x, part.bounds.y),
    }
}

/// The element's kind and what it reads.
#[cfg(feature = "devtools")]
fn describe(candidate: &Candidate<'_>) -> (&'static str, String) {
    match candidate {
        Candidate::Text { content, .. } => ("text", content.to_string()),
        Candidate::TextInput { state, .. } => ("field", state.text().to_owned()),
        Candidate::Container { .. } => ("box", String::new()),
        Candidate::Scrollable { .. } => ("list", String::new()),
        _ => ("widget", String::new()),
    }
}

/// The smallest part holding `at`.
#[cfg(feature = "devtools")]
fn innermost<'a>(parts: &'a [Drawn], at: Point) -> Option<&'a Drawn> {
    parts
        .iter()
        .filter(|part| part.bounds.contains(at))
        .min_by(|a, b| area(a.bounds).total_cmp(&area(b.bounds)))
}

#[cfg(feature = "devtools")]
fn area(bounds: Rectangle) -> f32 {
    bounds.width * bounds.height
}

/// Where the element is, rounded: x y width height.
#[cfg(feature = "devtools")]
fn place(bounds: Rectangle) -> String {
    format!(
        "{} {} {} {}",
        round(bounds.x),
        round(bounds.y),
        round(bounds.width),
        round(bounds.height)
    )
}

#[cfg(feature = "devtools")]
fn round(value: f32) -> i32 {
    value.round() as i32
}

/// The part's states, by name, as the inspector words them.
#[cfg(feature = "devtools")]
fn states(states: u8) -> Vec<String> {
    let words = inspect::states_words(states);
    if words == "rest" {
        Vec::new()
    } else {
        vec![words]
    }
}

/// The colours the theme gives the part: its ground and its ink.
#[cfg(feature = "devtools")]
fn colours(look: Option<&PartLook>) -> Vec<String> {
    let Some(look) = look else {
        return Vec::new();
    };
    let mut words = Vec::new();
    if let Some(Background::Color(colour)) = look.ground {
        words.push(format!("bg={}", hex(colour)));
    }
    if let Some(ink) = look.ink {
        words.push(format!("fg={}", hex(ink)));
    }
    words
}

#[cfg(feature = "devtools")]
fn hex(colour: Color) -> String {
    let byte = |value: f32| (value.clamp(0.0, 1.0) * 255.0).round() as u8;
    let mut text = format!(
        "#{:02x}{:02x}{:02x}",
        byte(colour.r),
        byte(colour.g),
        byte(colour.b)
    );
    if colour.a < 1.0 {
        let _ = write!(text, "/{:.2}", colour.a);
    }
    text
}

/// `text` with its runs of space, tabs and newlines made one space, so an
/// element is one line.
#[cfg(feature = "devtools")]
fn one_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// `text` cut to `limit` characters with a mark where it was cut.
#[cfg(feature = "devtools")]
fn elide(text: &str, limit: usize) -> String {
    if text.chars().count() <= limit {
        return text.to_owned();
    }
    let mut cut: String = text.chars().take(limit.saturating_sub(1)).collect();
    cut.push('…');
    cut
}

/// The lines, indented by what holds them, with a list's length counted
/// instead of shown.
#[cfg(feature = "devtools")]
fn render(lines: &[Line]) -> String {
    let mut out = String::new();
    let mut index = 0;
    while index < lines.len() {
        let mut end = index + 1;
        while end < lines.len()
            && lines[end].depth == lines[index].depth
            && lines[end].head == lines[index].head
        {
            end += 1;
        }
        if end - index > RUN {
            for line in &lines[index..index + KEEP] {
                line.render(&mut out);
            }
            let _ = writeln!(
                out,
                "{:indent$}… {} more {}",
                "",
                end - index - KEEP,
                lines[index].head,
                indent = lines[index].depth * 2
            );
        } else {
            for line in &lines[index..end] {
                line.render(&mut out);
            }
        }
        index = end;
    }
    out
}

#[cfg(feature = "devtools")]
impl Line {
    fn render(&self, out: &mut String) {
        let _ = writeln!(
            out,
            "{:indent$}{} {}",
            "",
            self.head,
            self.rest,
            indent = self.depth * 2
        );
    }
}

#[cfg(all(test, feature = "devtools"))]
mod tests {
    use super::*;

    #[test]
    fn a_long_text_is_cut_and_newlines_made_one_space() {
        assert_eq!(elide("short", 64), "short");
        assert_eq!(elide("abcdef", 4), "abc…");
        assert_eq!(one_line("two\nlines\there"), "two lines here");
    }

    #[test]
    fn a_list_counts_what_it_does_not_show() {
        let lines: Vec<Line> = (0..12)
            .map(|index| Line {
                depth: 1,
                head: "sidebar.channel-row".to_owned(),
                rest: format!("#general {index}"),
                at: Point::new(0.0, index as f32),
            })
            .collect();
        let text = render(&lines);
        assert_eq!(text.lines().count(), KEEP + 1);
        assert!(text.contains("… 8 more sidebar.channel-row"));
        assert!(
            text.lines()
                .next()
                .is_some_and(|line| line.starts_with("  "))
        );
    }

    #[test]
    fn the_colours_are_the_themes() {
        let look = PartLook {
            ground: Some(Background::Color(Color::from_rgb(0.1, 0.2, 0.3))),
            ink: Some(Color::WHITE),
            ..PartLook::default()
        };
        assert_eq!(colours(Some(&look)), ["bg=#1a334d", "fg=#ffffff"]);
    }
}
