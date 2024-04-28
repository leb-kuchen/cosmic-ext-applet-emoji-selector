use crate::config::{Config, CONFIG_VERSION};
#[allow(unused_imports)]
use crate::fl;
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
const EMOJI_FONT_FAMILY: cosmic::iced::Font = iced::Font::with_name("Noto Color Emoji");
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
}
#[derive(Clone, Debug)]
pub enum Message {
    Config(Config),
    TogglePopup,
    PopupClosed(Id),
    Group(Option<emojis::Group>),
    Emoji(String),
    Search(String),
    SearchDo,
    Frame(std::time::Instant),
    Ignore,
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
        let window = Window {
            scrollable_id: widget::Id::unique(),
            selected_group,
            core,
            config: flags.config,
            config_handler: flags.config_handler,
            popup: None,
            search: String::new(),
            timeline: Timeline::new(),
        };
        let font_load =
            iced::font::load(include_bytes!("../data/NotoColorEmoji-Regular.ttf").as_slice())
                .map(|_| cosmic::app::message::app(Message::Ignore));
        (window, font_load)
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
            Message::Emoji(emoji) => {
                use wl_clipboard_rs::copy::{MimeType, Options, Source};
                return Command::perform(
                    // todo how long does this block?
                    async move {
                        let opts = Options::new();
                        _ = opts.copy(
                            Source::Bytes(emoji.into_bytes().into()),
                            MimeType::Autodetect,
                        );
                    },
                    |_| cosmic::app::message::app(Message::Ignore),
                );
            }
            Message::Search(search) => {
                self.search = search;
            }
            Message::SearchDo => {
                dbg!("to search");
            }
            Message::Group(group) => {
                if self.selected_group == group {
                    return Command::none();
                }
                self.selected_group = group;
                return scrollable::scroll_to(
                    self.scrollable_id.clone(),
                    scrollable::AbsoluteOffset::default(),
                );
            }
            Message::Ignore => {}
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
        let mut content = widget::column::with_capacity(3)
            .padding([space_xxs, space_none])
            .spacing(space_m);
        // .width(200);

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
                .center_x()
                .apply(Element::from);
            groups = groups.push(group_btn);
        }
        content = content.push(groups);

        let search = widget::search_input("Search for emojis", &self.search)
            .on_input(Message::Search)
            .on_paste(Message::Search)
            .on_submit(Message::SearchDo)
            .on_clear(Message::Search(String::new()))
            .width(Length::Fill);
        content = content.push(search);

        let mut grid = widget::column();

        // array_chunks isn't stable
        const GRID_SIZE: usize = 10;
        const EMOJI_NONE: Option<&emojis::Emoji> = None;
        let mut emojis: [Option<&emojis::Emoji>; 10] = [EMOJI_NONE; GRID_SIZE];
        let mut emoji_idx = 0;

        let emoji_row = |emojis: [Option<&emojis::Emoji>; 10]| {
            let mut row = widget::row::with_capacity(GRID_SIZE);
            for emoji in emojis.iter().filter_map(|e| *e) {
                // question: do you need to align emojis?
                // todo figure out button and text style
                let emoji_txt = widget::text(emoji.to_string())
                    .size(25)
                    .width(35)
                    .height(35)
                    .font(EMOJI_FONT_FAMILY)
                    .shaping(cosmic::iced_core::text::Shaping::Advanced)
                    .horizontal_alignment(alignment::Horizontal::Center);
                // .vertical_alignment(alignment::Vertical::Center);
                let emoji_btn = widget::button(emoji_txt)
                    .on_press(Message::Emoji(emoji.to_string()))
                    .style(cosmic::theme::Button::Icon);
                let emoji_tooltip = widget::tooltip(
                    emoji_btn,
                    emoji.name().to_string(),
                    widget::tooltip::Position::Top,
                );

                row = row.push(emoji_tooltip);
            }
            row
        };
        let emoji_iter: Box<dyn Iterator<Item = &'static emojis::Emoji>> = match self.selected_group
        {
            Some(group) => Box::from(group.emojis()),
            None => Box::from(emojis::iter()),
        };
        // switch back to grid?
        for emoji in emoji_iter {
            if !self.search.is_empty() && !emoji.name().contains(&self.search) {
                continue;
            }
            emojis[emoji_idx] = Some(emoji);
            emoji_idx += 1;
            if emoji_idx == GRID_SIZE {
                emoji_idx = 0;
                grid = grid.push(emoji_row(emojis));
            }
        }
        if emoji_idx != 0 {
            for i in emoji_idx..GRID_SIZE {
                emojis[i] = None
            }
            grid = grid.push(emoji_row(emojis));
        }
        // just hardcode the width for now,
        // as grid won't work if there are no search results
        // and there a currently no grid templates

        // todo figure out positioning after I have configured sccache and mold linker
        let grid = grid
            .apply(widget::container)
            .apply(widget::scrollable)
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

        Subscription::batch(vec![config, timeline])
    }

    fn style(&self) -> Option<<Theme as application::StyleSheet>::Style> {
        Some(cosmic::applet::style())
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
