//! Settings → Themes' guided creator. The state here mirrors the owner's
//! `creator.slint` steps while all derivation, package writing and export
//! stay in `yapfast_core::themes::creator`.

use std::{
    collections::BTreeMap,
    path::{Component, Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use iced::{
    Alignment, Color, Element, Length, Task,
    advanced::image::Handle,
    widget::{Space, button, column, container, image, row},
};
use yapfast_core::{
    theme::{self as palette, Colour, Custom},
    themes::creator::{self as core, Background, Density, Messages, Spec},
};

use super::{Event as SettingsEvent, Page, Section, Settings, appearance};
use crate::{
    app::Message,
    controls::{Kind, action, slider},
    ctx::{Cx, Pref as Shown, Request},
    form, themes,
};

const HISTORY: usize = 64;
const TWO_COLUMNS: f32 = 520.0;
const COLOUR_GAP: f32 = 24.0;
static GENERATION: AtomicU64 = AtomicU64::new(1);

/// One creator step, in the owner's order. Advanced fine-tunes every derived
/// colour without throwing away the simple choices behind it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Step {
    #[default]
    Start,
    Colours,
    Shape,
    Fonts,
    Advanced,
}

impl Step {
    const ALL: [Self; 5] = [
        Self::Start,
        Self::Colours,
        Self::Shape,
        Self::Fonts,
        Self::Advanced,
    ];

    fn index(self) -> usize {
        Self::ALL.iter().position(|step| *step == self).unwrap_or(0)
    }

    fn name(self) -> &'static str {
        match self {
            Self::Start => "Start",
            Self::Colours => "Colours",
            Self::Shape => "Shape",
            Self::Fonts => "Fonts",
            Self::Advanced => "Advanced",
        }
    }
}

#[derive(Clone)]
struct Snapshot {
    spec: Spec,
    theme: palette::Theme,
    base: &'static str,
    assets: DraftAssets,
}

#[derive(Clone, Default)]
struct DraftAssets {
    picture: Option<Arc<[u8]>>,
    fonts: BTreeMap<String, Arc<[u8]>>,
}

/// The guided creator while open. Its theme is the fully derived palette
/// currently previewed; `fields` are Advanced's editable copies.
pub struct Creator {
    generation: u64,
    editing: Option<String>,
    pub step: Step,
    pub name: String,
    pub spec: Spec,
    pub theme: palette::Theme,
    pub base: &'static str,
    pub accent_hex: String,
    pub fields: Vec<(String, String, bool)>,
    pub note: Option<(String, bool)>,
    suggestions: Vec<Suggested>,
    confirming: bool,
    assets: DraftAssets,
    initial: Snapshot,
    history: Vec<Snapshot>,
}

/// The useful, cloneable part of a picture prepared off the UI thread.
#[derive(Clone, Debug)]
pub struct PictureArrival {
    jpeg: Vec<u8>,
    picture: core::Picture,
    accent: u32,
    ground: u32,
}

/// A newest picture found in the person's Pictures folders and made small
/// off the UI thread.
#[derive(Clone, Debug)]
pub struct Suggested {
    path: PathBuf,
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

impl Suggested {
    pub(crate) fn read(path: PathBuf) -> Option<Self> {
        let mut reader = ::image::ImageReader::open(&path)
            .and_then(|reader| reader.with_guessed_format())
            .ok()?;
        let mut limits = ::image::Limits::default();
        limits.max_image_width = Some(12_000);
        limits.max_image_height = Some(12_000);
        limits.max_alloc = Some(256 << 20);
        reader.limits(limits);
        let small = reader.decode().ok()?.thumbnail(168, 104).to_rgba8();
        Some(Self {
            path,
            width: small.width(),
            height: small.height(),
            rgba: small.into_raw(),
        })
    }
}

impl PictureArrival {
    pub(crate) fn prepare(bytes: &[u8]) -> Result<Self, String> {
        let prepared = core::prepare_picture(bytes)?;
        Ok(Self {
            jpeg: prepared.jpeg,
            picture: prepared.picture,
            accent: prepared.accent,
            ground: prepared.ground,
        })
    }
}

impl Creator {
    #[cfg(test)]
    pub(crate) fn generation(&self) -> u64 {
        self.generation
    }

    fn new(
        name: String,
        spec: Spec,
        theme: palette::Theme,
        base: &'static str,
        assets: DraftAssets,
        editing: Option<String>,
    ) -> Self {
        let initial = Snapshot {
            spec: spec.clone(),
            theme,
            base,
            assets: assets.clone(),
        };
        let mut creator = Self {
            generation: GENERATION.fetch_add(1, Ordering::Relaxed),
            editing,
            step: Step::Start,
            name,
            spec,
            theme,
            base,
            accent_hex: String::new(),
            fields: Vec::new(),
            note: None,
            suggestions: Vec::new(),
            confirming: false,
            assets,
            initial,
            history: Vec::new(),
        };
        creator.fill();
        creator
    }

    fn snapshot(&self) -> Snapshot {
        Snapshot {
            spec: self.spec.clone(),
            theme: self.theme,
            base: self.base,
            assets: self.assets.clone(),
        }
    }

    fn remember(&mut self) {
        if self.history.len() == HISTORY {
            self.history.remove(0);
        }
        self.history.push(self.snapshot());
        self.note = None;
    }

    fn restore(&mut self, snapshot: Snapshot) {
        self.spec = snapshot.spec;
        self.theme = snapshot.theme;
        self.base = snapshot.base;
        self.assets = snapshot.assets;
        self.fill();
        self.note = None;
    }

    fn fill(&mut self) {
        self.accent_hex = format!("#{:06x}", self.spec.accent);
        self.fields = palette::Theme::NAMES
            .iter()
            .enumerate()
            .filter_map(|(index, name)| {
                Some((
                    palette::token(name),
                    palette::hex(self.theme.colour(index)?),
                    false,
                ))
            })
            .collect();
    }

    fn valid(&self) -> bool {
        self.fields.iter().all(|(_, _, wrong)| !wrong)
    }

    fn derived(&self) -> core::Derived {
        based(
            &self.spec,
            core::finish(&self.spec, colours_of(&self.theme)),
        )
    }

    fn refresh(&mut self, recolour: bool) {
        let derived = if recolour {
            based(&self.spec, core::derive(&self.spec))
        } else {
            self.derived()
        };
        (self.base, self.theme) = palette_of(&derived);
        self.fill();
    }
}

#[derive(Clone, Debug)]
pub enum Event {
    Open,
    Step(Step),
    Name(String),
    Start(usize),
    Accent(String),
    AccentPreset(u32),
    Background(Background),
    PickPicture,
    PicturePicked(u64, Option<PathBuf>),
    PictureReady(u64, Result<PictureArrival, String>),
    Suggestions(u64, Vec<Suggested>),
    Suggested(usize),
    Blur(u8),
    Dim(u8),
    Neutral(bool),
    Messages(Messages),
    Corners(u8),
    Density(Density),
    Outline(u8),
    Glass(bool),
    Body(String),
    Headings(Option<String>),
    PickFont(bool),
    FontPicked(u64, bool, Option<PathBuf>),
    FontReady(u64, bool, Result<Vec<u8>, String>),
    AdvancedColour(usize, String),
    Undo,
    StartOver,
    Save,
    Export,
    ExportTo(u64, Option<PathBuf>),
    Delete,
    Cancel,
}

fn message(event: Event) -> Message {
    crate::app::settings(SettingsEvent::Theme(appearance::Event::Creator(event)))
}

fn pick_picture(generation: u64) -> Task<Message> {
    Task::perform(
        async {
            rfd::AsyncFileDialog::new()
                .set_title("Choose a picture for your theme")
                .add_filter("Pictures", &["jpg", "jpeg", "png", "webp", "bmp", "gif"])
                .pick_file()
                .await
                .map(|file| file.path().to_owned())
        },
        move |path| message(Event::PicturePicked(generation, path)),
    )
}

fn prepare_picture(generation: u64, path: PathBuf) -> Task<Message> {
    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || {
                let bytes = std::fs::read(path)
                    .map_err(|_| "That file isn't a picture Yapfast can use".to_owned())?;
                PictureArrival::prepare(&bytes)
            })
            .await
            .unwrap_or_else(|_| Err("That picture couldn't be read".to_owned()))
        },
        move |arrival| message(Event::PictureReady(generation, arrival)),
    )
}

fn pictures_folder() -> Option<PathBuf> {
    let home =
        std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" }).map(PathBuf::from)?;
    if cfg!(any(windows, target_os = "macos")) {
        return Some(home.join("Pictures"));
    }
    let config = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".config"));
    let text = std::fs::read_to_string(config.join("user-dirs.dirs")).ok()?;
    let line = text
        .lines()
        .find_map(|line| line.trim().strip_prefix("XDG_PICTURES_DIR="))?;
    let named = line.trim().trim_matches('"');
    let path = named.strip_prefix("$HOME").map_or_else(
        || PathBuf::from(named),
        |rest| home.join(rest.trim_start_matches('/')),
    );
    (path.is_absolute() && path != home).then_some(path)
}

fn newest_pictures() -> Vec<Suggested> {
    let Some(pictures) = pictures_folder() else {
        return Vec::new();
    };
    let mut found = Vec::new();
    for folder in [
        pictures.clone(),
        pictures.join("Wallpapers"),
        pictures.join("wallpapers"),
    ] {
        let Ok(entries) = std::fs::read_dir(folder) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let picture = path
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| {
                    matches!(
                        extension.to_ascii_lowercase().as_str(),
                        "jpg" | "jpeg" | "png" | "webp"
                    )
                });
            let Ok(metadata) = entry.metadata() else {
                continue;
            };
            if picture && metadata.is_file() && metadata.len() <= 40 << 20 {
                found.push((metadata.modified().unwrap_or(std::time::UNIX_EPOCH), path));
            }
        }
    }
    found.sort_by_key(|(modified, _)| std::cmp::Reverse(*modified));
    found
        .into_iter()
        .filter_map(|(_, path)| Suggested::read(path))
        .take(5)
        .collect()
}

fn suggest_pictures(generation: u64) -> Task<Message> {
    Task::perform(
        async move {
            tokio::task::spawn_blocking(newest_pictures)
                .await
                .unwrap_or_default()
        },
        move |pictures| message(Event::Suggestions(generation, pictures)),
    )
}

fn pick_font(generation: u64, headings: bool) -> Task<Message> {
    Task::perform(
        async move {
            (
                generation,
                headings,
                rfd::AsyncFileDialog::new()
                    .set_title("Import a font for your theme")
                    .add_filter("Fonts", &["ttf", "otf"])
                    .pick_file()
                    .await
                    .map(|file| file.path().to_owned()),
            )
        },
        |(generation, headings, path)| message(Event::FontPicked(generation, headings, path)),
    )
}

fn font_bytes(file: &str) -> Option<&'static [u8]> {
    Some(match file {
        "AtkinsonHyperlegibleNext-Regular.ttf" => {
            include_bytes!("../../../../ui/fonts/AtkinsonHyperlegibleNext-Regular.ttf")
        }
        "DisplayName-Tempo.ttf" => include_bytes!("../../../../ui/fonts/DisplayName-Tempo.ttf"),
        "DisplayName-Modern.ttf" => include_bytes!("../../../../ui/fonts/DisplayName-Modern.ttf"),
        "DisplayName-MonkeyBars.ttf" => {
            include_bytes!("../../../../ui/fonts/DisplayName-MonkeyBars.ttf")
        }
        "DisplayName-Journal.ttf" => {
            include_bytes!("../../../../ui/fonts/DisplayName-Journal.ttf")
        }
        "DisplayName-8Bit.ttf" => include_bytes!("../../../../ui/fonts/DisplayName-8Bit.ttf"),
        "DisplayName-Mainframe.ttf" => {
            include_bytes!("../../../../ui/fonts/DisplayName-Mainframe.ttf")
        }
        _ => return None,
    })
}

fn colours_of(theme: &palette::Theme) -> Vec<(&'static str, u32)> {
    core::BASE_TOKENS
        .iter()
        .filter_map(|token| {
            let index = palette::token_index(token)?;
            Some((*token, theme.colour(index)?.argb()))
        })
        .collect()
}

fn based(spec: &Spec, mut derived: core::Derived) -> core::Derived {
    if let Some(base) = spec
        .base
        .as_deref()
        .and_then(palette::find)
        .filter(|base| base.theme.dark == derived.dark)
    {
        derived.base = base.id;
    }
    derived
}

fn palette_of(derived: &core::Derived) -> (&'static str, palette::Theme) {
    let base = palette::find(derived.base).unwrap_or(&palette::BUILT_IN[0]);
    let mut theme = base.theme;
    theme.dark = derived.dark;
    for (token, colour) in &derived.colours {
        if let Some(slot) = palette::token_index(token).and_then(|index| theme.colour_mut(index)) {
            *slot = Colour::from_argb(*colour);
        }
    }
    (base.id, theme)
}

fn spec_from(theme: palette::Theme, base: &str) -> Spec {
    Spec {
        base: Some(base.to_owned()),
        accent: theme.accent.rgb_part(),
        ground: theme.chat.rgb_part(),
        background: if theme.dark {
            Background::Dark
        } else {
            Background::Light
        },
        ..Spec::default()
    }
}

fn creator_relative(id: &str) -> String {
    format!(
        "themes/creator/{}",
        core::slug(id.strip_prefix("custom:").unwrap_or(id))
    )
}

impl Settings {
    pub(super) fn creator_event(&mut self, event: Event, cx: &mut Cx<'_>) -> Task<Message> {
        match event {
            Event::Open => {
                self.open_creator(cx);
                return self
                    .creator
                    .as_ref()
                    .map_or_else(Task::none, |creator| suggest_pictures(creator.generation));
            }
            Event::Step(step) => {
                if let Some(creator) = self.creator.as_mut() {
                    creator.step = step;
                    if step == Step::Colours && creator.suggestions.is_empty() {
                        return suggest_pictures(creator.generation);
                    }
                }
            }
            Event::Name(name) => {
                if let Some(creator) = self.creator.as_mut() {
                    creator.name = name;
                }
            }
            Event::Start(index) => self.creator_start(index, cx),
            Event::Accent(text) => {
                if let Some(accent) = palette::parse(&text) {
                    self.creator_change(cx, true, |spec| {
                        let accent = accent.rgb_part();
                        spec.accent = accent;
                        if spec.background != Background::Picture && !neutral(spec.ground) {
                            spec.ground = accent;
                        }
                    });
                }
            }
            Event::AccentPreset(accent) => self.creator_change(cx, true, |spec| {
                spec.accent = accent;
                if spec.background != Background::Picture && !neutral(spec.ground) {
                    spec.ground = accent;
                }
            }),
            Event::Background(background) => {
                if background == Background::Picture
                    && self
                        .creator
                        .as_ref()
                        .is_none_or(|creator| creator.spec.picture.is_none())
                {
                    return self
                        .creator
                        .as_ref()
                        .map_or_else(Task::none, |creator| pick_picture(creator.generation));
                }
                self.creator_change(cx, true, |spec| spec.background = background)
            }
            Event::PickPicture => {
                return self
                    .creator
                    .as_ref()
                    .map_or_else(Task::none, |creator| pick_picture(creator.generation));
            }
            Event::PicturePicked(generation, path) => {
                if !self.creator_is(generation) {
                    return Task::none();
                }
                let Some(path) = path else {
                    return Task::none();
                };
                return prepare_picture(generation, path);
            }
            Event::PictureReady(generation, arrival) => {
                if self.creator_is(generation) {
                    self.creator_picture(arrival, cx);
                }
            }
            Event::Suggestions(generation, suggestions) => {
                if let Some(creator) = self
                    .creator
                    .as_mut()
                    .filter(|creator| creator.generation == generation)
                {
                    creator.suggestions = suggestions;
                }
            }
            Event::Suggested(index) => {
                let Some(creator) = self.creator.as_ref() else {
                    return Task::none();
                };
                let Some(suggestion) = creator.suggestions.get(index) else {
                    return Task::none();
                };
                return prepare_picture(creator.generation, suggestion.path.clone());
            }
            Event::Blur(blur) => self.creator_change(cx, false, |spec| spec.blur = blur.min(100)),
            Event::Dim(dim) => self.creator_change(cx, false, |spec| spec.dim = dim.min(100)),
            Event::Neutral(on) => self.creator_change(cx, true, |spec| {
                spec.ground = if on { 0x80_8080 } else { spec.accent };
            }),
            Event::Messages(messages) => {
                self.creator_change(cx, false, |spec| spec.messages = messages)
            }
            Event::Corners(corners) => {
                self.creator_change(cx, false, |spec| spec.corners = corners.min(100))
            }
            Event::Density(density) => {
                self.creator_change(cx, false, |spec| spec.density = density)
            }
            Event::Outline(outline) => {
                self.creator_change(cx, false, |spec| spec.outline = outline.min(4))
            }
            Event::Glass(glass) => self.creator_change(cx, false, |spec| spec.glass = glass),
            Event::Body(body) => self.creator_change(cx, false, |spec| spec.body = body),
            Event::Headings(headings) => {
                self.creator_change(cx, false, |spec| spec.headings = headings)
            }
            Event::PickFont(headings) => {
                return self.creator.as_ref().map_or_else(Task::none, |creator| {
                    pick_font(creator.generation, headings)
                });
            }
            Event::FontPicked(generation, headings, path) => {
                if !self.creator_is(generation) {
                    return Task::none();
                }
                let Some(path) = path else {
                    return Task::none();
                };
                return Task::perform(
                    async move { std::fs::read(path).map_err(|_| core::NOT_A_FONT.to_owned()) },
                    move |bytes| message(Event::FontReady(generation, headings, bytes)),
                );
            }
            Event::FontReady(generation, headings, bytes) => {
                if self.creator_is(generation) {
                    self.creator_font(headings, bytes, cx);
                }
            }
            Event::AdvancedColour(index, text) => self.creator_colour(index, text, cx),
            Event::Undo => {
                let restored = self
                    .creator
                    .as_mut()
                    .and_then(|creator| creator.history.pop().map(|before| (creator, before)));
                if let Some((creator, before)) = restored {
                    creator.restore(before);
                    if let Err(error) = Self::sync_draft(cx.client.root(), creator) {
                        creator.note =
                            Some((format!("The draft couldn't be restored: {error}"), true));
                    }
                    self.show_creator(cx);
                }
            }
            Event::StartOver => {
                if let Some(creator) = self.creator.as_mut() {
                    let initial = creator.initial.clone();
                    creator.remember();
                    creator.restore(initial);
                    if let Err(error) = Self::sync_draft(cx.client.root(), creator) {
                        creator.note =
                            Some((format!("The draft couldn't be restored: {error}"), true));
                    }
                    self.show_creator(cx);
                }
            }
            Event::Save => self.save_creator(cx),
            Event::Export => {
                if !self.prepare_creator_draft(cx) {
                    return Task::none();
                }
                let generation = self
                    .creator
                    .as_ref()
                    .map(|creator| creator.generation)
                    .unwrap_or_default();
                return Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .set_title("Export the theme as a package")
                            .pick_folder()
                            .await
                            .map(|folder| folder.path().to_owned())
                    },
                    move |path| message(Event::ExportTo(generation, path)),
                );
            }
            Event::ExportTo(generation, path) => {
                if self.creator_is(generation) && path.is_some() && self.prepare_creator_draft(cx) {
                    self.export_creator(path, cx);
                }
            }
            Event::Delete => self.delete_creator(cx),
            Event::Cancel => self.close_creator(cx),
        }
        Task::none()
    }

    fn open_creator(&mut self, cx: &mut Cx<'_>) {
        self.close_editor(cx);
        let chosen = cx.themes.chosen_id();
        let (base, theme, name, source) = if let Some(built_in) = palette::find(chosen) {
            (
                built_in.id,
                built_in.theme,
                format!("{} copy", built_in.name),
                None,
            )
        } else if let Some(custom) = cx.themes.custom.iter().find(|custom| custom.id == chosen) {
            (
                custom.base,
                custom.theme,
                format!("{} copy", custom.name),
                custom.folder.clone(),
            )
        } else {
            let built_in = &palette::BUILT_IN[0];
            (built_in.id, built_in.theme, "My theme".to_owned(), None)
        };
        let mut spec = cx
            .themes
            .custom
            .iter()
            .find(|custom| custom.id == chosen)
            .and_then(|custom| custom.creator.clone())
            .unwrap_or_else(|| spec_from(theme, base));
        let assets = Self::source_assets(cx.client.root(), source.as_deref(), &mut spec);
        self.creator = Some(Creator::new(name, spec, theme, base, assets, None));
        if let Some(creator) = self.creator.as_mut()
            && let Err(error) = Self::sync_draft(cx.client.root(), creator)
        {
            creator.note = Some((format!("The draft couldn't be opened: {error}"), true));
        }
        self.section = Some(Section::Themes);
        self.show_creator(cx);
    }

    pub(super) fn edit_creator(&mut self, id: &str, cx: &mut Cx<'_>) -> bool {
        let Some(custom) = cx
            .themes
            .custom
            .iter()
            .find(|custom| custom.id == id && custom.creator.is_some())
            .cloned()
        else {
            return false;
        };
        self.close_editor(cx);
        let mut spec = custom.creator.clone().expect("checked above");
        let assets = Self::source_assets(cx.client.root(), custom.folder.as_deref(), &mut spec);
        self.creator = Some(Creator::new(
            custom.name,
            spec,
            custom.theme,
            custom.base,
            assets,
            Some(custom.id),
        ));
        if let Some(creator) = self.creator.as_mut()
            && let Err(error) = Self::sync_draft(cx.client.root(), creator)
        {
            creator.note = Some((format!("The draft couldn't be opened: {error}"), true));
        }
        self.section = Some(Section::Themes);
        self.show_creator(cx);
        true
    }

    fn creator_start(&mut self, index: usize, cx: &mut Cx<'_>) {
        let Some(start) = palette::starts(&cx.themes.custom).nth(index) else {
            return;
        };
        let custom = cx.themes.custom.iter().find(|custom| custom.id == start.id);
        let mut spec = custom
            .and_then(|custom| custom.creator.clone())
            .unwrap_or_else(|| spec_from(start.theme, start.base));
        let source = custom.and_then(|custom| custom.folder.as_deref());
        let assets = Self::source_assets(cx.client.root(), source, &mut spec);
        let (name, theme, base) = (format!("{} copy", start.name), start.theme, start.base);
        if let Some(creator) = self.creator.as_mut() {
            creator.remember();
            creator.name = name;
            creator.spec = spec;
            creator.theme = theme;
            creator.base = base;
            creator.assets = assets;
            creator.fill();
            if let Err(error) = Self::sync_draft(cx.client.root(), creator) {
                creator.note = Some((format!("The draft couldn't be opened: {error}"), true));
            }
        }
        self.show_creator(cx);
    }

    fn creator_change(&mut self, cx: &mut Cx<'_>, recolour: bool, change: impl FnOnce(&mut Spec)) {
        let Some(creator) = self.creator.as_mut() else {
            return;
        };
        creator.remember();
        change(&mut creator.spec);
        creator.refresh(recolour);
        self.show_creator(cx);
    }

    fn creator_colour(&mut self, index: usize, text: String, cx: &mut Cx<'_>) {
        let Some(creator) = self.creator.as_mut() else {
            return;
        };
        let parsed = palette::parse(&text);
        let Some(field) = creator.fields.get_mut(index) else {
            return;
        };
        (field.1, field.2) = (text, parsed.is_none());
        if let (Some(colour), Some(slot)) = (parsed, creator.theme.colour_mut(index))
            && *slot != colour
        {
            creator.remember();
            if let Some(slot) = creator.theme.colour_mut(index) {
                *slot = colour;
            }
            creator.fill();
            self.show_creator(cx);
        }
    }

    fn show_creator(&mut self, cx: &mut Cx<'_>) {
        let Some(creator) = self.creator.as_ref() else {
            return;
        };
        let fallback = themes::theme(themes::CREATOR_PREVIEW_ID, &creator.theme);
        let folder = Self::draft_folder(cx);
        let name = if creator.name.trim().is_empty() {
            "My theme"
        } else {
            creator.name.trim()
        };
        let derived = creator.derived();
        let ready = Self::sync_draft(cx.client.root(), creator)
            .map_err(|error| error.to_string())
            .and_then(|()| {
                core::write_folder(
                    &folder,
                    &creator.spec,
                    name,
                    themes::CREATOR_PREVIEW_ID,
                    &derived,
                    font_bytes,
                )
                .map_err(|error| error.to_string())
            });
        self.preview = Some(
            match ready.and_then(|()| cx.themes.preview_creator(&folder, cx.layout)) {
                Ok(preview) => preview,
                Err(error) => {
                    cx.themes.clear_creator_preview();
                    if let Some(creator) = self.creator.as_mut() {
                        creator.note =
                            Some((format!("The preview couldn't be shown: {error}"), true));
                    }
                    fallback
                }
            },
        );
        cx.request(Request::Changed(Shown::Theme));
    }

    pub(super) fn close_creator(&mut self, cx: &mut Cx<'_>) {
        if self.creator.take().is_some() {
            cx.themes.clear_creator_preview();
            std::fs::remove_dir_all(Self::draft_folder(cx)).ok();
            if self.preview.take().is_some() {
                cx.request(Request::Changed(Shown::Theme));
            }
        }
    }

    fn creator_is(&self, generation: u64) -> bool {
        self.creator
            .as_ref()
            .is_some_and(|creator| creator.generation == generation)
    }

    fn source_assets(root: &Path, source: Option<&str>, spec: &mut Spec) -> DraftAssets {
        let mut assets = DraftAssets::default();
        let source = source.map(|source| root.join(source));
        if spec.picture.is_some() {
            assets.picture = source
                .as_ref()
                .and_then(|source| std::fs::read(source.join(core::PICTURE)).ok())
                .map(Arc::from);
            if assets.picture.is_none() {
                spec.picture = None;
                if spec.background == Background::Picture {
                    spec.background = Background::Dark;
                }
            }
        }
        spec.fonts.retain(|font| {
            let safe = Path::new(&font.file)
                .components()
                .all(|part| matches!(part, Component::Normal(_)))
                && font.file.starts_with("fonts/");
            let bytes = safe
                .then_some(source.as_ref())
                .flatten()
                .and_then(|source| std::fs::read(source.join(&font.file)).ok());
            if let Some(bytes) = bytes {
                assets.fonts.insert(font.file.clone(), Arc::from(bytes));
                true
            } else {
                false
            }
        });
        let built_in = |key: &str| core::FONTS.iter().any(|font| font.key == key);
        if !built_in(&spec.body) && !assets.fonts.contains_key(&spec.body) {
            spec.body = "default".to_owned();
        }
        if spec
            .headings
            .as_deref()
            .is_some_and(|key| !built_in(key) && !assets.fonts.contains_key(key))
        {
            spec.headings = None;
        }
        assets
    }

    fn sync_draft(root: &Path, creator: &Creator) -> std::io::Result<()> {
        let draft = root.join("themes/creator/draft");
        match std::fs::remove_dir_all(&draft) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
        if let Some(picture) = creator.assets.picture.as_deref() {
            core::store_picture(&draft, picture)?;
        }
        for (file, bytes) in &creator.assets.fonts {
            let path = draft.join(file);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(path, bytes)?;
        }
        Ok(())
    }

    fn creator_picture(&mut self, arrival: Result<PictureArrival, String>, cx: &mut Cx<'_>) {
        let arrival = match arrival {
            Ok(arrival) => arrival,
            Err(reason) => {
                if let Some(creator) = self.creator.as_mut() {
                    creator.note = Some((reason, true));
                }
                return;
            }
        };
        let Some(creator) = self.creator.as_mut() else {
            return;
        };
        let first = creator.spec.picture.is_none();
        creator.remember();
        creator.assets.picture = Some(Arc::from(arrival.jpeg));
        creator.spec.picture = Some(arrival.picture);
        creator.spec.background = Background::Picture;
        creator.spec.accent = arrival.accent;
        creator.spec.ground = arrival.ground;
        if first {
            creator.spec.glass = true;
        }
        creator.refresh(true);
        creator.note = Some(("Added the picture and used its colours.".to_owned(), false));
        if let Err(error) = Self::sync_draft(cx.client.root(), creator) {
            creator.note = Some((format!("The picture couldn't be kept: {error}"), true));
        }
        self.show_creator(cx);
    }

    fn creator_font(&mut self, headings: bool, bytes: Result<Vec<u8>, String>, cx: &mut Cx<'_>) {
        let bytes = match bytes {
            Ok(bytes) => bytes,
            Err(reason) => {
                if let Some(creator) = self.creator.as_mut() {
                    creator.note = Some((reason, true));
                }
                return;
            }
        };
        let imported = match core::store_font(&Self::draft_folder(cx), &bytes) {
            Ok(imported) => imported,
            Err(reason) => {
                if let Some(creator) = self.creator.as_mut() {
                    creator.note = Some((reason, true));
                }
                return;
            }
        };
        let mut note = format!("Added {}.", imported.family);
        if imported.partial {
            note.push_str(" Some characters will use the default font.");
        }
        note.push_str(" Only share themes with fonts you're allowed to share.");
        let Some(creator) = self.creator.as_mut() else {
            return;
        };
        creator.remember();
        creator
            .assets
            .fonts
            .insert(imported.file.clone(), Arc::from(bytes));
        let file = imported.file.clone();
        creator.spec.fonts.retain(|font| font.file != imported.file);
        creator.spec.fonts.push(imported);
        if headings {
            creator.spec.headings = Some(file);
        } else {
            creator.spec.body = file;
        }
        creator.refresh(false);
        creator.note = Some((note, false));
        if let Err(error) = Self::sync_draft(cx.client.root(), creator) {
            creator.note = Some((format!("The font couldn't be kept: {error}"), true));
        }
        self.show_creator(cx);
    }

    fn save_creator(&mut self, cx: &mut Cx<'_>) {
        let Some(creator) = self.creator.as_ref() else {
            return;
        };
        if !creator.valid() {
            return;
        }
        let editing = creator.editing.clone();
        let id = editing
            .clone()
            .unwrap_or_else(|| palette::new_id(&cx.themes.custom));
        // A stored Custom.folder can come from older layout data and is not
        // authority to overwrite that path. Creator saves always land in the
        // managed tree derived from the validated custom id.
        let relative = creator_relative(&id);
        let folder = cx.client.root().join(&relative);
        let name = if creator.name.trim().is_empty() {
            "My theme".to_owned()
        } else {
            creator.name.trim().to_owned()
        };
        let spec = creator.spec.clone();
        let derived = creator.derived();
        let (base, theme) = palette_of(&derived);
        let draft = Self::draft_folder(cx);
        if let Err(error) = Self::sync_draft(cx.client.root(), creator) {
            if let Some(creator) = self.creator.as_mut() {
                creator.note = Some((format!("The theme couldn't be saved: {error}"), true));
            }
            return;
        }
        let written = core::write_folder(&draft, &spec, &name, &id, &derived, font_bytes)
            .and_then(|()| core::copy_folder(&draft, &folder));
        if let Err(error) = written {
            if let Some(creator) = self.creator.as_mut() {
                creator.note = Some((format!("The theme couldn't be saved: {error}"), true));
            }
            return;
        }
        let custom = Custom {
            id: id.clone(),
            name,
            base,
            theme,
            creator: Some(spec),
            folder: Some(relative),
        };
        if let Some(saved) = editing
            .as_ref()
            .and_then(|id| cx.themes.custom.iter_mut().find(|custom| custom.id == *id))
        {
            *saved = custom;
        } else {
            cx.themes.custom.push(custom);
        }
        cx.layout.custom_themes.clone_from(&cx.themes.custom);
        cx.keep("custom_themes", &cx.themes.custom);
        cx.themes.refresh(cx.client.root(), cx.layout);
        cx.themes.clear_creator_preview();
        self.creator = None;
        self.preview = None;
        std::fs::remove_dir_all(draft).ok();
        cx.themes.choose(cx.client.root(), &id);
        cx.request(Request::Changed(Shown::Theme));
    }

    fn delete_creator(&mut self, cx: &mut Cx<'_>) {
        let Some(creator) = self.creator.as_mut() else {
            return;
        };
        let Some(id) = creator.editing.clone() else {
            return;
        };
        if !creator.confirming {
            creator.confirming = true;
            return;
        }
        let folder = cx
            .themes
            .custom
            .iter()
            .find(|custom| custom.id == id)
            .and_then(|custom| custom.folder.as_deref());
        if let Some(folder) = folder
            && let Err(error) = Self::remove_creator_folder(cx.client.root(), folder)
        {
            creator.note = Some((format!("The theme couldn't be deleted: {error}"), true));
            creator.confirming = false;
            return;
        }
        let changed = cx.themes.forget_custom(cx.client.root(), &id);
        cx.layout.custom_themes.clone_from(&cx.themes.custom);
        cx.keep("custom_themes", &cx.themes.custom);
        self.close_creator(cx);
        if changed {
            cx.request(Request::Changed(Shown::Theme));
        }
    }

    fn remove_creator_folder(root: &Path, named: &str) -> std::io::Result<()> {
        let owned = root.join("themes/creator");
        let target = root.join(named);
        let owned = std::fs::canonicalize(owned)?;
        let target = match std::fs::canonicalize(&target) {
            Ok(target) => target,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(error),
        };
        if target == owned || target.strip_prefix(&owned).is_err() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "its folder is not owned by the theme creator",
            ));
        }
        std::fs::remove_dir_all(target)
    }

    fn draft_folder(cx: &Cx<'_>) -> PathBuf {
        cx.client.root().join("themes/creator/draft")
    }

    fn prepare_creator_draft(&mut self, cx: &mut Cx<'_>) -> bool {
        let Some(creator) = self.creator.as_ref() else {
            return false;
        };
        let name = creator.name.trim();
        let derived = creator.derived();
        if let Err(error) = Self::sync_draft(cx.client.root(), creator) {
            if let Some(creator) = self.creator.as_mut() {
                creator.note = Some((format!("The theme couldn't be exported: {error}"), true));
            }
            return false;
        }
        match core::write_folder(
            &Self::draft_folder(cx),
            &creator.spec,
            name,
            "draft",
            &derived,
            font_bytes,
        ) {
            Ok(()) => true,
            Err(error) => {
                if let Some(creator) = self.creator.as_mut() {
                    creator.note = Some((format!("The theme couldn't be exported: {error}"), true));
                }
                false
            }
        }
    }

    fn export_creator(&mut self, parent: Option<PathBuf>, cx: &mut Cx<'_>) {
        let Some(parent) = parent else {
            return;
        };
        let Some(creator) = self.creator.as_ref() else {
            return;
        };
        let folder_name = core::slug(if creator.name.trim().is_empty() {
            "my theme"
        } else {
            creator.name.trim()
        });
        let target = parent.join(folder_name);
        let result = core::export_folder(&Self::draft_folder(cx), &target);
        if let Some(creator) = self.creator.as_mut() {
            creator.note = Some(match result {
                Ok(()) => (
                    format!("Exported the package to {}.", target.display()),
                    false,
                ),
                Err(error) => (format!("The theme couldn't be exported: {error}"), true),
            });
        }
    }
}

fn neutral(ground: u32) -> bool {
    core::lch(ground)[1] < 0.004
}

fn choice(title: &'static str, colour: Option<Color>) -> form::Choice<'static> {
    form::Choice {
        title: title.into(),
        description: "".into(),
        colour,
        offered: true,
    }
}

impl<'a> Page<'a> {
    pub(super) fn creator_view(self, creator: &'a Creator) -> Element<'a, Message> {
        let theme = self.cx.look.theme;
        let mut top = row![
            form::field(theme, "creator-name", "Theme name", &creator.name)
                .on_input(|name| message(Event::Name(name)))
                .on_submit(message(Event::Save)),
            action(
                theme,
                "Save",
                Kind::Primary,
                creator.valid().then(|| message(Event::Save))
            ),
            action(theme, "Export", Kind::Plain, Some(message(Event::Export))),
            action(theme, "Cancel", Kind::Plain, Some(message(Event::Cancel))),
        ]
        .spacing(8)
        .align_y(Alignment::Center);
        if creator.editing.is_some() {
            top = top.push(action(
                theme,
                if creator.confirming {
                    "Delete it for good"
                } else {
                    "Delete theme"
                },
                if creator.confirming {
                    Kind::Danger
                } else {
                    Kind::Plain
                },
                Some(message(Event::Delete)),
            ));
        }
        if !creator.history.is_empty() {
            top = top.push(action(theme, "Undo", Kind::Plain, Some(message(Event::Undo))).small());
        }

        let steps = form::options(
            theme,
            Step::ALL
                .into_iter()
                .map(|step| choice(step.name(), None))
                .collect(),
            Some(creator.step.index()),
            true,
            |index| message(Event::Step(Step::ALL[index])),
        );

        let body = match creator.step {
            Step::Start => self.creator_start_view(creator),
            Step::Colours => self.creator_colours_view(creator),
            Step::Shape => self.creator_shape_view(creator),
            Step::Fonts => self.creator_fonts_view(creator),
            Step::Advanced => self.creator_advanced_view(creator),
        };
        let at = creator.step.index();
        let navigation = row![
            action(
                theme,
                "Previous",
                Kind::Plain,
                at.checked_sub(1)
                    .map(|index| message(Event::Step(Step::ALL[index]))),
            ),
            action(
                theme,
                "Next",
                Kind::Primary,
                (at + 1 < Step::ALL.len()).then(|| message(Event::Step(Step::ALL[at + 1]))),
            ),
            action(
                theme,
                "Start over",
                Kind::Plain,
                Some(message(Event::StartOver))
            )
            .small(),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        let note: Element<'a, Message> = creator.note.as_ref().map_or_else(
            || Space::new().height(0).into(),
            |(note, bad)| {
                form::note_in(
                    note.clone(),
                    if *bad { theme.danger_text } else { theme.live },
                )
            },
        );
        form::page([
            form::title(theme, "Create theme"),
            form::note(
                theme,
                "A guided owner theme: every step updates the window preview. Save keeps its creator choices and package folder.",
            ),
            top.into(),
            steps,
            body,
            note,
            navigation.into(),
        ])
    }

    fn creator_start_view(self, creator: &'a Creator) -> Element<'a, Message> {
        let theme = self.cx.look.theme;
        let starts: Vec<_> = palette::starts(&self.cx.themes.custom).collect();
        let choices = starts
            .iter()
            .map(|start| form::Choice {
                title: start.name.into(),
                description: "Use this theme's colours as the starting point.".into(),
                colour: Some(themes::color(start.theme.accent)),
                offered: true,
            })
            .collect();
        let selected = starts.iter().position(|start| start.base == creator.base);
        form::group([
            form::heading(theme, "START FROM"),
            form::note(
                theme,
                "Choose a starting point. You can undo this or return to how the creator opened.",
            ),
            form::options(theme, choices, selected, true, |index| {
                message(Event::Start(index))
            }),
        ])
    }

    fn creator_colours_view(self, creator: &'a Creator) -> Element<'a, Message> {
        let theme = self.cx.look.theme;
        let accents = core::ACCENTS
            .iter()
            .map(|accent| form::Choice {
                title: format!("#{accent:06x}").into(),
                description: "".into(),
                colour: Some(Color::from_rgb8(
                    (accent >> 16) as u8,
                    (accent >> 8) as u8,
                    *accent as u8,
                )),
                offered: true,
            })
            .collect();
        let accent_at = core::ACCENTS
            .iter()
            .position(|accent| *accent == creator.spec.accent);
        let backgrounds = vec![
            choice("Dark", None),
            choice("Light", None),
            choice("Picture", None),
        ];
        let background = match creator.spec.background {
            Background::Dark => 0,
            Background::Light => 1,
            Background::Picture => 2,
        };
        let tint = vec![choice("Tinted", None), choice("Neutral greys", None)];
        let mut background_controls = vec![
            form::heading(theme, "BACKGROUND"),
            form::options(theme, backgrounds, Some(background), true, |index| {
                message(Event::Background(match index {
                    1 => Background::Light,
                    2 => Background::Picture,
                    _ => Background::Dark,
                }))
            }),
            action(
                theme,
                if creator.spec.picture.is_some() {
                    "Change picture…"
                } else {
                    "Choose picture…"
                },
                Kind::Plain,
                Some(message(Event::PickPicture)),
            )
            .into(),
        ];
        if !creator.suggestions.is_empty() {
            let mut thumbnails: Vec<Element<'a, Message>> = Vec::new();
            for (index, suggestion) in creator.suggestions.iter().enumerate() {
                let thumbnail =
                    Handle::from_rgba(suggestion.width, suggestion.height, suggestion.rgba.clone());
                thumbnails.push(
                    button(
                        image(thumbnail)
                            .width(168)
                            .height(104)
                            .content_fit(iced::ContentFit::Cover),
                    )
                    .padding(0)
                    .on_press(message(Event::Suggested(index)))
                    .into(),
                );
            }
            let recent = row(thumbnails).spacing(8).wrap();
            background_controls.push(form::heading(theme, "RECENT PICTURES"));
            background_controls.push(recent.into());
        }
        if let Some(picture) = creator.spec.picture.as_ref() {
            background_controls.push(form::options(
                theme,
                picture
                    .palette
                    .iter()
                    .take(5)
                    .map(|colour| {
                        choice(
                            "Picture colour",
                            Some(Color::from_rgb8(
                                (colour >> 16) as u8,
                                (colour >> 8) as u8,
                                *colour as u8,
                            )),
                        )
                    })
                    .collect(),
                None,
                false,
                |_| message(Event::PickPicture),
            ));
            background_controls.push(
                slider("Picture blur", u32::from(creator.spec.blur), 100, |value| {
                    message(Event::Blur(value as u8))
                })
                .step(1)
                .shown(format!("{}%", creator.spec.blur))
                .mark(false)
                .view(theme),
            );
            background_controls.push(
                slider("Picture dim", u32::from(creator.spec.dim), 100, |value| {
                    message(Event::Dim(value as u8))
                })
                .step(1)
                .shown(format!("{}%", creator.spec.dim))
                .mark(false)
                .view(theme),
            );
        }
        form::page([
            form::group([
                form::heading(theme, "ACCENT"),
                form::options(theme, accents, accent_at, true, |index| {
                    message(Event::AccentPreset(core::ACCENTS[index]))
                }),
                form::field(
                    theme,
                    "creator-accent",
                    "Accent as #rrggbb",
                    &creator.accent_hex,
                )
                .on_input(|text| message(Event::Accent(text)))
                .into(),
            ]),
            form::group(background_controls),
            form::group([
                form::heading(theme, "GROUND TINT"),
                form::options(
                    theme,
                    tint,
                    Some(usize::from(neutral(creator.spec.ground))),
                    true,
                    |index| message(Event::Neutral(index == 1)),
                ),
            ]),
        ])
    }

    fn creator_shape_view(self, creator: &'a Creator) -> Element<'a, Message> {
        let theme = self.cx.look.theme;
        let messages = [
            Messages::Flat,
            Messages::Bubbles,
            Messages::Cards,
            Messages::Irc,
        ];
        let densities = [Density::Compact, Density::Cozy, Density::Roomy];
        form::page([
            form::group([
                form::heading(theme, "MESSAGES"),
                form::options(
                    theme,
                    ["Flat", "Bubbles", "Cards", "IRC"]
                        .into_iter()
                        .map(|name| choice(name, None))
                        .collect(),
                    messages
                        .iter()
                        .position(|value| *value == creator.spec.messages),
                    true,
                    |index| message(Event::Messages(messages[index])),
                ),
            ]),
            form::group([
                form::heading(theme, "CORNERS"),
                slider(
                    "Square to round",
                    u32::from(creator.spec.corners),
                    100,
                    |value| message(Event::Corners(value as u8)),
                )
                .step(1)
                .mark(false)
                .view(theme),
            ]),
            form::group([
                form::heading(theme, "DENSITY"),
                form::options(
                    theme,
                    ["Compact", "Cozy", "Roomy"]
                        .into_iter()
                        .map(|name| choice(name, None))
                        .collect(),
                    densities
                        .iter()
                        .position(|value| *value == creator.spec.density),
                    true,
                    |index| message(Event::Density(densities[index])),
                ),
            ]),
            form::group([
                form::heading(theme, "OUTLINES"),
                slider(
                    "Outline width",
                    u32::from(creator.spec.outline),
                    4,
                    |value| message(Event::Outline(value as u8)),
                )
                .step(1)
                .shown(format!("{} px", creator.spec.outline))
                .mark(false)
                .view(theme),
            ]),
            crate::form::switch(
                "Frosted glass panes",
                creator.spec.glass,
                Some(message(Event::Glass(!creator.spec.glass))),
            )
            .description("The sidebar and conversation show the background through frosted panes.")
            .view(self.cx.look),
        ])
    }

    fn creator_fonts_view(self, creator: &'a Creator) -> Element<'a, Message> {
        let theme = self.cx.look.theme;
        let mut body_keys = core::FONTS
            .iter()
            .map(|font| font.key.to_owned())
            .collect::<Vec<_>>();
        let mut fonts = core::FONTS
            .iter()
            .map(|font| form::Choice {
                title: font.label.into(),
                description: font.typeface.into(),
                colour: None,
                offered: true,
            })
            .collect::<Vec<_>>();
        for font in &creator.spec.fonts {
            body_keys.push(font.file.clone());
            fonts.push(form::Choice {
                title: font.family.clone().into(),
                description: "Imported font".into(),
                colour: None,
                offered: true,
            });
        }
        let body = body_keys.iter().position(|key| *key == creator.spec.body);
        let mut heading_fonts = vec![form::Choice {
            title: "Same as body".into(),
            description: "".into(),
            colour: None,
            offered: true,
        }];
        heading_fonts.extend(core::FONTS.iter().map(|font| form::Choice {
            title: font.label.into(),
            description: font.typeface.into(),
            colour: None,
            offered: true,
        }));
        let mut heading_keys = vec![None];
        heading_keys.extend(core::FONTS.iter().map(|font| Some(font.key.to_owned())));
        for font in &creator.spec.fonts {
            heading_keys.push(Some(font.file.clone()));
            heading_fonts.push(form::Choice {
                title: font.family.clone().into(),
                description: "Imported font".into(),
                colour: None,
                offered: true,
            });
        }
        let headings = heading_keys
            .iter()
            .position(|key| key == &creator.spec.headings)
            .unwrap_or(0);
        form::page([
            form::group([
                form::heading(theme, "BODY FONT"),
                form::options(theme, fonts, body, true, move |index| {
                    message(Event::Body(body_keys[index].clone()))
                }),
                action(
                    theme,
                    "Import body font…",
                    Kind::Plain,
                    Some(message(Event::PickFont(false))),
                )
                .into(),
            ]),
            form::group([
                form::heading(theme, "HEADING FONT"),
                form::options(theme, heading_fonts, Some(headings), true, move |index| {
                    message(Event::Headings(heading_keys[index].clone()))
                }),
                action(
                    theme,
                    "Import heading font…",
                    Kind::Plain,
                    Some(message(Event::PickFont(true))),
                )
                .into(),
            ]),
            form::note(
                theme,
                "Imported fonts and built-in fonts are copied into the saved or exported package with the creator's licence notice.",
            ),
        ])
    }

    fn creator_advanced_view(self, creator: &'a Creator) -> Element<'a, Message> {
        let theme = self.cx.look.theme;
        let colours = iced::widget::responsive(move |size| {
            let columns = if size.width >= TWO_COLUMNS { 2 } else { 1 };
            let per = creator.fields.len().div_ceil(columns);
            let mut line = row![].spacing(COLOUR_GAP);
            for column_index in 0..columns {
                let mut cells = column![];
                for index in
                    column_index * per..((column_index + 1) * per).min(creator.fields.len())
                {
                    let (name, hex, wrong) = &creator.fields[index];
                    let shown = creator
                        .theme
                        .colour(index)
                        .map_or(Color::TRANSPARENT, themes::color);
                    cells = cells.push(form::colour_row(
                        theme,
                        name,
                        shown,
                        hex,
                        *wrong,
                        move |text| message(Event::AdvancedColour(index, text)),
                    ));
                }
                line = line.push(container(cells).width(Length::FillPortion(1)));
            }
            line.into()
        });
        form::group([
            form::heading(theme, "ADVANCED COLOURS"),
            form::note(
                theme,
                "Fine-tune the derived palette. The creator's shape and font choices stay behind it, and readability is worked out again on Save or Export.",
            ),
            colours.into(),
        ])
    }
}
