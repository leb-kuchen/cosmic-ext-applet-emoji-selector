use std::borrow::Cow;
use std::collections::HashMap;
use std::iter;

use crate::config::{Annotation, ClickMode, ColorButton, SkinToneMode};
use crate::config::{Config, CONFIG_VERSION};
#[allow(unused_imports)]
use crate::fl;
use crate::widget_copy;
use cosmic::app::Core;
use cosmic::iced;
use cosmic::iced::wayland::popup::{destroy_popup, get_popup};
use cosmic::iced::window::Id;
#[allow(unused_imports)]
use cosmic::iced::{alignment, Alignment, Length};
use cosmic::iced::{Command, Limits};
use cosmic::iced_core::mouse::click;
use cosmic::iced_futures::Subscription;
use cosmic::iced_runtime::core::window;
use cosmic::iced_style::application;
use cosmic::iced_widget::scrollable;
use cosmic::widget::{self};
use cosmic::{cosmic_config, iced_core};
use cosmic::{Apply, Element, Theme};
use cosmic_time::Timeline;
use regex::RegexBuilder;
pub const ID: &str = "dev.dominiccgeh.CosmicAppletEmojiSelector";
const ICON: &str = ID;
pub struct Window {
    all_emojis: Vec<&'static emojis::Emoji>,
    emojis_filtered: Vec<&'static emojis::Emoji>,
    annotations: HashMap<String, Annotation>,
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
    EmojiCopy(&'static emojis::Emoji, ClickMode),
    Search(String),
    Frame(std::time::Instant),
    EmojiHovered(&'static emojis::Emoji),
    Exit,
    FocusTextInput,
    Enter,
    ArrowRight,
    ArrowLeft,
    ScrollToPercent(u8),
    ToggleColorButton(usize),
}

#[derive(Clone, Debug)]
pub struct Flags {
    pub config_handler: Option<cosmic_config::Config>,
    pub config: Config,
    pub annotations: HashMap<String, Annotation>,
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

        let mut all_emojis = Vec::new();
        for emoji in emojis::iter() {
            match emoji.skin_tones() {
                Some(skin_tones) => all_emojis.extend(skin_tones),
                None => all_emojis.push(emoji),
            }
        }
        let emojis_filtered = all_emojis.iter().copied().collect();
        let window = Window {
            all_emojis,
            emojis_filtered,
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
            annotations: flags.annotations,
        };

        (
            window,
            cosmic::command::message(Message::Search(String::new())),
        )
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
            Message::ToggleColorButton(idx) => {
                let mut color_buttons = self.config.color_buttons.clone();
                if let Some(color_button) = color_buttons.get_mut(idx) {
                    color_button.active = !color_button.active;
                    let mut skin_tone_mode = self.config.skin_tone_mode;
                    skin_tone_mode.set(color_button.skin_tone_mode, color_button.active);
                    config_set!(skin_tone_mode, skin_tone_mode);
                    config_set!(color_buttons, color_buttons);
                    return cosmic::command::message(Message::Search(self.search.clone()));
                }
            }
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
            Message::EmojiCopy(emoji, click_mode) => {
                let mut commands = Vec::new();
                if click_mode.intersects(ClickMode::APPEND) {
                    self.search.push_str(emoji.as_str());
                }
                if click_mode.intersects(ClickMode::COPY) {
                    if !click_mode.intersects(ClickMode::PRIVATE) {
                        let mut last_used = self.config.last_used.clone();
                        if let Some(idx) = last_used.iter().position(|e| e == emoji.as_str()) {
                            last_used.remove(idx);
                        }
                        last_used.insert(0, emoji.to_string());
                        last_used.truncate(self.config.last_used_limit);
                        config_set!(last_used, last_used);
                    }
                    commands.push(iced::clipboard::write(emoji.to_string()))
                }

                if click_mode.intersects(ClickMode::CLOSE) {
                    commands.push(cosmic::command::message(Message::Exit));
                }
                return Command::batch(commands);
            }
            Message::Search(search) => {
                self.search = search;
                self.emoji_hovered = None;
                self.emojis_filtered.clear();
                let skin_tones_config = self.config.skin_tone_mode;
                let skin_tones_exact = skin_tones_config.intersects(SkinToneMode::ALL_EXACT);
                let skin_tones_mode_new = if skin_tones_exact {
                    SkinToneMode::new_exact
                } else {
                    SkinToneMode::new
                };
                for emoji in &self.all_emojis {
                    if Some(emoji.group()) != self.selected_group && !self.selected_group.is_none()
                    {
                        continue;
                    }
                    let emjoji_skin_tone_mode = emoji
                        .skin_tone()
                        .map_or(SkinToneMode::NO_SKIN, skin_tones_mode_new);
                    let config_skin_tone_contains_emoji = if skin_tones_exact {
                        skin_tones_config.intersects(emjoji_skin_tone_mode)
                    } else {
                        skin_tones_config.contains(emjoji_skin_tone_mode)
                    };
                    if config_skin_tone_contains_emoji
                        && (self.search.is_empty()
                            || self.emoji_name_localized(emoji).contains(&self.search))
                    {
                        self.emojis_filtered.push(emoji);
                    }
                }
            }
            Message::Group(group) => return self.update_group(group),

            Message::EmojiHovered(emoji) => self.emoji_hovered = Some(emoji),
            Message::Exit => {
                if let Some(p) = self.popup.take() {
                    return destroy_popup(p);
                }
            }
            Message::Enter => {
                config_set!(
                    middle_click_action,
                    ClickMode::COPY | ClickMode::APPEND | ClickMode::PRIVATE
                );
                config_set!(
                    skin_tone_mode,
                    SkinToneMode::NO_SKIN
                        | SkinToneMode::DEFAULT
                        | SkinToneMode::DARK
                        | SkinToneMode::FILTER_EXACT
                );
            }
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
    fn view_window(&self, _id: Id) -> Element<Self::Message> {
        let mut content = widget::column::with_capacity(3);
        // let theme = cosmic::theme::active();
        // let cosmic_theme = theme.cosmic();
        // search input start
        content = content.push(
            widget::search_input(fl!("search-for-emojis"), &self.search)
                .on_clear(Message::Search(String::new()))
                .on_paste(Message::Search)
                .on_input(Message::Search)
                .on_submit(Message::Enter),
        );
        // search input end

        // group icons start

        let mut groups = widget::row::with_capacity(9).width(Length::Fill);

        for group in emojis::Group::iter() {
            let is_selected = self.selected_group.is_some_and(|sel| sel == group);
            let group_btn =
                widget::button::icon(widget::icon::from_name(group_icon(group)).symbolic(true))
                    .on_press(Message::Group((!is_selected).then_some(group)));

            groups = groups.push(group_btn);
        }
        content = content.push(groups);

        // group icons end

        // preview start
        let preview_emoji_opt = self
            .emoji_hovered
            .or_else(|| self.emojis_filtered.first().copied());
        let mut preview_row = widget::row();
        match preview_emoji_opt {
            Some(preview_emoji) => {
                // dup 1
                let emoji_txt = widget::text(preview_emoji.as_str())
                    .size(35)
                    .width(50)
                    .height(50)
                    .font(self.font_family)
                    .shaping(iced_core::text::Shaping::Advanced)
                    .wrap(iced::widget::text::Wrap::None)
                    .horizontal_alignment(alignment::Horizontal::Center)
                    .vertical_alignment(alignment::Vertical::Center);
                preview_row = preview_row
                    .push(emoji_txt)
                    .push(widget::text(self.emoji_name_localized(preview_emoji)));
            }
            None => {
                let group_str = self
                    .selected_group
                    .map_or_else(|| fl!("emojis-and-favorites"), group_string);
                preview_row = preview_row.push(widget::text(group_str));
            }
        }
        // preview end
        let color_buttons_conf = &self.config.color_buttons;
        let mut color_buttons = widget::row::with_capacity(color_buttons_conf.len());
        for (idx, color_button) in color_buttons_conf.iter().enumerate() {
            let color = color_button.color;
            let active = color_button.active;
            let button_style = cosmic::theme::Button::Custom {
                active: Box::new(move |_selected, theme| {
                    color_button_apperance(color, Some(active), theme)
                }),
                disabled: Box::new(move |theme| color_button_apperance(color, None, theme)),
                hovered: Box::new(move |_selected, theme| {
                    color_button_apperance(color, Some(active), theme)
                }),
                pressed: Box::new(move |_selected, theme| {
                    color_button_apperance(color, Some(active), theme)
                }),
            };

            color_buttons = color_buttons.push(
                widget::button(widget::horizontal_space(0.1))
                    .width(20)
                    .height(20)
                    .style(button_style)
                    .on_press(Message::ToggleColorButton(idx)),
            );
        }
        preview_row = preview_row.push(color_buttons);

        content = content.push(preview_row);

        let mut emojis = Vec::with_capacity(self.emojis_filtered.len());

        let left_click_action = self.config.left_click_action;
        let right_click_action = self.config.right_click_action;
        let middle_click_action = self.config.middle_click_action;

        for emoji in &self.emojis_filtered {
            // dup 1
            let emoji_txt = widget::text(emoji.as_str())
                .size(25)
                .width(35)
                .height(35)
                .font(self.font_family)
                .shaping(iced_core::text::Shaping::Advanced)
                .wrap(iced::widget::text::Wrap::None)
                .horizontal_alignment(alignment::Horizontal::Center)
                .vertical_alignment(alignment::Vertical::Center);

            let mut emoji_btn = widget::button(emoji_txt).style(cosmic::theme::Button::Transparent);
            if left_click_action != ClickMode::NONE {
                emoji_btn = emoji_btn.on_press(Message::EmojiCopy(emoji, left_click_action));
            }
            let mut emoji_mouse_area =
                widget::mouse_area(emoji_btn).on_mouse_enter(Message::EmojiHovered(emoji));

            if right_click_action != ClickMode::NONE {
                emoji_mouse_area = widget::mouse_area(emoji_mouse_area)
                    .on_right_release(Message::EmojiCopy(emoji, right_click_action))
            }
            if middle_click_action != ClickMode::NONE {
                emoji_mouse_area = widget::mouse_area(emoji_mouse_area)
                    .on_middle_release(Message::EmojiCopy(emoji, middle_click_action))
            }
            emojis.push(emoji_mouse_area.into());
        }
        let flex_row = widget::flex_row(emojis);

        let container = flex_row
            .apply(widget::container)
            .apply(widget_copy::Scrollable::new)
            .id(self.scrollable_id.clone())
            .height(Length::Fill)
            .width(Length::Fill)
            .apply(widget::container)
            .width(Length::Fill)
            .height(500);
        content = content.push(container);
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

fn color_button_apperance(
    color: [f32; 4],
    selected: Option<bool>,
    theme: &Theme,
) -> widget::button::Appearance {
    let is_selected = selected.is_some_and(|s| s);
    widget::button::Appearance {
        background: Some(iced::Color::from(color).into()),
        border_radius: theme.cosmic().radius_s().into(),
        border_width: if is_selected { 2.0 } else { 0.0 },
        border_color: if is_selected {
            theme.cosmic().accent.border.into()
        } else {
            Default::default()
        },
        ..Default::default()
    }
}

impl Window {
    fn emoji_name_localized<'a>(&'a self, emoji: &'static emojis::Emoji) -> &'a str {
        let emoji_name = self
            .annotations
            .get(&emoji.as_str().replace(&['\u{fe0f}', '\u{fe0e}'], ""))
            .and_then(|annotation| annotation.tts.first().map(String::as_str))
            .unwrap_or_else(|| emoji.name());
        emoji_name
    }

    fn update_group(
        &mut self,
        group: Option<emojis::Group>,
    ) -> Command<cosmic::app::Message<Message>> {
        self.emoji_hovered = None;
        self.selected_group = group;
        return Command::batch([
            scrollable::scroll_to(
                self.scrollable_id.clone(),
                scrollable::AbsoluteOffset::default(),
            ),
            cosmic::command::message(Message::Search(self.search.clone())),
        ]);
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
