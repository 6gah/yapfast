//! The conversation column: what takes the timeline's place (the friends
//! screens, browsing, the forum, a call) and what stands beside it
//! (search, the thread, the member list, the profile panel).

use iced::{
    Alignment, Element, Length,
    widget::{column, container, row},
};

use crate::{
    app::{App, Message},
    controls::bar,
};
use yapfast_core::themes::layout::{ComposerSide, MembersSide, RailSide};

pub fn themed_navigation(
    layout: &crate::layout::Layout,
    theme: &crate::theme::Theme,
    package: Option<&yapfast_core::themes::layout::Layout>,
    chrome: Option<&yapfast_core::themes::chrome::Chrome>,
) -> f32 {
    let rail = if matches!(
        package.and_then(|layout| layout.rail),
        Some(RailSide::Hidden | RailSide::Top | RailSide::Tree)
    ) {
        0.0
    } else {
        crate::rail::resolved_width(package, chrome)
    };
    rail + crate::sidebar::width(layout, theme)
}

pub fn navigation_right(package: Option<&yapfast_core::themes::layout::Layout>) -> bool {
    package.and_then(|layout| layout.rail) == Some(RailSide::Right)
}

pub fn navigation_ctx(ctx: crate::ctx::Ctx<'_>) -> f32 {
    themed_navigation(
        ctx.layout,
        ctx.look.theme,
        ctx.themes.package_layout(&ctx.look.theme.id),
        ctx.themes.chrome(&ctx.look.theme.id),
    )
}

/// The navigation's share of the conversation's left edge. A right-docked
/// rail/sidebar still consumes width, but does not move window coordinates
/// measured from the chat's left.
pub fn navigation_left_ctx(ctx: crate::ctx::Ctx<'_>) -> f32 {
    let package = ctx.themes.package_layout(&ctx.look.theme.id);
    if navigation_right(package) {
        0.0
    } else {
        themed_navigation(
            ctx.layout,
            ctx.look.theme,
            package,
            ctx.themes.chrome(&ctx.look.theme.id),
        )
    }
}

/// Centres the message lane and composer at an installed theme's maximum
/// chat width. The surrounding chat surface still fills the window, so its
/// ground and art do not acquire an artificial narrow edge.
fn bounded_chat<'a>(
    content: impl Into<Element<'a, Message>>,
    package: Option<&yapfast_core::themes::layout::Layout>,
) -> Element<'a, Message> {
    let content = content.into();
    let Some(width) = package.and_then(|layout| layout.chat_max_width) else {
        return content;
    };
    container(container(content).width(Length::Fill).max_width(width))
        .width(Length::Fill)
        .align_x(Alignment::Center)
        .into()
}

fn themed_composer(app: &App) -> Element<'_, Message> {
    crate::theme_art::surface(
        crate::composer::view(app.ctx()),
        app.ctx(),
        yapfast_core::themes::package::art::ArtSurface::Composer,
    )
}

/// The conversation column: the header, the timeline and the composer,
/// with a server's member list beside them; or a voice channel's call.
pub fn conversation(app: &App) -> Element<'_, Message> {
    if crate::call::stage::voice_panel(app) {
        return crate::theme_art::surface(
            crate::call::stage::call_view(app),
            app.ctx(),
            yapfast_core::themes::package::art::ArtSurface::Call,
        );
    }
    let outline = app.theme.outline_chat;
    let line = bar(outline, Length::Fill, 1);
    if let Some(screen) = crate::friends::dm_screen(app) {
        let page = match screen {
            yapfast_core::application::DmScreen::Friends => {
                crate::friends::friends_screen(app.ctx())
            }
            yapfast_core::application::DmScreen::Requests => {
                crate::friends::requests::requests_screen(app.ctx())
            }
        };
        return column![
            crate::friends::friends_header(app.ctx(), screen),
            line,
            page
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into();
    }
    if let Some(screen) = crate::browse_channels::view(app.ctx()) {
        return screen;
    }
    let mut chat = column(crate::forum::post_bar_view(app.ctx()))
        .extend(crate::banners::view(app.ctx()))
        .width(Length::Fill)
        .height(Length::Fill);
    // A DM's call shows above its messages.
    if let Some(call) = crate::call::dm::dm_call_view(app) {
        chat = chat.push(call);
    }
    let package_layout = app.themes.package_layout(&app.theme.id);
    let chat = match crate::forum::view(app.ctx()) {
        Some(forum) => chat.push(forum),
        None if package_layout.and_then(|layout| layout.composer) == Some(ComposerSide::Top) => {
            chat.push(bounded_chat(themed_composer(app), package_layout))
                .push(bounded_chat(
                    crate::timeline::view::timeline(app.ctx()),
                    package_layout,
                ))
        }
        None => chat
            .push(bounded_chat(
                crate::timeline::view::timeline(app.ctx()),
                package_layout,
            ))
            .push(bounded_chat(themed_composer(app), package_layout)),
    };
    let chat = crate::theme_art::surface(
        chat,
        app.ctx(),
        yapfast_core::themes::package::art::ArtSurface::Chat,
    );
    let chat = crate::theme_transition::chat(app, chat);
    let members_side = package_layout
        .and_then(|layout| layout.members)
        .unwrap_or(MembersSide::Right);
    let mut members = (crate::header::members_shown(app) && members_side != MembersSide::Hidden)
        .then(|| {
            let members = crate::theme_art::surface(
                crate::members::view(&app.roster, app.ctx()),
                app.ctx(),
                yapfast_core::themes::package::art::ArtSurface::Members,
            );
            crate::theme_art::outlined(
                members,
                app.ctx(),
                yapfast_core::themes::layout::BorderPlace::Panes,
                "members",
                0.0,
                1.0,
                app.theme.outline,
            )
        });
    let members_left = members_side == MembersSide::Left;
    let mut body = row![];
    if members_left && let Some(members) = members.take() {
        body = body.push(members);
    }
    body = body.push(chat);
    if let Some(search) = crate::search::view(&app.searching, app.ctx()) {
        let search = crate::theme_art::surface(
            search,
            app.ctx(),
            yapfast_core::themes::package::art::ArtSurface::Aside,
        );
        body = body.push(crate::theme_art::outlined(
            search,
            app.ctx(),
            yapfast_core::themes::layout::BorderPlace::Panes,
            "panel",
            0.0,
            1.0,
            app.theme.outline,
        ));
    } else if let Some(aside) = crate::thread::aside_view(app.ctx()) {
        let aside = crate::theme_art::surface(
            aside,
            app.ctx(),
            yapfast_core::themes::package::art::ArtSurface::Aside,
        );
        body = body.push(crate::theme_art::outlined(
            aside,
            app.ctx(),
            yapfast_core::themes::layout::BorderPlace::Panes,
            "panel",
            0.0,
            1.0,
            app.theme.outline,
        ));
    } else if !members_left && let Some(members) = members.take() {
        body = body.push(members);
    } else if crate::profile::panel_shown(app)
        && let Some(panel) = crate::profile::profile_panel_view(app.ctx())
    {
        let panel = crate::theme_art::surface(
            panel,
            app.ctx(),
            yapfast_core::themes::package::art::ArtSurface::ProfilePane,
        );
        body = body.push(crate::theme_art::outlined(
            panel,
            app.ctx(),
            yapfast_core::themes::layout::BorderPlace::Panes,
            "panel",
            0.0,
            1.0,
            app.theme.outline,
        ));
    }
    let header = iced::widget::container(crate::theme_art::surface(
        crate::header::view(app.ctx()),
        app.ctx(),
        yapfast_core::themes::package::art::ArtSurface::Header,
    ))
    .height(crate::header::HEIGHT);
    column![header, line, body.height(Length::Fill)]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// The conversation column's left edge: the navigation's right.
pub fn chat_left(app: &App) -> f32 {
    let package = app.themes.package_layout(&app.theme.id);
    let navigation = if navigation_right(package) {
        0.0
    } else {
        themed_navigation(
            &app.layout,
            &app.theme,
            package,
            app.themes.chrome(&app.theme.id),
        )
    };
    navigation
        + if crate::header::members_shown(app)
            && app
                .themes
                .package_layout(&app.theme.id)
                .and_then(|layout| layout.members)
                == Some(MembersSide::Left)
        {
            crate::members::docked_width(app)
        } else {
            0.0
        }
}

/// The conversation column's right edge: the member list's left.
pub fn chat_right(app: &App) -> f32 {
    let package = app.themes.package_layout(&app.theme.id);
    let navigation = if navigation_right(package) {
        themed_navigation(
            &app.layout,
            &app.theme,
            package,
            app.themes.chrome(&app.theme.id),
        )
    } else {
        0.0
    };
    app.window.width
        - navigation
        - if crate::search::search_shown(app) {
            crate::search::WIDTH
        } else if crate::thread::aside_shown(app) {
            crate::thread::aside_width(app)
        } else if crate::header::members_shown(app)
            && app
                .themes
                .package_layout(&app.theme.id)
                .and_then(|layout| layout.members)
                .unwrap_or(MembersSide::Right)
                != MembersSide::Left
            && app
                .themes
                .package_layout(&app.theme.id)
                .and_then(|layout| layout.members)
                .unwrap_or(MembersSide::Right)
                != MembersSide::Hidden
        {
            crate::members::width(&app.layout, &app.theme)
        } else if crate::profile::panel_shown(app) {
            crate::profile::PANEL_WIDTH
        } else {
            0.0
        }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_right_rail_docks_navigation_on_the_right() {
        let mut layout = yapfast_core::themes::layout::Layout::default();
        assert!(!navigation_right(Some(&layout)));
        layout.rail = Some(RailSide::Hidden);
        assert!(!navigation_right(Some(&layout)));
        layout.rail = Some(RailSide::Right);
        assert!(navigation_right(Some(&layout)));
    }
}
