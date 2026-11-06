//! The preview's #profile-tests: made-up members for testing how profiles draw
//! without opening real people's or using an account. Their pictures are small
//! drawings in `testing/fixtures/preview-profiles`, read from the checkout this
//! was built in; without it, the members show their initials and no effects.
//! See the README there.

use std::path::PathBuf;

use crate::chat::{
    BadgeLevel, DirectoryBadge, EffectLayer, NameStyle, Nameplate, Picture, Presence, ProfileBadge,
    ProfileFriend, ProfileGame, ProfileServer, ProfileWidget, ServerTag, Wishlist, WishlistItem,
    WishlistPlace,
};

use super::*;

/// The channel they're members of.
pub(super) const ROOM: &str = "profile-tests";
/// Their pictures' addresses: never looked up, as the preview fetches nothing,
/// and `.invalid` so they couldn't be. Their paths are Discord's where
/// [`crate::discord::picture_size`] goes by them.
const ADDRESS: &str = "https://preview-profiles.invalid";
/// The custom emoji in a status, whose address only Discord's code makes.
const STATUS_EMOJI: &str = "1000000000000000001";
/// The test member who is this account.
pub(super) const OWN_ID: &str = "@own-profile:preview.local";

/// Someone made up, with what shows only in a list.
struct TestMember {
    group: &'static str,
    nameplate: Option<Nameplate>,
    profile: Profile,
}

impl TestMember {
    fn new(group: &'static str, key: &str, name: &str, avatar: u32, bio: &str) -> Self {
        Self {
            group,
            nameplate: None,
            profile: Profile {
                id: format!("@{key}:preview.local"),
                name: name.into(),
                handle: key.into(),
                avatar: Some(picture(&format!("avatars/avatar-{avatar}.png?size=128"))),
                presence: Some(Presence::Online),
                bio: bio.into(),
                created: Some("Jan 1, 2020".into()),
                joined: Some("Feb 2, 2024".into()),
                friends_since: Some("Mar 3, 2025".into()),
                roles: vec![ProfileRole {
                    name: "Test member".into(),
                    color: Some(0x5c_a8_f0),
                }],
                ..Default::default()
            },
        }
    }

    /// The avatar a list or message shows, which moves while pointed at when
    /// its address says so, as Discord's do.
    fn listed_avatar(&self) -> Option<String> {
        let avatar = self.profile.avatar.as_deref()?;
        Some(avatar.replace(".gif?", ".png?"))
    }

    fn member(&self) -> ChatMember {
        let profile = &self.profile;
        ChatMember {
            owner: false,
            id: profile.id.clone(),
            name: profile.name.clone(),
            username: Some(profile.handle.clone()),
            role: Role::Member,
            avatar: self.listed_avatar(),
            group: Some(self.group.into()),
            presence: profile.presence,
            color: None,
            activity: profile.activity.clone(),
            activities: Default::default(),
            roles: Default::default(),
            timed_out_until: None,
            tag: profile.tag.clone(),
            nameplate: self.nameplate.clone(),
            name_style: profile.name_style.clone(),
        }
    }
}

fn picture(path: &str) -> String {
    format!("{ADDRESS}/{path}")
}

/// A layer the size of Discord's whole canvas.
fn layer(file: &str, start: u32, duration: u32, looping: bool, loop_delay: u32) -> EffectLayer {
    EffectLayer {
        src: picture(&format!("assets/content/{file}")),
        start,
        duration,
        looping,
        loop_delay,
        x: 0,
        y: 0,
        width: 450,
        height: 880,
    }
}

/// A member with a profile effect.
fn with_effect(
    key: &str,
    name: &str,
    avatar: u32,
    bio: &str,
    effect: Vec<EffectLayer>,
) -> TestMember {
    let mut member = TestMember::new("Effects", key, name, avatar, bio);
    member.profile.effect = effect;
    member
}

/// A member whose name is styled, in a font the UI's `name_face` knows.
fn with_style(
    key: &str,
    name: &str,
    avatar: u32,
    (font, effect): (u32, u32),
    colors: &[u32],
) -> TestMember {
    let mut member = TestMember::new(
        "Name styles",
        key,
        name,
        avatar,
        "Its name moves on the card; on a message or member row, while pointed at.",
    );
    member.profile.name_style = Some(NameStyle {
        font,
        effect,
        colors: colors.to_vec(),
    });
    member
}

/// A themed member with a moving banner.
fn with_theme(key: &str, name: &str, avatar: u32, theme: [u32; 2]) -> TestMember {
    let mut member = TestMember::new(
        "Theme and banner",
        key,
        name,
        avatar,
        "Two theme colours, and a banner that moves.",
    );
    member.profile.theme = Some(theme);
    member.profile.accent = Some(theme[0]);
    member.profile.banner = Some(picture("banners/a_stripes/banner-stripes.gif?size=600"));
    member
}

fn everyone() -> Vec<TestMember> {
    let mut plate = TestMember::new(
        "Looks",
        "nameplate",
        "Nameplate",
        4,
        "Behind this row in the member list: a still, and a see-through WebM while pointed at.",
    );
    plate.nameplate = Some(Nameplate {
        still: picture("assets/collectibles/nameplate-still.png"),
        moving: picture("assets/collectibles/gif-vp9-alpha.webm"),
        palette: Some("berry".into()),
    });

    let mut frame = TestMember::new(
        "Looks",
        "decoration",
        "Decoration and frame",
        5,
        "A moving avatar in a moving frame. In lists and on messages, the avatar moves while pointed at.",
    );
    frame.profile.avatar = Some(picture("avatars/a_spin.gif?size=128"));
    frame.profile.decoration = Some(picture(
        "avatar-decoration-presets/decoration-ring.png?passthrough=true",
    ));

    let mut badges = TestMember::new(
        "Looks",
        "badges",
        "Badges and tag",
        6,
        "Five badges, the last moving, and a server tag with its picture. Click a badge for their window: levels reached and locked, and Quests, with none.",
    );
    badges.profile.badges = ["Game Time", "Streaming", "Gifting", "Account Age", "Quests"]
        .iter()
        .zip(["badge-1", "badge-2", "badge-3", "badge-4", "badge-moving"])
        .zip([20, 19, 17, 18, 14])
        .map(|((description, file), directory)| ProfileBadge {
            description: (*description).into(),
            icon: picture(&format!("badge-icons/{file}.png")),
            link: None,
            directory: Some(directory),
        })
        .collect();
    badges.profile.tag = Some(ServerTag {
        text: "TEST".into(),
        badge: Some(picture("clan-badges/tag.png")),
    });

    let mut showcase = TestMember::new(
        "Looks",
        "showcase",
        "Widgets and wishlist",
        1,
        "Open the full profile: this one starts on the Board tab, with a Wishlist, an activity, and mutual friends and servers.",
    );
    showcase.profile.widgets = vec![
        ProfileWidget {
            title: "Favourite games".into(),
            application: None,
            games: vec![
                ProfileGame {
                    id: "1".into(),
                    name: "Made-up Racers".into(),
                    icon: Some(picture("game-icons/game-1.png")),
                    comment: Some("Too many hours.".into()),
                },
                ProfileGame {
                    id: "2".into(),
                    name: "Pretend Farm".into(),
                    icon: Some(picture("game-icons/game-2.png")),
                    comment: None,
                },
            ],
            lists_games: true,
        },
        ProfileWidget {
            title: "Something shown only in Discord".into(),
            application: None,
            games: vec![],
            lists_games: false,
        },
    ];
    showcase.profile.activities = test_activities();
    showcase.profile.wishlist_place = Some(WishlistPlace {
        id: "1".into(),
        private: false,
    });
    showcase.profile.wishlist = Wishlist::Loaded(vec![
        WishlistItem {
            name: "A pretend bundle".into(),
            picture: Some(picture("store/wish-1.png")),
        },
        WishlistItem {
            name: "One without a picture".into(),
            picture: None,
        },
    ]);

    let bio = "A **very** long bio, with _Markdown_ and a word that won't break: \
               ThisWordGoesOnAndOnAndOnAndOnAndOnAndOnAndOnAndOnAndOnAndOnAndOnAndOnAndOnAndOnAndOn.\n\n";
    let mut long = TestMember::new(
        "Looks",
        "long-text",
        "Long text: a very long name that goes on well past what Discord lets anyone choose",
        2,
        &bio.repeat(8),
    );
    long.profile.pronouns =
        "they/them, or whichever pronouns fit in a very long line that keeps going".into();
    long.profile.activity = Some(format!(
        "<:preview:{STATUS_EMOJI}> A status with a custom emoji, long enough that it has to wrap or be cut off somewhere"
    ));
    long.profile.presence = Some(Presence::DoNotDisturb);
    plate.profile.activity = Some("Short status".into());

    plate.profile.frame = Some(crate::chat::ProfileFrame {
        inner_width: 1200,
        overflow_top: 150,
        overflow_bottom: 0,
        overflow_horizontal: 56,
        layers: vec![
            crate::chat::ProfileFrameLayer {
                src: picture("frames/oin-c.png"),
                kind: "staple".into(),
                anchor: "top".into(),
                front: false,
            },
            crate::chat::ProfileFrameLayer {
                src: picture("frames/oin-d.png"),
                kind: "staple".into(),
                anchor: "top".into(),
                front: true,
            },
        ],
    });
    showcase.profile.recent = vec![crate::chat::RecentActivity {
        title: "A pretend song".into(),
        subtitle: "A pretend band".into(),
        picture: Some(picture("game-icons/game-2.png")),
        music: true,
        at: Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64
                - 8 * 3_600_000,
        ),
    }];

    let own = TestMember::new(
        "Looks",
        "own-profile",
        "Your own profile",
        3,
        "This one is this account: no mutuals, and Edit Profile for Message.",
    );

    let mut members = vec![
        with_effect(
            "effect-burst",
            "Effect: burst then loop",
            1,
            "The burst plays once and stays gone. Stars come about 5 seconds in for 2 seconds, then every 5.",
            vec![
                layer("effect-burst.png", 0, 800, true, 0),
                layer("effect-stars.png", 5000, 2000, true, 3000),
            ],
        ),
        with_effect(
            "effect-looping",
            "Effect: looping",
            2,
            "Bubbles rise for ever.",
            vec![layer("effect-loop.png", 0, 1000, true, 0)],
        ),
        with_effect(
            "effect-once",
            "Effect: plays-once layer",
            3,
            "A bar goes down the card once, and doesn't come back.",
            vec![layer("effect-once.png", 0, 1200, false, 0)],
        ),
        with_style("name-solid", "Name 1 solid", 1, (3, 1), &[0xff_7a_b6]),
        with_style(
            "name-gradient",
            "Name 2 gradient",
            2,
            (4, 2),
            &[0x6e_e7_f9, 0xa7_8b_fa],
        ),
        with_style(
            "name-neon",
            "Name 3 neon",
            3,
            (14, 3),
            &[0x39_ff_14, 0x00_c2_ff],
        ),
        with_style(
            "name-toon",
            "Name 4 toon",
            4,
            (6, 4),
            &[0xff_d1_66, 0xef_47_6f],
        ),
        with_style(
            "name-pop",
            "Name 5 pop",
            5,
            (8, 5),
            &[0xff_5d_a2, 0xff_e6_6d],
        ),
        with_style(
            "name-prism",
            "Name 7 prism",
            6,
            (12, 7),
            &[0xff_5e_5e, 0xff_d9_3d, 0x6b_cb_77, 0x4d_96_ff],
        ),
        with_style(
            "name-gummy",
            "Name 8 gummy",
            1,
            (16, 8),
            &[0xff_6b_6b, 0x4e_cd_c4, 0xff_e6_6d],
        ),
        with_style(
            "name-gummy-long",
            "Gummy with a name long enough to run past its row",
            1,
            (13, 8),
            &[0xff_6b_6b, 0x4e_cd_c4, 0xff_e6_6d],
        ),
        plate,
        frame,
        {
            let mut light = with_theme("theme-light", "Theme: light", 2, [0xfd_e2_e4, 0xbe_e3_f8]);
            light.profile.activity = Some("Short status".into());
            light
        },
        {
            let mut dark = with_theme("theme-dark", "Theme: dark", 3, [0x4a_2a_8c, 0x12_5e_60]);
            dark.profile.activity =
                Some("A status that's long enough to need a second line".into());
            dark
        },
        badges,
        showcase,
        long,
        own,
    ];
    add_mutuals(&mut members);
    members
}

/// What Discord says of a shared friend or server, which a list of mutuals
/// shows: everyone but this account has both, but for one with no friends in
/// common, to see the card's line without its avatars, and one with neither.
fn add_mutuals(members: &mut [TestMember]) {
    let friends: Vec<ProfileFriend> = members
        .iter()
        .take(4)
        .map(|member| ProfileFriend {
            id: member.profile.id.clone(),
            name: member.profile.name.clone(),
            avatar: member.listed_avatar(),
        })
        .collect();
    let server = |id: &str, name: &str, nick: Option<&str>| ProfileServer {
        id: id.into(),
        name: name.into(),
        icon: None,
        banner: None,
        members: Some(12),
        channels: 5,
        nick: nick.map(str::to_owned),
    };
    for member in members {
        let profile = &mut member.profile;
        match profile.handle.as_str() {
            "own-profile" | "effect-once" => continue,
            "effect-looping" => {}
            _ => {
                profile.friends = friends
                    .iter()
                    .filter(|friend| friend.id != profile.id)
                    .cloned()
                    .collect();
            }
        }
        profile.servers = vec![
            server("friends", "Friends", None),
            server("coop", "Co-op crew", Some("Test nickname")),
        ];
    }
}

/// What an activity tab shows: a game with a timer, and a song with a bar.
fn test_activities() -> crate::chat::Activities {
    #[cfg(feature = "discord")]
    {
        use crate::chat::{Activity, ActivityKind};
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        vec![
            Activity {
                kind: ActivityKind::Playing,
                name: "Made-up Racers".into(),
                details: Some("In a race".into()),
                state: Some("Lap 2 of 3".into()),
                party: Some((2, 4)),
                application_id: None,
                start: Some(now.saturating_sub(600_000)),
                end: None,
                large_image: Some(picture("game-icons/game-1.png").into()),
                large_text: Some("Made-up Racers".into()),
                small_image: None,
                small_text: None,
                url: None,
                link: None,
                emoji: None,
            },
            Activity {
                kind: ActivityKind::Listening,
                name: "A pretend player".into(),
                details: Some("A pretend song".into()),
                state: Some("A pretend band".into()),
                party: None,
                application_id: None,
                start: Some(now.saturating_sub(60_000)),
                end: Some(now + 120_000),
                large_image: Some(picture("game-icons/game-2.png").into()),
                large_text: Some("A pretend album".into()),
                small_image: None,
                small_text: None,
                url: None,
                link: None,
                emoji: None,
            },
        ]
        .into()
    }
    #[cfg(not(feature = "discord"))]
    Default::default()
}

/// A badge in a badge window, its picture one of the test ones.
#[allow(clippy::too_many_arguments)]
fn test_badge(
    id: u64,
    name: &str,
    file: &str,
    rarity: u8,
    owned: bool,
    description: &str,
    info: &str,
    unlocked: Option<&str>,
    levels: &[(&str, &str, bool)],
) -> DirectoryBadge {
    let icon = picture(&format!("badge-icons/{file}.png"));
    let reached = levels.iter().rposition(|level| level.2).unwrap_or(0);
    DirectoryBadge {
        id,
        name: name.into(),
        owned,
        earnable: true,
        description: description.into(),
        info: info.into(),
        rarity,
        icon: icon.clone(),
        art: None,
        unlocked: unlocked.map(str::to_owned),
        level: levels.get(reached).map(|level| level.0.to_owned()),
        first_level: reached == 0,
        level_rarity: rarity,
        level_art: icon.clone(),
        levels: levels
            .iter()
            .map(|(name, milestone, owned)| BadgeLevel {
                name: (*name).into(),
                icon: icon.clone(),
                owned: *owned,
                milestone: (*milestone).into(),
            })
            .collect(),
        ..Default::default()
    }
}

/// What the badge window shows of a test member; they own these, as someone
/// else's list from Discord has only what they own.
pub(in crate::application) fn test_badges(user: &str) -> Option<Vec<DirectoryBadge>> {
    (user == "@badges:preview.local").then(|| {
        vec![
            test_badge(
                20,
                "Game Time",
                "badge-1",
                3,
                true,
                "Play 1 hour of games with Discord open to unlock this badge.",
                "1,008 hours of games played",
                Some("Oct 27, 2026"),
                &[
                    ("Casual", "1 hour", true),
                    ("Recreational", "5 hours", true),
                    ("Dedicated", "20 hours", true),
                    ("Committed", "75 hours", true),
                    ("Serious", "150 hours", true),
                    ("Devoted", "300 hours", true),
                    ("Seasoned", "500 hours", true),
                    ("Ironclad", "1,000 hours", true),
                    ("Unshakeable", "2,000 hours", false),
                    ("Eternal", "5,000+ hours", false),
                ],
            ),
            test_badge(
                19,
                "Streaming",
                "badge-2",
                2,
                true,
                "Stream more hours to other users to level up this badge.",
                "197 hours streamed to others",
                Some("Jul 29, 2026"),
                &[
                    ("Newcomer", "1 hour", true),
                    ("Fledgling", "5 hours", true),
                    ("Breakout", "20 hours", true),
                    ("Headliner", "300 hours", false),
                ],
            ),
            test_badge(
                17,
                "Gifting",
                "badge-3",
                2,
                true,
                "Send 1 gift to unlock this badge.",
                "2 gifts given",
                Some("May 28, 2026"),
                &[
                    ("Patron", "1 gift", true),
                    ("Champion", "2 gifts", true),
                    ("Luminary", "3 gifts", false),
                ],
            ),
            test_badge(
                18,
                "Account Age",
                "badge-4",
                1,
                true,
                "Have an active Discord account for 1 year to unlock this badge.",
                "2 years, 11 months",
                Some("Nov 30, 2025"),
                &[
                    ("Seed", "1 year", true),
                    ("Sprout", "2 years", true),
                    ("Sapling", "3 years", false),
                ],
            ),
            // No levels and no date, as Discord gives Quests.
            test_badge(
                14,
                "Quests",
                "badge-moving",
                1,
                true,
                "Complete 1 Quest to unlock this badge.",
                "30 Quests completed",
                None,
                &[],
            ),
        ]
    })
}

/// The preview account's own badges: Streaming, so its "View yours" shows.
pub(in crate::application) fn test_own_badges() -> Vec<DirectoryBadge> {
    vec![test_badge(
        19,
        "Streaming",
        "badge-2",
        2,
        true,
        "Stream more hours to other users to level up this badge.",
        "3 hours streamed to others",
        Some("Sep 1, 2026"),
        &[
            ("Newcomer", "1 hour", true),
            ("Fledgling", "5 hours", false),
        ],
    )]
}

/// The channel's member list, in the order they're made and under their groups.
pub(super) fn members() -> Vec<ChatMember> {
    everyone().iter().map(TestMember::member).collect()
}

/// The whole profile of a test member.
pub(in crate::application) fn test_profile(id: &str) -> Option<Profile> {
    everyone()
        .into_iter()
        .find(|member| member.profile.id == id)
        .map(|member| member.profile)
}

/// A few messages from them, to see their names and avatars on message rows.
pub(super) fn messages() -> Vec<Arc<ChatMessage>> {
    let everyone = everyone();
    [
        ("effect-burst", "Open my card: burst once, then stars."),
        ("name-solid", "Solid, and it holds still."),
        ("name-gradient", "Gradient, and it holds still."),
        ("name-neon", "Point at this message to see my name glow."),
        ("name-toon", "Point at this message to see my name move."),
        ("name-pop", "Point at this message to see my name move."),
        (
            "name-prism",
            "Point at this message to see my colours go round.",
        ),
        (
            "name-gummy",
            "Point at this message to see my letters move.",
        ),
        ("decoration", "Point at this message to see my avatar move."),
        ("badges", "My server tag is beside my name."),
        ("long-text", "My long name has to fit on this row somehow."),
    ]
    .into_iter()
    .enumerate()
    .filter_map(|(index, (key, body))| {
        let id = format!("@{key}:preview.local");
        let member = everyone.iter().find(|member| member.profile.id == id)?;
        let row = format!("{ROOM}-{index}");
        Some(Arc::new(ChatMessage {
            id: row.clone(),
            item: row,
            author: member.profile.name.clone(),
            sender: id,
            avatar: member.listed_avatar(),
            tag: member.profile.tag.clone(),
            name_style: member.profile.name_style.clone(),
            body: body.into(),
            time: format!("19:{:02}", 20 + index),
            can_edit: false,
            can_delete: false,
            ..sample_actions()
        }))
    })
    .collect()
}

/// The file behind a test picture's address; none for any other address.
fn picture_file(url: &str) -> Option<PathBuf> {
    let fixtures = PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../testing/fixtures"
    ));
    let ours = fixtures.join("preview-profiles");
    if crate::discord::emoji_picture(STATUS_EMOJI).as_deref() == Some(url) {
        return Some(ours.join("emoji-status.png"));
    }
    let path = url.strip_prefix(ADDRESS)?.split('?').next()?;
    let name = path.rsplit('/').next()?;
    Some(match name {
        "gif-vp9-alpha.webm" => fixtures.join(name),
        _ => ours.join(name),
    })
}

/// A test picture as it would have downloaded and decoded.
fn decode_test_picture(url: &str) -> Option<Picture> {
    let data = std::fs::read(picture_file(url)?).ok()?;
    let (width, height) = crate::discord::picture_size(url);
    crate::chat::decode_picture(&data, width, height)
}

impl Application {
    /// Has the pictures the preview asked for arrive, those it has: read from
    /// disk where Discord's would download, and from there on the same.
    pub(in crate::application) fn keep_test_pictures(&mut self, urls: Vec<String>) {
        for url in urls {
            if let Some(picture) = decode_test_picture(&url) {
                self.keep_picture(url, Arc::new(picture));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn previewing() -> Application {
        let mut app = Application::default();
        app.dispatch(Action::Preview);
        assert!(app.state.preview);
        app
    }

    /// Opens the channel, as selecting it does.
    fn in_room(app: &mut Application) {
        assert!(no_backend(&app.dispatch(Action::SelectRoom(ROOM.into()))));
        assert_eq!(app.state.selected.as_deref(), Some(ROOM));
    }

    fn no_backend(effects: &[Effect]) -> bool {
        !effects
            .iter()
            .any(|effect| matches!(effect, Effect::Backend(_)))
    }

    fn profile(name: &str) -> Profile {
        everyone()
            .into_iter()
            .map(|member| member.profile)
            .find(|profile| profile.name.starts_with(name))
            .unwrap_or_else(|| panic!("{name}"))
    }

    /// Every address a test member shows, or would while pointed at.
    fn addresses() -> Vec<String> {
        let mut addresses = Vec::new();
        for member in everyone() {
            let profile = member.profile;
            addresses.extend(profile.avatar.clone());
            addresses.extend(
                member
                    .nameplate
                    .iter()
                    .flat_map(|plate| [plate.still.clone(), plate.moving.clone()]),
            );
            addresses.extend(profile.decoration);
            addresses.extend(
                profile
                    .frame
                    .into_iter()
                    .flat_map(|frame| frame.layers)
                    .map(|layer| layer.src),
            );
            addresses.extend(profile.banner);
            addresses.extend(profile.tag.and_then(|tag| tag.badge));
            addresses.extend(profile.effect.into_iter().map(|layer| layer.src));
            addresses.extend(profile.badges.into_iter().map(|badge| badge.icon));
            addresses.extend(
                profile
                    .friends
                    .into_iter()
                    .filter_map(|friend| friend.avatar),
            );
            for widget in profile.widgets {
                addresses.extend(widget.games.into_iter().filter_map(|game| game.icon));
            }
            if let Wishlist::Loaded(items) = profile.wishlist {
                addresses.extend(items.into_iter().filter_map(|item| item.picture));
            }
            if let Some(avatar) = profile.avatar.filter(|avatar| avatar.contains(".gif?")) {
                addresses.push(avatar.replace(".gif?", ".png?"));
            }
            if let Some((Some(emoji), _)) = profile
                .activity
                .as_deref()
                .map(crate::discord::status_parts)
            {
                addresses.push(emoji);
            }
        }
        addresses.sort();
        addresses.dedup();
        addresses
    }

    #[test]
    fn the_preview_lists_them_in_its_own_channel_with_messages() {
        let mut app = previewing();
        let friends = app
            .state
            .servers
            .iter()
            .find(|server| server.id == "friends");
        assert!(
            friends
                .unwrap()
                .channels
                .iter()
                .any(|channel| channel == ROOM)
        );
        in_room(&mut app);
        let listed = app.state.members.clone().unwrap().unwrap();
        for name in [
            "Effect: burst then loop",
            "Effect: looping",
            "Effect: plays-once layer",
            "Nameplate",
            "Decoration and frame",
            "Theme: light",
            "Theme: dark",
            "Badges and tag",
            "Widgets and wishlist",
            "Long text",
        ] {
            assert!(
                listed.iter().any(|member| member.name.starts_with(name)),
                "{name}"
            );
        }
        for message in app.state.messages.iter() {
            assert!(
                listed.iter().any(|member| member.id == message.sender),
                "{}",
                message.author
            );
        }
        assert!(
            app.state
                .messages
                .iter()
                .any(|message| message.tag.is_some())
        );
        assert!(app.state.messages.iter().any(|message| {
            message
                .name_style
                .as_ref()
                .is_some_and(|style| style.effect == 8)
        }));
        // Their groups show in the order they're made.
        assert_eq!(listed[0].group.as_deref(), Some("Effects"));
    }

    #[test]
    fn name_styles_cover_each_effect_in_a_different_font() {
        let everyone = everyone();
        let styles: Vec<_> = everyone
            .iter()
            .filter_map(|member| member.profile.name_style.as_ref())
            .collect();
        // Gummy twice: the second's name is long enough to run past its row.
        assert_eq!(
            styles.iter().map(|style| style.effect).collect::<Vec<_>>(),
            [1, 2, 3, 4, 5, 7, 8, 8]
        );
        let fonts: HashSet<u32> = styles.iter().map(|style| style.font).collect();
        assert_eq!(fonts.len(), styles.len());
        assert!(styles.iter().all(|style| !style.colors.is_empty()));
    }

    #[test]
    fn profiles_have_what_each_tests() {
        let plate = everyone()
            .into_iter()
            .find_map(|member| member.nameplate)
            .unwrap();
        assert!(plate.moving.ends_with(".webm"));
        assert!(plate.palette.is_some());

        let frame = profile("Decoration");
        assert!(frame.avatar.unwrap().contains("/a_spin.gif?"));
        assert!(frame.decoration.is_some());

        let (light, dark) = (profile("Theme: light"), profile("Theme: dark"));
        assert!(light.theme_is_light());
        assert!(dark.theme.is_some() && !dark.theme_is_light());
        assert!(light.banner.is_some() && dark.banner.is_some());

        let badges = profile("Badges");
        assert!(badges.badges.len() > 2);
        assert!(badges.tag.unwrap().badge.is_some());

        let showcase = profile("Widgets");
        assert!(showcase.widgets[0].lists_games && !showcase.widgets[0].games.is_empty());
        assert!(matches!(showcase.wishlist, Wishlist::Loaded(items) if !items.is_empty()));

        let long = profile("Long text");
        assert!(long.name.len() > 32 && long.pronouns.len() > 40 && long.bio.len() > 1000);
        let (emoji, words) = crate::discord::status_parts(long.activity.as_deref().unwrap());
        assert_eq!(emoji, crate::discord::emoji_picture(STATUS_EMOJI));
        assert!(words.starts_with("A status"));
    }

    #[test]
    fn every_picture_decodes_as_a_real_one_does() {
        for url in addresses() {
            let picture = decode_test_picture(&url).unwrap_or_else(|| panic!("{url}"));
            assert!(picture.width > 0 && picture.height > 0, "{url}");
            if let Some(animation) = &picture.animation {
                assert!(animation.delays.len() > 1, "{url}");
                assert_eq!(picture.frames().count(), animation.delays.len(), "{url}");
            }
        }
    }

    #[test]
    fn the_decoder_reports_how_animations_say_they_play() {
        let animated = |url: &str| {
            let picture = decode_test_picture(url).unwrap_or_else(|| panic!("{url}"));
            let animation = picture
                .animation
                .as_ref()
                .unwrap_or_else(|| panic!("{url}"));
            (
                animation.plays,
                animation.delays.iter().sum::<u32>(),
                animation.delays.len(),
            )
        };
        for (name, plays) in [
            ("Effect: burst", [1, 0].as_slice()),
            ("Effect: looping", &[0]),
            ("Effect: plays-once", &[1]),
        ] {
            let layers = profile(name).effect;
            assert_eq!(layers.len(), plays.len(), "{name}");
            for (layer, plays) in layers.iter().zip(plays) {
                let (found, total, frames) = animated(&layer.src);
                assert_eq!(found, *plays, "{}", layer.src);
                // A layer's frames are spread over its duration.
                assert_eq!(total, layer.duration, "{}", layer.src);
                assert!(frames > 1, "{}", layer.src);
            }
        }
        // The case fixed: a loop with no wait over a file that plays once, and
        // a loop with a wait over one that comes in late.
        let burst = profile("Effect: burst").effect;
        assert!(burst[0].looping && burst[0].loop_delay == 0);
        assert!(burst[1].looping && burst[1].start == 5000 && burst[1].loop_delay == 3000);
        assert!(!profile("Effect: plays-once").effect[0].looping);
        // It ends clear, so the held frame shows nothing.
        let picture = decode_test_picture(&burst[0].src).unwrap();
        let frames: Vec<_> = picture.frames().collect();
        assert!(frames[0].chunks(4).any(|pixel| pixel[3] > 0));
        assert!(frames.last().unwrap().chunks(4).all(|pixel| pixel[3] == 0));

        let moving = |url: String| decode_test_picture(&url).unwrap().animation.is_some();
        assert!(moving(profile("Decoration").avatar.unwrap()));
        assert!(moving(profile("Decoration").decoration.unwrap()));
        assert!(moving(profile("Theme: dark").banner.unwrap()));
    }

    #[test]
    #[cfg(feature = "discord")]
    fn the_moving_nameplate_is_a_see_through_video() {
        let plate = everyone()
            .into_iter()
            .find_map(|member| member.nameplate)
            .unwrap();
        let picture = decode_test_picture(&plate.moving).unwrap();
        assert!(picture.animation.is_some());
        assert!(
            picture
                .frames()
                .any(|frame| frame.chunks(4).any(|pixel| pixel[3] < 255))
        );
        assert!(
            decode_test_picture(&plate.still)
                .unwrap()
                .animation
                .is_none()
        );
    }

    #[test]
    fn opening_one_shows_all_of_it_and_asks_no_backend() {
        let mut app = previewing();
        in_room(&mut app);
        let id = profile("Effect: burst").id;
        let effects = app.dispatch(Action::ShowProfile {
            user: id.clone(),
            large: false,
        });
        assert!(no_backend(&effects));
        let view = app.state.profile.as_ref().unwrap();
        assert!(!view.loading);
        assert_eq!(view.profile.effect.len(), 2);
        // The UI asks for what it draws; it arrives by the path Discord's do,
        // with the effect's layers before it plays.
        let layers: Vec<String> = view
            .profile
            .effect
            .iter()
            .map(|layer| layer.src.clone())
            .collect();
        let changes = app.state.picture_changes;
        assert!(no_backend(
            &app.dispatch(Action::LoadPictures(layers.clone()))
        ));
        assert!(app.state.picture_changes > changes);
        assert!(
            layers
                .iter()
                .all(|url| app.state.pictures.contains_key(url))
        );
        assert_eq!(
            app.state.pictures[&layers[0]]
                .animation
                .as_ref()
                .unwrap()
                .plays,
            1
        );
        // What's on screen with no asking, its avatar, came too.
        let avatar = view_avatar(&app);
        assert!(app.state.pictures.contains_key(&avatar));
        assert!(no_backend(&app.dispatch(Action::EnlargeProfile)));
        assert!(app.state.profile.as_ref().unwrap().large);
    }

    #[test]
    fn the_full_profiles_tabs_are_filled_and_ask_nothing() {
        let mut app = previewing();
        in_room(&mut app);
        let id = profile("Widgets").id;
        app.dispatch(Action::ShowProfile {
            user: id,
            large: true,
        });
        assert!(no_backend(
            &app.dispatch(Action::OpenProfileTab(ProfileTab::Board))
        ));
        assert!(no_backend(
            &app.dispatch(Action::OpenProfileTab(ProfileTab::Wishlist))
        ));
        let view = app.state.profile.as_ref().unwrap();
        assert!(matches!(&view.profile.wishlist, Wishlist::Loaded(items) if items.len() == 2));
        assert!(view.profile.widgets[0].lists_games);
        assert!(!view.profile.friends.is_empty() && !view.profile.servers.is_empty());
        #[cfg(feature = "discord")]
        assert_eq!(view.profile.activities.len(), 2);
    }

    #[test]
    fn a_badge_opens_its_owners_window_with_no_backend() {
        let mut app = previewing();
        in_room(&mut app);
        let id = profile("Badges").id;
        app.dispatch(Action::ShowProfile {
            user: id.clone(),
            large: false,
        });
        assert!(no_backend(&app.dispatch(Action::ShowPersonBadges {
            user: id,
            badge: Some(14),
        })));
        let window = app.state.person_badges.as_ref().unwrap();
        assert_eq!(window.badge, Some(14));
        let badges = window.badges.as_ref().unwrap();
        // Both kinds: levels reached and locked, and none at all.
        assert!(
            badges[0].levels.iter().any(|level| level.owned)
                && badges[0].levels.iter().any(|level| !level.owned)
        );
        let quests = badges.iter().find(|badge| badge.id == 14).unwrap();
        assert!(quests.levels.is_empty() && quests.unlocked.is_none() && quests.level.is_none());
        // Only Streaming is the account's too.
        assert_eq!(app.state.badges.as_ref().unwrap()[0].id, 19);
    }

    #[test]
    fn the_own_member_is_this_account_with_no_mutuals() {
        let mut app = previewing();
        assert_eq!(app.state.account_id.as_deref(), Some(OWN_ID));
        let own = profile("Your own");
        assert_eq!(own.id, OWN_ID);
        assert!(own.friends.is_empty() && own.servers.is_empty());
        in_room(&mut app);
        // The badge window for this account is its own.
        app.dispatch(Action::ShowPersonBadges {
            user: OWN_ID.into(),
            badge: Some(19),
        });
        assert!(app.state.badges_open && app.state.person_badges.is_none());
        assert_eq!(app.state.badges_initial, Some(19));
    }

    fn view_avatar(app: &Application) -> String {
        app.state
            .profile
            .as_ref()
            .unwrap()
            .profile
            .avatar
            .clone()
            .unwrap()
    }

    #[test]
    fn asking_for_other_addresses_in_the_preview_keeps_nothing() {
        let mut app = previewing();
        let elsewhere = "https://cdn.discordapp.com/avatars/1/2.png?size=128".to_owned();
        assert!(no_backend(
            &app.dispatch(Action::LoadPictures(vec![elsewhere.clone()]))
        ));
        assert!(!app.state.pictures.contains_key(&elsewhere));
        assert!(picture_file(&elsewhere).is_none());
    }

    #[test]
    fn without_the_preview_nothing_here_happens() {
        let mut app = Application::default();
        let url = profile("Effect: burst").effect[0].src.clone();
        let effects = app.dispatch(Action::LoadPictures(vec![url.clone()]));
        assert!(matches!(
            effects.as_slice(),
            [Effect::Backend(Command::Discord(DiscordCommand::Pictures(urls)))] if urls == &[url.clone()]
        ));
        assert!(!app.state.pictures.contains_key(&url));
        let id = profile("Effect: burst").id;
        app.open_profile(
            "room".into(),
            Profile {
                id,
                ..Default::default()
            },
            false,
        );
        assert!(app.state.profile.unwrap().profile.effect.is_empty());
    }
}
