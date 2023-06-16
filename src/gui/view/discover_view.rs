use crate::core::api::episodes_information::Episode;
use crate::core::api::tv_schedule::get_episodes_with_date;
use crate::gui::Message as GuiMessage;
use episode_poster::Message as EpisodePosterMessage;

use iced::{
    widget::{row, scrollable, text, Column, Row},
    Command, Element, Length, Renderer,
};

#[derive(Default)]
enum LoadState {
    #[default]
    Loading,
    Loaded,
}

#[derive(Clone, Debug)]
pub enum Message {
    LoadSchedule,
    ScheduleLoaded(Vec<Episode>),
    EpisodePosterAction(/*poster index*/ usize, EpisodePosterMessage),
}

#[derive(Default)]
pub struct Discover {
    load_state: LoadState,
    episodes: Vec<episode_poster::EpisodePoster>,
}

impl Discover {
    pub fn update(&mut self, message: Message) -> Command<GuiMessage> {
        match message {
            Message::LoadSchedule => {
                if let LoadState::Loaded = self.load_state {
                    return Command::none();
                }
                return Command::perform(get_episodes_with_date("2023-06-16"), |episodes| {
                    GuiMessage::DiscoverAction(Message::ScheduleLoaded(
                        episodes.expect("Failed to load episodes schedule"),
                    ))
                });
            }
            Message::ScheduleLoaded(episodes) => {
                self.load_state = LoadState::Loaded;

                let mut episode_posters = Vec::with_capacity(episodes.len());
                let mut commands = Vec::with_capacity(episodes.len());
                for (index, episode) in episodes.into_iter().enumerate() {
                    let (poster, command) = episode_poster::EpisodePoster::new(index, episode);
                    episode_posters.push(poster);
                    commands.push(command);
                }

                self.episodes = episode_posters;
                return Command::batch(commands).map(|m| GuiMessage::DiscoverAction(m));
            }
            Message::EpisodePosterAction(index, message) => {
                return self.episodes[index]
                    .update(message)
                    .map(GuiMessage::DiscoverAction)
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message, Renderer> {
        match self.load_state {
            LoadState::Loading => row!(text("loading..."))
                .align_items(iced::Alignment::Center)
                .width(Length::Fill)
                .into(),
            LoadState::Loaded => scrollable(Column::with_children(
                self.episodes
                    .iter()
                    .enumerate()
                    .map(|(index, poster)| {
                        poster
                            .view()
                            .map(move |m| Message::EpisodePosterAction(index, m))
                    })
                    .collect(),
            ))
            .width(Length::Fill)
            .into(),
        }
    }
}

mod episode_poster {
    use core::panic;

    use crate::core::api::load_image;
    use crate::core::api::series_information::get_series_main_info_with_url;
    use crate::core::api::series_information::SeriesMainInformation;
    use crate::core::api::ApiError;
    use iced::widget::{column, image, text};
    use iced::{Command, Element, Renderer};

    use super::Episode;
    use super::Message as DiscoverMessage;

    #[derive(Clone, Debug)]
    pub enum Message {
        ImageLoaded(Option<Vec<u8>>),
        SeriesInformationLoaded(SeriesMainInformation),
    }

    pub struct EpisodePoster {
        index: usize,
        episode: Episode,
        image: Option<Vec<u8>>,
        series_belonging: Option<SeriesMainInformation>,
    }

    impl EpisodePoster {
        pub fn new(index: usize, episode: Episode) -> (Self, Command<DiscoverMessage>) {
            let episode_image = episode.image.clone();
            let series_url = episode.links.show.href.clone();

            let poster = Self {
                index,
                episode,
                image: None,
                series_belonging: None,
            };

            let series_information_command = Command::perform(
                async move {
                    loop {
                        match get_series_main_info_with_url(series_url.clone()).await {
                            Ok(series_info) => break series_info,
                            Err(err) => {
                                if let ApiError::Network(ref err) = err {
                                    if err.is_request() {
                                        // TODO: using random sleep time
                                        tokio::time::sleep(std::time::Duration::from_millis(5))
                                            .await;
                                        continue;
                                    }
                                }
                                panic!(
                                    "Failed to collect series information for discover: {:?}",
                                    err
                                )
                            }
                        }
                    }
                },
                move |series| {
                    DiscoverMessage::EpisodePosterAction(
                        index,
                        Message::SeriesInformationLoaded(series),
                    )
                },
            );

            (poster, series_information_command)
        }

        pub fn update(&mut self, message: Message) -> Command<DiscoverMessage> {
            match message {
                Message::ImageLoaded(image) => self.image = image,
                Message::SeriesInformationLoaded(series_info) => {
                    let series_image_url = series_info.image.clone();
                    let poster_index = self.index;
                    self.series_belonging = Some(series_info);

                    if let Some(image) = series_image_url {
                        return Command::perform(
                            load_image(image.medium_image_url),
                            move |image| {
                                DiscoverMessage::EpisodePosterAction(
                                    poster_index,
                                    Message::ImageLoaded(image),
                                )
                            },
                        );
                    }
                }
            }
            Command::none()
        }

        pub fn view(&self) -> Element<'_, Message, Renderer> {
            let mut content = column!().padding(2).spacing(1);
            if let Some(image_bytes) = self.image.clone() {
                let image_handle = image::Handle::from_memory(image_bytes);
                let image = image(image_handle).height(60);
                content = content.push(image);
            };

            if let Some(series_info) = &self.series_belonging {
                content = content.push(text(&series_info.name))
            }

            // content.push(text(&self.episode.name)).into()
            content.into()
        }
    }
}
