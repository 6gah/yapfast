//! `goto`: the private test display's way to a place in the window, as
//! `scripts/ui-session do 'goto TARGET'` asks. It sends the app's own
//! messages and the core's own actions, so a screen it opens is the screen
//! a click would have opened, and nothing is faked with a click at a
//! guessed spot.
//!
//! Targets: `home` (`dms`), `server NAME` or `server #INDEX` (a server on
//! the rail, counted from 0 among the servers), `channel NAME` (in the open
//! place first, then anywhere the core lists), `settings/PAGE` (such as
//! `settings/appearance`), `profile USER` (their card), and `menu N` (the
//! context menu of the Nth message in the timeline, counted from 0).

use iced::{Point, Task};
use yapfast_core::application::{Action, Navigation};

use crate::{
    app::{App, Message},
    rail::Kind,
    settings::Section,
    sidebar::SideRow,
};

/// Goes where `target` says; why it couldn't, when it couldn't.
pub fn run(app: &mut App, target: &str) -> Result<Task<Message>, String> {
    let target = target.trim();
    let (verb, rest) = target.split_once(' ').unwrap_or((target, ""));
    let rest = rest.trim();
    if let Some(page) = verb.strip_prefix("settings/").filter(|page| !page.is_empty()) {
        return settings(app, page);
    }
    match verb {
        "home" | "dms" => Ok(navigate(app, Navigation::DirectMessages)),
        "server" => server(app, rest),
        "channel" => channel(app, rest),
        "profile" => profile(app, rest),
        _ => Err(unknown(target)),
    }
}

/// The context menu of the `number`th message in the timeline.
pub fn menu(app: &mut App, number: usize) -> Result<Task<Message>, String> {
    let item = app
        .timeline
        .messages
        .get(number)
        .map(|message| message.item.to_owned())
        .ok_or_else(|| {
            format!(
                "no message {number}: the timeline holds {}",
                app.timeline.messages.len()
            )
        })?;
    let row = crate::timeline::row_of(app, &item)
        .ok_or_else(|| format!("message {number} has no row: it's not in the timeline"))?;
    // Where the menu opens: in the timeline, as a right click there does.
    let place = app.timeline.place.get();
    let at = if place.width > 0.0 {
        place.center()
    } else {
        Point::new(app.shell.frame.width / 2.0, app.shell.frame.height / 2.0)
    };
    Ok(crate::menus::open_menu(app, row, at))
}

/// A server on the rail, by name or by `#INDEX`. A DM home is `home`.
fn server(app: &mut App, query: &str) -> Result<Task<Message>, String> {
    let servers: Vec<(String, String)> = app
        .rail
        .items
        .iter()
        .filter(|item| matches!(item.kind, Kind::Server | Kind::Nested { .. }))
        .map(|item| (item.id.clone(), item.name.clone()))
        .collect();
    match resolve(&servers, query) {
        Some(id) => Ok(navigate(app, Navigation::Server(id))),
        None => Err(format!(
            "no server {query:?}; the rail lists {servers:?}"
        )),
    }
}

/// A channel by name: the open place's first, then anywhere the core lists.
fn channel(app: &mut App, query: &str) -> Result<Task<Message>, String> {
    let here: Vec<(String, String)> = app
        .sidebar
        .rows
        .iter()
        .filter_map(|row| match row {
            SideRow::Room(room) => Some((room.id.clone(), room.name.clone())),
            _ => None,
        })
        .collect();
    let in_place = resolve(&here, query);
    let id = match in_place {
        Some(id) => id,
        None => {
            let all: Vec<(String, String)> = app
                .client
                .state()
                .rooms
                .iter()
                .filter(|room| !room.direct)
                .map(|room| (room.id.clone(), room.name.clone()))
                .collect();
            resolve(&all, query).ok_or_else(|| {
                format!("no channel {query:?}; this place has {here:?}")
            })?
        }
    };
    // Another place's channel goes to its place first, as the server tree's
    // rows do.
    if resolve(&here, query).as_deref() != Some(id.as_str()) {
        app.client.dispatch(Action::Open(id.clone()));
    }
    app.client.dispatch(Action::SelectRoom(id));
    Ok(app.sync())
}

/// Someone's card, by the name of their DM or their id.
fn profile(app: &mut App, query: &str) -> Result<Task<Message>, String> {
    let people: Vec<(String, String)> = app
        .client
        .state()
        .rooms
        .iter()
        .filter_map(|room| room.person.clone().map(|person| (person, room.name.clone())))
        .collect();
    let user = if !query.is_empty() && query.chars().all(|character| character.is_ascii_digit()) {
        query.to_owned()
    } else {
        resolve(&people, query).ok_or_else(|| {
            format!("no person {query:?}; the DMs are {people:?}")
        })?
    };
    let at = Point::new(app.shell.frame.width / 2.0, app.shell.frame.height / 2.0);
    Ok(app.open_card(user, at))
}

/// A page of Settings, by the name its page goes by here.
fn settings(app: &mut App, page: &str) -> Result<Task<Message>, String> {
    let section = match page {
        "account" => Section::Account,
        "voice" | "voice-video" => Section::Voice,
        "messages" => Section::Messages,
        "notifications" => Section::Notifications,
        "discord" => Section::Discord,
        "sounds" => Section::Sounds,
        "advanced" => Section::Advanced,
        "appearance" => Section::Appearance,
        "themes" => Section::Themes,
        "games" | "game-activity" => Section::Games,
        "keybinds" => Section::Keybinds,
        "about" => Section::About,
        other => {
            return Err(format!(
                "no settings page {other:?}: account, voice, messages, \
                 notifications, discord, sounds, advanced, appearance, themes, \
                 games, keybinds, about"
            ));
        }
    };
    Ok(app.open_settings(section))
}

fn navigate(app: &mut App, navigation: Navigation) -> Task<Message> {
    app.client.dispatch(Action::Navigate(navigation));
    app.sync()
}

fn unknown(target: &str) -> String {
    format!(
        "no target {target:?}: home, dms, server NAME, channel NAME, \
         settings/PAGE, profile USER"
    )
}

/// The id of the place named `query`: `#INDEX` counts them in order, and a
/// name is matched whole first, then in part, without regard to case.
fn resolve(places: &[(String, String)], query: &str) -> Option<String> {
    if let Some(index) = query
        .strip_prefix('#')
        .and_then(|number| number.parse::<usize>().ok())
    {
        return places.get(index).map(|(id, _)| id.clone());
    }
    if query.is_empty() {
        return None;
    }
    let wanted = query.to_lowercase();
    places
        .iter()
        .find(|(_, name)| name.to_lowercase() == wanted)
        .or_else(|| {
            places
                .iter()
                .find(|(_, name)| name.to_lowercase().contains(&wanted))
        })
        .map(|(id, _)| id.clone())
}
