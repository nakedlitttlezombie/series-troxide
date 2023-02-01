pub mod cli;

use anyhow::{Context, Result};
use cli::{database_cli, episode_cli, season_cli, series_cli};
use troxide_core::{database::*, SeriesCollection, SeriesSort};

/// Entry point for series command-line option
pub fn run_series_command(series_cli: series_cli::SeriesCli) -> Result<()> {
    let database_path = get_database_path().context("Could not get the database path")?;

    let mut series_collection = SeriesCollection::load_series_with_db_path(&database_path)
        .context("Failed to load the database")?;

    match series_cli.command {
        series_cli::SeriesCommand::List(list_cli) => {
            let series_list;
            if let Some(sort_command) = list_cli.sort_command {
                series_list = series_collection.get_series_names_sorted(sort_command);
            } else {
                series_list = series_collection.get_series_names_sorted(SeriesSort::Default);
            };
            series_list.iter().for_each(|name| println!("{}", name));
        }
        series_cli::SeriesCommand::Add(series_add_cli) => {
            series_collection
                .add_series(series_add_cli.name, series_add_cli.episode_duration)
                .context("Failed to add series")?;

            series_collection
                .save_file(database_path)
                .context("Failed to save the series file")?;
        }
        series_cli::SeriesCommand::Remove(series_remove_cli) => {
            series_collection
                .remove_series(&series_remove_cli.name)
                .context("Could not remove series")?;

            series_collection
                .save_file(database_path)
                .context("Failed to save the series file")?;
        }
        series_cli::SeriesCommand::Summary(series_summary_cli) => {
            println!(
                "{}",
                series_collection.get_summary(&series_summary_cli.name)?
            );
        }
        series_cli::SeriesCommand::ChangeName(series_change_name_cli) => {
            series_collection
                .change_series_name(
                    &series_change_name_cli.old_name,
                    series_change_name_cli.new_name,
                )
                .context("Failed to change series name")?;

            series_collection
                .save_file(database_path)
                .context("Failed to save the series file")?;
        }
        series_cli::SeriesCommand::SeasonSummary(season_summary_cli) => {
            let seasons_summaries = series_collection
                .get_series(&season_summary_cli.name)?
                .get_seasons_summary();

            if let Some(season_summaries) = seasons_summaries {
                for summary in season_summaries {
                    println!("{}\n", summary)
                }
            } else {
                println!("No seasons found for series {}", season_summary_cli.name);
            }
        }
        series_cli::SeriesCommand::ChangeEpisodeDuration(series_change_duration_cli) => {
            series_collection
                .get_series_mut(&series_change_duration_cli.name)?
                .change_episode_duration(series_change_duration_cli.episode_duration);

            series_collection
                .save_file(database_path)
                .context("Failed to save the series file")?;
        }
        series_cli::SeriesCommand::WatchTime(watch_time_cli) => {
            let series = series_collection.get_series(&watch_time_cli.name)?;

            match watch_time_cli.watch_time_command {
                series_cli::WatchTimeCommand::Seconds => {
                    println!("{} seconds", series.get_total_watch_time().as_secs())
                }
                series_cli::WatchTimeCommand::Minutes => {
                    println!(
                        "{:.2} minutes",
                        series.get_total_watch_time().as_secs() as f32 / 60.0
                    )
                }
                series_cli::WatchTimeCommand::Hours => {
                    println!(
                        "{:.2} hours",
                        series.get_total_watch_time().as_secs() as f32 / (60 * 60) as f32
                    )
                }
                series_cli::WatchTimeCommand::Days => {
                    println!(
                        "{:.2} days",
                        series.get_total_watch_time().as_secs() as f32 / (60 * 60 * 24) as f32
                    )
                }
            }
        }
        series_cli::SeriesCommand::TotalWatchTime(total_watch_time_cli) => {
            match total_watch_time_cli.watch_time_command {
                series_cli::WatchTimeCommand::Seconds => {
                    println!(
                        "{} seconds",
                        series_collection.get_total_watch_time().as_secs()
                    )
                }
                series_cli::WatchTimeCommand::Minutes => {
                    println!(
                        "{:.2} minutes",
                        series_collection.get_total_watch_time().as_secs() as f32 / 60.0
                    )
                }
                series_cli::WatchTimeCommand::Hours => {
                    println!(
                        "{:.2} hours",
                        series_collection.get_total_watch_time().as_secs() as f32
                            / (60 * 60) as f32
                    )
                }
                series_cli::WatchTimeCommand::Days => {
                    println!(
                        "{:.2} days",
                        series_collection.get_total_watch_time().as_secs() as f32
                            / (60 * 60 * 24) as f32
                    )
                }
            }
        }
        series_cli::SeriesCommand::TotalEpisodes => {
            println! {"{} episodes", series_collection.get_total_episodes()};
        }
        series_cli::SeriesCommand::TotalSeasons => {
            println! {"{} seasons", series_collection.get_total_seasons()};
        }
        series_cli::SeriesCommand::TotalSeries => {
            println! {"{} series", series_collection.get_total_series()};
        }
    }

    Ok(())
}

/// Entry point for season command-line option
pub fn run_season_command(season_cli: season_cli::SeasonCli) -> Result<()> {
    let database_path = get_database_path().context("Could not get the database path")?;

    let mut series_collection = SeriesCollection::load_series_with_db_path(&database_path)
        .context("Failed to load the database")?;

    match season_cli.season_command {
        season_cli::SeasonCommand::Add(add_season_cli) => {
            series_collection
                .get_series_mut(&add_season_cli.series)?
                .add_season(add_season_cli.season)
                .context("Could not add season")?;
        }
        season_cli::SeasonCommand::Remove(remove_season_cli) => {
            series_collection
                .get_series_mut(&remove_season_cli.series)?
                .remove_season(remove_season_cli.season)
                .context("Could not remove season")?;
        }
        season_cli::SeasonCommand::AddRange(add_season_range_cli) => {
            let season_range = cli::RangeParser::get_range(&add_season_range_cli.season_range)?;

            series_collection
                .get_series_mut(&add_season_range_cli.series)?
                .add_season_range(season_range)
                .context("Could not add season range")?;
        }
    }

    series_collection
        .save_file(database_path)
        .context("Failed to save the series file")?;

    Ok(())
}

/// Entry point for episode command-line option
pub fn run_episode_command(episode_cli: episode_cli::EpisodeCli) -> Result<()> {
    let database_path = get_database_path().context("Could not get the database path")?;

    let mut series_collection = SeriesCollection::load_series_with_db_path(&database_path)
        .context("Failed to load the database")?;

    match episode_cli.episode_command {
        episode_cli::EpisodeCommand::Add(add_episode_cli) => {
            series_collection
                .get_series_mut(&add_episode_cli.series)?
                .add_episode(add_episode_cli.season, add_episode_cli.episode)
                .context("Could not add episode")?;
        }
        episode_cli::EpisodeCommand::Remove(remove_episode_cli) => {
            series_collection
                .get_series_mut(&remove_episode_cli.series)?
                .remove_episode(remove_episode_cli.season, remove_episode_cli.episode)
                .context("Could not remove episode")?;
        }
        episode_cli::EpisodeCommand::List(list_episode_cli) => {
            let episodes_summary = series_collection
                .get_series(&list_episode_cli.series)?
                .get_episodes_summary(list_episode_cli.season)?;

            println!("{}", episodes_summary);
        }
        episode_cli::EpisodeCommand::AddRange(add_episode_range_cli) => {
            let episode_range = cli::RangeParser::get_range(&add_episode_range_cli.episode_range)?;

            series_collection
                .get_series_mut(&add_episode_range_cli.series)?
                .add_episode_range(add_episode_range_cli.season, episode_range)
                .context("Could not add episode range")?;
        }
    }

    series_collection
        .save_file(database_path)
        .context("Failed to save the series file")?;

    Ok(())
}

pub fn run_database_command(database_cli: database_cli::DatabaseCli) -> Result<()> {
    match database_cli.database_command {
        database_cli::DatabaseCommand::Create { force } => {
            create_empty_database(force).context("Failed to create empty database")?;
        }
        database_cli::DatabaseCommand::Import(import_database_cli) => {
            let file_path = std::path::Path::new(&import_database_cli.file);
            import_database(file_path, import_database_cli.force)
                .context("Failed to import database")?
        }
        database_cli::DatabaseCommand::Export(export_database_cli) => {
            let destination_dir = std::path::PathBuf::from(export_database_cli.folder);
            export_database(destination_dir).context("Failed to export the database")?;
        }
    }
    Ok(())
}