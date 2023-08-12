use iced::widget::{column, container, horizontal_space, row, text};
use iced::{Alignment, Element, Length, Renderer};

use crate::core::api::series_information::SeriesMainInformation;
use crate::core::database;
use crate::gui::styles;

use super::Message;

pub fn watch_count() -> Element<'static, Message, Renderer> {
    let series_total_number = database::DB.get_total_series();
    let seasons_total_number = database::DB.get_total_seasons();
    let episodes_total_number = database::DB.get_total_episodes();

    let episodes_count = column![
        text(episodes_total_number)
            .size(31)
            .style(styles::text_styles::purple_text_theme()),
        text("Episodes").size(11),
    ]
    .align_items(Alignment::Center);

    let series_seasons_count = row![
        column![
            text(series_total_number)
                .size(31)
                .style(styles::text_styles::purple_text_theme()),
            text("Series").size(11)
        ]
        .align_items(Alignment::Center),
        horizontal_space(10),
        column![
            text(seasons_total_number)
                .size(31)
                .style(styles::text_styles::purple_text_theme()),
            text("Seasons").size(11)
        ]
        .align_items(Alignment::Center)
    ]
    .align_items(Alignment::Center);

    let content = column![
        text("You've seen a total of"),
        episodes_count,
        text("In exactly"),
        series_seasons_count,
    ]
    .align_items(Alignment::Center)
    .spacing(5);

    container(content)
        .width(Length::Fill)
        .padding(10)
        .center_x()
        .center_y()
        .style(styles::container_styles::first_class_container_rounded_theme())
        .into()
}

pub fn time_count(
    series_infos_and_time: &[(SeriesMainInformation, u32)],
) -> Element<'_, Message, Renderer> {
    let total_average_minutes: u32 = series_infos_and_time
        .iter()
        .map(|(_, average_runtime)| average_runtime)
        .sum();

    let total_minutes_count = column![
        text(total_average_minutes)
            .style(styles::text_styles::purple_text_theme())
            .size(31),
        text("Minutes").size(11)
    ]
    .align_items(Alignment::Center);

    let years = total_average_minutes / (60 * 24 * 365);
    let months = (total_average_minutes / (60 * 24 * 30)) % 12;
    let days = (total_average_minutes / (60 * 24)) % 30;
    let hours = (total_average_minutes / 60) % 24;

    let complete_time_count = row![
        column![
            text(years)
                .size(31)
                .style(styles::text_styles::purple_text_theme()),
            text("Years").size(11)
        ]
        .align_items(Alignment::Center),
        column![
            text(months)
                .size(31)
                .style(styles::text_styles::purple_text_theme()),
            text("Months").size(11)
        ]
        .align_items(Alignment::Center),
        column![
            text(days)
                .size(31)
                .style(styles::text_styles::purple_text_theme()),
            text("Days").size(11)
        ]
        .align_items(Alignment::Center),
        column![
            text(hours)
                .size(31)
                .style(styles::text_styles::purple_text_theme()),
            text("Hours").size(11)
        ]
        .align_items(Alignment::Center),
    ]
    .align_items(Alignment::Center)
    .spacing(10);

    let content = column![
        text("Total average time spent watching Series"),
        total_minutes_count,
        text("Which is exactly"),
        complete_time_count,
    ]
    .align_items(Alignment::Center)
    .spacing(5);

    container(content)
        .width(Length::Fill)
        .padding(10)
        .center_x()
        .center_y()
        .style(styles::container_styles::first_class_container_rounded_theme())
        .into()
}

pub mod series_banner {
    use bytes::Bytes;
    use iced::widget::{column, container, image, row, text, Space};
    use iced::{Alignment, Command, Element, Length, Renderer};

    use crate::core::caching;
    use crate::core::{api::series_information::SeriesMainInformation, database};
    use crate::gui::styles;

    #[derive(Debug, Clone)]
    pub enum Message {
        #[allow(dead_code)]
        BannerReceived(usize, Option<Bytes>),
    }

    impl Message {
        pub fn get_id(&self) -> usize {
            match self {
                Message::BannerReceived(id, _) => *id,
            }
        }
    }

    pub struct SeriesBanner {
        id: usize,
        series_info_and_time: (SeriesMainInformation, u32),
        banner: Option<Bytes>,
    }

    impl SeriesBanner {
        pub fn new(
            id: usize,
            series_info_and_time: (SeriesMainInformation, u32),
        ) -> (Self, Command<Message>) {
            let image_url = series_info_and_time
                .0
                .clone()
                .image
                .map(|image| image.medium_image_url);
            (
                Self {
                    id,
                    series_info_and_time,
                    banner: None,
                },
                image_url
                    .map(|image_url| {
                        Command::perform(caching::load_image(image_url), move |message| {
                            Message::BannerReceived(id, message)
                        })
                    })
                    .unwrap_or(Command::none()),
            )
        }

        pub fn update(&mut self, message: Message) {
            match message {
                Message::BannerReceived(_, banner) => self.banner = banner,
            }
        }

        pub fn view(&self) -> Element<'_, Message, Renderer> {
            let series_id = self.series_info_and_time.0.id;
            let series = database::DB.get_series(series_id).unwrap();

            let series_name = format!("{}: {}", self.id + 1, &self.series_info_and_time.0.name);
            let time_in_hours = self.series_info_and_time.1 / 60;
            let seasons = series.get_total_seasons();
            let episodes = series.get_total_episodes();

            let metadata = row![
                column![
                    text(time_in_hours)
                        .size(31)
                        .style(styles::text_styles::purple_text_theme()),
                    text("Hours").size(11)
                ]
                .align_items(Alignment::Center),
                column![
                    text(seasons)
                        .size(31)
                        .style(styles::text_styles::purple_text_theme()),
                    text("Seasons").size(11)
                ]
                .align_items(Alignment::Center),
                column![
                    text(episodes)
                        .size(31)
                        .style(styles::text_styles::purple_text_theme()),
                    text("episodes").size(11)
                ]
                .align_items(Alignment::Center),
            ]
            .align_items(Alignment::Center)
            .spacing(5);

            let banner: Element<'_, Message, Renderer> =
                if let Some(image_bytes) = self.banner.clone() {
                    let image_handle = image::Handle::from_memory(image_bytes);
                    image(image_handle).height(100).into()
                } else {
                    Space::new(71, 100).into()
                };

            let content = column![text(series_name), metadata]
                .spacing(5)
                .align_items(Alignment::Center);

            let content = row![banner, container(content).center_x().width(Length::Fill)]
                .spacing(5)
                .padding(5)
                .width(Length::Fill);

            container(content)
                .width(300)
                .style(styles::container_styles::first_class_container_rounded_theme())
                .padding(10)
                .center_x()
                .center_y()
                .into()
        }
    }
}