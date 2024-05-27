use std::borrow::Cow;
use std::iter;

use crate::config::{Config, CONFIG_VERSION};
#[allow(unused_imports)]
use crate::fl;
use crate::localize::LANGUAGE_LOADER;
use crate::widget_copy;
use cosmic::app::Core;
use cosmic::cosmic_config;
use cosmic::iced;
use cosmic::iced::wayland::popup::{destroy_popup, get_popup};
use cosmic::iced::window::Id;
#[allow(unused_imports)]
use cosmic::iced::{alignment, Alignment, Length};
use cosmic::iced::{Command, Limits};
use cosmic::iced_futures::Subscription;
use cosmic::iced_runtime::core::window;
use cosmic::iced_style::application;

use cosmic::iced_widget::scrollable;
use cosmic::widget::{self};
use cosmic::{Apply, Element, Theme};

use cosmic_time::Timeline;

use regex::RegexBuilder;

pub const ID: &str = "dev.dominiccgeh.CosmicAppletEmojiSelector";
const ICON: &str = ID;
pub struct Window {
    core: Core,
    popup: Option<Id>,
    config: Config,
    #[allow(dead_code)]
    config_handler: Option<cosmic_config::Config>,
    timeline: Timeline,
    selected_group: Option<emojis::Group>,
    search: String,
    scrollable_id: widget::Id,
    font_family: cosmic::iced::font::Font,
    emoji_hovered: Option<&'static emojis::Emoji>,
    text_input_id: widget::Id,
}
#[derive(Clone, Debug)]
pub enum Message {
    Config(Config),
    TogglePopup,
    PopupClosed(Id),
    Group(Option<emojis::Group>),
    EmojiCopy(&'static emojis::Emoji),
    Search(String),
    Frame(std::time::Instant),
    EmojiHovered(&'static emojis::Emoji),
    Exit,
    FocusTextInput,
    Enter,
    ArrowRight,
    ArrowLeft,
    ScrollToPercent(u8),
}

#[derive(Clone, Debug)]
pub struct Flags {
    pub config_handler: Option<cosmic_config::Config>,
    pub config: Config,
}

impl cosmic::Application for Window {
    type Executor = cosmic::SingleThreadExecutor;
    type Flags = Flags;
    type Message = Message;
    const APP_ID: &'static str = ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(
        core: Core,
        flags: Self::Flags,
    ) -> (Self, Command<cosmic::app::Message<Self::Message>>) {
        let selected_group = None;
        let config = flags.config;
        let font_family =
            iced::Font::with_name(Box::leak(config.font_family.clone().into_boxed_str()));
        let window = Window {
            font_family,
            scrollable_id: widget::Id::unique(),
            selected_group,
            core,
            config,
            config_handler: flags.config_handler,
            popup: None,
            search: String::new(),
            timeline: Timeline::new(),
            emoji_hovered: None,
            text_input_id: widget::Id::unique(),
        };

        (window, Command::none())
    }

    fn on_close_requested(&self, id: window::Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn update(&mut self, message: Self::Message) -> Command<cosmic::app::Message<Self::Message>> {
        // Helper for updating config values efficiently
        #[allow(unused_macros)]
        macro_rules! config_set {
            ($name: ident, $value: expr) => {
                match &self.config_handler {
                    Some(config_handler) => {
                        match paste::paste! { self.config.[<set_ $name>](config_handler, $value) } {
                            Ok(_) => {}
                            Err(err) => {
                                eprintln!("failed to save config {:?}: {}", stringify!($name), err);
                            }
                        }
                    }
                    None => {
                        self.config.$name = $value;
                        eprintln!(
                            "failed to save config {:?}: no config handler",
                            stringify!($name),
                        );
                    }
                }
            };
        }

        match message {
            Message::Config(config) => {
                if config != self.config {
                    if config.font_family != self.config.font_family {
                        self.font_family = iced::Font::with_name(Box::leak(
                            config.font_family.clone().into_boxed_str(),
                        ));
                    }
                    self.config = config
                }
            }
            Message::Frame(now) => self.timeline.now(now),

            Message::TogglePopup => {
                return if let Some(p) = self.popup.take() {
                    destroy_popup(p)
                } else {
                    let new_id = Id::unique();
                    self.popup.replace(new_id);
                    let mut popup_settings =
                        self.core
                            .applet
                            .get_popup_settings(Id::MAIN, new_id, None, None, None);
                    popup_settings.positioner.size_limits = Limits::NONE
                        .max_width(475.0)
                        .min_width(300.0)
                        .min_height(200.0)
                        .max_height(1080.0);
                    get_popup(popup_settings)
                }
            }
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
            }
            Message::EmojiCopy(emoji) => {
                let mut last_used = self.config.last_used.clone();
                if let Some(idx) = last_used.iter().position(|e| e == emoji.as_str()) {
                    last_used.swap(0, idx);
                } else {
                    last_used.insert(0, emoji.to_string());
                }
                last_used.truncate(self.config.last_used_limit);
                config_set!(last_used, last_used);
                use wl_clipboard_rs::copy::{MimeType, Options, Source};
                let opts = Options::new();
                if let Err(_) = opts.copy(
                    Source::Bytes(emoji.to_string().into_bytes().into()),
                    MimeType::Autodetect,
                ) {
                    if self.config.use_wl_copy {
                        _ = std::process::Command::new("wl-copy")
                            .arg(emoji.as_str())
                            .spawn();
                    }
                }
                if self.config.close_on_copy {
                    if let Some(p) = self.popup.take() {
                        return destroy_popup(p);
                    }
                }
            }
            Message::Search(search) => {
                self.search = search;
                self.emoji_hovered = None;
            }
            Message::Group(group) => return self.update_group(group),

            Message::EmojiHovered(emoji) => self.emoji_hovered = Some(emoji),
            Message::Exit => {
                if let Some(p) = self.popup.take() {
                    return destroy_popup(p);
                }
            }
            Message::Enter => {}
            Message::FocusTextInput => {
                return widget::text_input::focus(self.text_input_id.clone());
            }
            Message::ArrowRight => {
                let mut key = key_from_group(self.selected_group);
                key = if key >= b'9' { b'0' } else { key + 1 };
                return self.update_group(group_from_key(key));
            }
            Message::ArrowLeft => {
                let mut key = key_from_group(self.selected_group);
                key = if key <= b'0' { b'9' } else { key - 1 };
                return self.update_group(group_from_key(key));
            }
            Message::ScrollToPercent(percent) => {
                let offset = if percent == 0 {
                    scrollable::RelativeOffset::START
                } else {
                    scrollable::RelativeOffset::END
                };
                return scrollable::snap_to(self.scrollable_id.clone(), offset);
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        self.core
            .applet
            .icon_button(ICON)
            .on_press(Message::TogglePopup)
            .into()
    }

    // todo extract more code into functions
    fn view_window(&self, _id: Id) -> Element<Self::Message> {
        // use regex to apply simple unicode case folding

        let regex_pattern = regex::escape(&self.search);
        let search_regex = RegexBuilder::new(&regex_pattern)
            .case_insensitive(true)
            .build()
            .ok();

        let search_filter = |emoji: &emojis::Emoji, search_regex: Option<&regex::Regex>| {
            if self.search.is_empty() {
                return true;
            }
            match search_regex {
                Some(re) => re.is_match(emoji.name()),
                None => emoji.name().contains(&self.search),
            }
        };

        #[allow(unused_variables)]
        let cosmic::cosmic_theme::Spacing {
            space_none, // 0
            space_xxxs, // 4
            space_xxs,  // 8
            space_xs,   // 12
            space_s,    // 16
            space_m,    // 24
            space_l,    // 32
            space_xl,   // 48
            space_xxl,  // 64
            space_xxxl, // 128
        } = self.core.system_theme().cosmic().spacing;
        let mut content = widget::column::with_capacity(6)
            .padding([space_xxs, space_xxxs])
            .spacing(space_m);

        let mut groups = widget::row::with_capacity(9).width(Length::Fill);

        for group in emojis::Group::iter() {
            let is_selected = self.selected_group.is_some_and(|sel| sel == group);
            let group_btn = widget::icon::from_name(group_icon(group))
                .symbolic(true)
                .size(space_m)
                .apply(widget::button)
                // honestly there isnt a good style
                // needs containers
                .style(cosmic::theme::Button::Icon)
                .selected(is_selected)
                .padding(space_xs)
                .on_press(Message::Group((!is_selected).then_some(group)))
                .apply(widget::container)
                .width(Length::Fill)
                .center_x();

            groups = groups.push(group_btn);
        }
        content = content.push(groups);

        let search = widget::search_input(fl!("search-for-emojis"), &self.search)
            .on_input(Message::Search)
            .on_clear(Message::Search(String::new()))
            .id(self.text_input_id.clone())
            .on_submit(Message::Enter)
            .width(Length::Fill);
        content = content.push(search);

        if self.config.show_preview {
            let preview = self.preview(
                search_filter,
                &search_regex,
                &self.core.system_theme().cosmic().spacing,
            );
            let preview_container = widget::container(preview).center_y().height(65);
            content = content.push(preview_container);
        }
        const GRID_SIZE: usize = 10;

        let mut grid = widget::column();

        let emoji_row = |emojis: [Option<&'static emojis::Emoji>; 10]| {
            let mut row = widget::row::with_capacity(GRID_SIZE);
            for emoji in emojis.iter().filter_map(|e| *e) {
                // todo figure out button and text style
                let emoji_txt = widget::text(emoji.as_str())
                    .size(25)
                    .width(35)
                    .height(35)
                    .font(self.font_family)
                    .shaping(cosmic::iced_core::text::Shaping::Advanced)
                    .horizontal_alignment(alignment::Horizontal::Center)
                    .vertical_alignment(alignment::Vertical::Center);
                let mut emoji_btn = widget::button(emoji_txt)
                    .on_press(Message::EmojiCopy(emoji))
                    .style(cosmic::theme::Button::Icon)
                    .apply(widget_copy::MouseArea::new)
                    .on_enter(Message::EmojiHovered(emoji))
                    .apply(Element::from);

                if self.config.show_tooltip {
                    let tooltip = format_emoji(&emoji, self.config.show_unicode);
                    let emoji_tooltip =
                        widget::tooltip(emoji_btn, tooltip, widget::tooltip::Position::Top);
                    emoji_btn = emoji_tooltip.into()
                }
                row = row.push(emoji_btn)
            }
            row
        };

        let search_iter = self.config_emoji_iter(search_filter, &search_regex);
        let mut has_favorite = false;
        for emojis in chunks(search_iter) {
            has_favorite = true;
            grid = grid.push(emoji_row(emojis));
        }
        if has_favorite {
            grid = grid.push(widget::vertical_space(space_xs));
            grid = grid.push(widget::divider::horizontal::default());
            grid = grid.push(widget::vertical_space(space_xs));
        }

        let emoji_iter = self.emoji_iter(search_filter, &search_regex);
        for emojis in chunks(emoji_iter) {
            grid = grid.push(emoji_row(emojis));
        }
        let grid = grid
            .apply(widget::container)
            .apply(widget_copy::Scrollable::new)
            .id(self.scrollable_id.clone())
            .height(Length::Fill)
            .width(Length::Fill)
            .apply(widget::container)
            .width(Length::Fill)
            .height(500);
        content = content.push(grid);
        self.core.applet.popup_container(content).into()
    }
    fn subscription(&self) -> Subscription<Self::Message> {
        struct ConfigSubscription;
        let config = cosmic_config::config_subscription(
            std::any::TypeId::of::<ConfigSubscription>(),
            Self::APP_ID.into(),
            CONFIG_VERSION,
        )
        .map(|update| {
            if !update.errors.is_empty() {
                eprintln!(
                    "errors loading config {:?}: {:?}",
                    update.keys, update.errors
                );
            }
            Message::Config(update.config)
        });

        let timeline = self
            .timeline
            .as_subscription()
            .map(|(_, now)| Message::Frame(now));

        Subscription::batch(vec![config, timeline, navigation_subscription()])
    }

    fn style(&self) -> Option<<Theme as application::StyleSheet>::Style> {
        Some(cosmic::applet::style())
    }
}

impl Window {
    fn config_emoji_iter<'a>(
        &'a self,
        search_filter: impl Fn(&'static emojis::Emoji, Option<&regex::Regex>) -> bool + 'a,
        search_regex: &'a Option<regex::Regex>,
    ) -> impl Iterator<Item = &'static emojis::Emoji> + 'a {
        let search_iter = self
            .config
            .last_used
            .iter()
            .filter_map(|e| emojis::get(e))
            .filter(|e| self.selected_group.is_none() || Some(e.group()) == self.selected_group)
            .filter(move |emoji| search_filter(emoji, search_regex.as_ref()));
        search_iter
    }
    fn emoji_iter<'a>(
        &'a self,
        search_filter: impl Fn(&'static emojis::Emoji, Option<&regex::Regex>) -> bool + 'a,
        search_regex: &'a Option<regex::Regex>,
    ) -> impl Iterator<Item = &'static emojis::Emoji> + 'a {
        let emoji_iter: Box<dyn Iterator<Item = &'static emojis::Emoji>> = match self.selected_group
        {
            Some(group) => Box::from(group.emojis()),
            None => Box::from(emojis::iter()),
        };
        emoji_iter.filter(move |emoji| search_filter(emoji, search_regex.as_ref()))
    }
    fn preview(
        &self,
        search_filter: impl Fn(&'static emojis::Emoji, Option<&regex::Regex>) -> bool,
        search_regex: &Option<regex::Regex>,
        spacing: &cosmic::cosmic_theme::Spacing,
    ) -> Element<Message> {
        let favorites_first = || self.config_emoji_iter(&search_filter, &search_regex).next();
        let emojis_first = || {
            (!self.search.is_empty()).then_some(())?;
            self.emoji_iter(&search_filter, &search_regex).next()
        };
        let preview = if let Some(emoji_hovered) = self
            .emoji_hovered
            .or_else(favorites_first)
            .or_else(emojis_first)
        {
            let mut preview = widget::row::with_capacity(2)
                .spacing(spacing.space_xxs)
                .align_items(Alignment::Center);
            // todo size and width is arbitary; user config?

            let display_characters = &['\u{fe0f}', '\u{fe0e}'];
            let mut emoji_hovered_no_display_characters = Cow::from(
                emoji_hovered
                    .as_str()
                    .strip_suffix(display_characters)
                    .unwrap_or(emoji_hovered.as_str()),
            );
            if emoji_hovered_no_display_characters.contains(display_characters) {
                emoji_hovered_no_display_characters = emoji_hovered_no_display_characters
                    .replace(display_characters, "")
                    .into()
            }

            let preview_name = std::iter::once(String::from("tts"))
                .chain(
                    emoji_hovered_no_display_characters
                        .chars()
                        .map(|c| format!("{:x}", c as u32)),
                )
                .collect::<Vec<_>>()
                .join("-");
            let preview_name = LANGUAGE_LOADER.get(&preview_name);
            println!("{preview_name}, {}", format_emoji(emoji_hovered, true));

            let preview_emoji = widget::text(emoji_hovered.as_str())
                .font(self.font_family)
                .shaping(cosmic::iced_core::text::Shaping::Advanced)
                .size(35)
                .height(50)
                .width(50)
                .horizontal_alignment(alignment::Horizontal::Center)
                .vertical_alignment(alignment::Vertical::Center);
            preview = preview.push(preview_emoji);
            let show_unicode = self.config.show_unicode;
            let mut right_preview = widget::column::with_capacity(2 + show_unicode as usize);

            // this all for south georgia and south sandwich islands
            // replace if iced gets proper text wrapping
            let mut emoji_name = preview_name.as_str();
            let emoji_name_len = emoji_name.len();
            let cut_off_idx = emoji_name
                .char_indices()
                .nth(40)
                .map_or(emoji_name_len, |(i, _)| i);
            emoji_name = emoji_name.get(..cut_off_idx).unwrap_or(emoji_name);
            let emoji_name = if emoji_name_len == emoji_name.len() {
                emoji_name.to_owned()
            } else {
                emoji_name.to_owned() + "..."
            };
            let preview_name = widget::text::title4(emoji_name);
            right_preview = right_preview.push(preview_name);

            if let Some(shortcode) = emoji_hovered.shortcode() {
                right_preview = right_preview.push(widget::text::body(shortcode))
            }
            if show_unicode {
                let unicode_chars = emoji_hovered
                    .as_str()
                    .chars()
                    .map(|c| format!("U+{:X}", c as u32))
                    .collect::<Vec<_>>()
                    .join(" ");
                right_preview = right_preview.push(widget::text::caption(unicode_chars));
            }

            preview = preview.push(right_preview);
            preview.apply(Element::from)
        } else if let Some(group) = self.selected_group {
            widget::text::title1(group_string(group)).into()
        } else {
            widget::text::title1(fl!("emojis-and-favorites")).into()
        };
        return preview;
    }

    fn update_group(
        &mut self,
        group: Option<emojis::Group>,
    ) -> Command<cosmic::app::Message<Message>> {
        self.emoji_hovered = None;
        self.selected_group = group;
        return scrollable::scroll_to(
            self.scrollable_id.clone(),
            scrollable::AbsoluteOffset::default(),
        );
    }
}
macro_rules! icon {
    ($name:expr) => {{
        concat!("dev.dominiccgeh.", $name)
    }};
}
// todo icon cache
fn group_icon(group: emojis::Group) -> &'static str {
    let icon = match group {
        emojis::Group::SmileysAndEmotion => icon!("emotion-satisfied"),
        emojis::Group::PeopleAndBody => icon!("people-nearby"),
        emojis::Group::AnimalsAndNature => icon!("pets"),
        emojis::Group::FoodAndDrink => icon!("food"),
        emojis::Group::TravelAndPlaces => icon!("world-1"),
        emojis::Group::Activities => icon!("walking"),
        emojis::Group::Objects => icon!("objects-column"),
        emojis::Group::Symbols => icon!("symbols"),
        emojis::Group::Flags => icon!("black-flag-icon"),
    };
    icon
}
// todo switch to array chunks once stable
fn chunks<T, const N: usize>(
    mut iter: impl Iterator<Item = T>,
) -> impl Iterator<Item = [Option<T>; N]> {
    let mut is_break = false;
    iter::from_fn(move || {
        if is_break {
            return None;
        }
        let array = [(); N].map(|_| {
            let next = iter.next();
            is_break = is_break || next.is_none();
            next
        });
        if array[0].is_none() {
            return None;
        }
        Some(array)
    })
}

fn group_from_key(key: u8) -> Option<emojis::Group> {
    use emojis::Group::*;
    let group = match key {
        b'1' => SmileysAndEmotion,
        b'2' => PeopleAndBody,
        b'3' => AnimalsAndNature,
        b'4' => FoodAndDrink,
        b'5' => TravelAndPlaces,
        b'6' => Activities,
        b'7' => Objects,
        b'8' => Symbols,
        b'9' => Flags,
        _ => return None,
    };
    return Some(group);
}
fn key_from_group(group: Option<emojis::Group>) -> u8 {
    use emojis::Group::*;
    let group = match group {
        Some(SmileysAndEmotion) => b'1',
        Some(PeopleAndBody) => b'2',
        Some(AnimalsAndNature) => b'3',
        Some(FoodAndDrink) => b'4',
        Some(TravelAndPlaces) => b'5',
        Some(Activities) => b'6',
        Some(Objects) => b'7',
        Some(Symbols) => b'8',
        Some(Flags) => b'9',
        None => b'0',
    };
    return group;
}

fn group_string(group: emojis::Group) -> String {
    match group {
        emojis::Group::SmileysAndEmotion => fl!("smileys-and-emotion"),
        emojis::Group::PeopleAndBody => fl!("people-and-body"),
        emojis::Group::AnimalsAndNature => fl!("animals-and-nature"),
        emojis::Group::FoodAndDrink => fl!("food-and-drink"),
        emojis::Group::TravelAndPlaces => fl!("travel-and-places"),
        emojis::Group::Activities => fl!("activities"),
        emojis::Group::Objects => fl!("objects"),
        emojis::Group::Symbols => fl!("symbols"),
        emojis::Group::Flags => fl!("flags"),
    }
}

fn format_emoji(emoji: &emojis::Emoji, show_unicode: bool) -> String {
    return if !show_unicode {
        emoji.name().to_string()
    } else {
        format!(
            "{} - {}",
            emoji.name(),
            emoji
                .as_str()
                .chars()
                .map(|c| format!("U+{:X}", c as u32))
                .collect::<Vec<_>>()
                .join(" ")
        )
    };
}

fn navigation_subscription() -> Subscription<Message> {
    use cosmic::iced::event;
    cosmic::iced_futures::event::listen_with(|event, status| {
        if status == event::Status::Captured {
            return None;
        }
        let event::Event::Keyboard(key_event) = event else {
            return None;
        };

        let cosmic::iced_runtime::keyboard::Event::KeyReleased { key, .. } = key_event else {
            return None;
        };
        match key {
            cosmic::iced_runtime::keyboard::Key::Named(key_named) => match key_named {
                cosmic::iced::keyboard::key::Named::Escape => return Some(Message::Exit),
                cosmic::iced::keyboard::key::Named::ArrowRight => return Some(Message::ArrowRight),
                cosmic::iced::keyboard::key::Named::ArrowLeft => return Some(Message::ArrowLeft),
                cosmic::iced::keyboard::key::Named::End => {
                    return Some(Message::ScrollToPercent(1))
                }
                cosmic::iced::keyboard::key::Named::Home => {
                    return Some(Message::ScrollToPercent(0))
                }

                _ => {}
            },
            cosmic::iced_runtime::keyboard::Key::Character(key_character) => {
                if key_character == "/" {
                    return Some(Message::FocusTextInput);
                }
                if key_character.len() == 1 && key_character.as_bytes()[0].is_ascii_digit() {
                    return Some(Message::Group(group_from_key(key_character.as_bytes()[0])));
                }
            }
            _ => {}
        }
        return None;
    })
}
