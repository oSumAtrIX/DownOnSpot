#[macro_use]
extern crate log;

mod arg;
mod converter;
mod downloader;
mod error;
mod settings;
mod spotify;
mod tag;

use arg::Args;
use async_std::task;
use colored::Colorize;
use downloader::{DownloadState, Downloader};
use settings::Settings;
use spotify::Spotify;
use std::time::{Duration, Instant};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(not(windows))]
#[tokio::main]
async fn main() {
	start().await;
}

#[cfg(windows)]
#[tokio::main]
async fn main() {
	use colored::control;

	//backwards compatibility.
	if control::set_virtual_terminal(true).is_ok() {};
	start().await;
}

async fn start() {
	env_logger::init();

	let args = Args::from_cli();

	let settings = match Settings::load().await {
		Ok(settings) => {
			println!(
				"{} {}.",
				"Settings successfully loaded.\nContinuing with spotify account:".green(),
				settings.username
			);
			settings
		}
		Err(e) => {
			println!(
				"{} {}...",
				"Settings could not be loaded, because of the following error:".red(),
				e
			);
			let default_settings = Settings::new("username", "password", "client_id", "secret");
			match default_settings.save().await {
				Ok(path) => {
					println!(
						"{}{}",
						"..but default settings have been created successfully. Edit them and run the program again.\nFind the settings file at: ".green(),
						path.to_string_lossy()
					);
				}
				Err(e) => {
					println!(
						"{} {}",
						"..and default settings could not be written:".red(),
						e
					);
				}
			};
			return;
		}
	};

	let spotify = match Spotify::new(
		&settings.username,
		&settings.password,
		&settings.client_id,
		&settings.client_secret,
		settings.market_country_code,
	)
	.await
	{
		Ok(spotify) => {
			println!("{}", "Login succeeded.".green());
			spotify
		}
		Err(e) => {
			println!(
				"{} {}",
				"Login failed, possibly due to invalid credentials or settings:".red(),
				e
			);
			return;
		}
	};

	let downloader = Downloader::new(settings.downloader, spotify);
	match downloader.handle_input(&args.input).await {
		Ok(search_results) => {
			if let Some(search_results) = search_results {
				print!("{esc}[2J{esc}[1;1H", esc = 27 as char);

				for (i, track) in search_results.iter().enumerate() {
					println!("{}: {} - {}", i + 1, track.author, track.title);
				}
				println!("{}", "Select the track (default: 1): ".green());

				let mut selection;
				loop {
					let mut input = String::new();
					std::io::stdin()
						.read_line(&mut input)
						.expect("Failed to read line");

					selection = input.trim().parse::<usize>().unwrap_or(1) - 1;

					if selection < search_results.len() {
						break;
					}
					println!("{}", "Invalid selection. Try again or quit (CTRL+C):".red());
				}

				let track = &search_results[selection];

				if let Err(e) = downloader
					.add_uri(&format!("spotify:track:{}", track.track_id))
					.await
				{
					error!(
						"{}",
						format!(
							"{}: {}",
							"Track could not be added to download queue.".red(),
							e
						)
					);
					return;
				}
			}

			let refresh = Duration::from_secs(settings.refresh_ui_seconds);
			let now = Instant::now();
			let mut time_elapsed: u64 = 0;

            let mut download_states = vec![DownloadState::None; downloader.get_downloads().await.len()];
            let mut messages = vec![];

			'outer: loop {
				print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
				let mut exit_flag: i8 = 1;

                let mut num_completed = 0;
                let mut num_err = 0;
                let mut num_waiting = 0;

                let mut current_download_view = String::new();


				for (i, download) in (&downloader.get_downloads().await).iter().enumerate() {
					let state = &download.state;
                    // println!("{} {:?}", download.title, state);

                    if state != &download_states[i]{
                        // println!("Changed {} from {:?} to {:?}", download.title, download_states[i], state);
                        download_states[i] = state.clone();
                        match state {
                            DownloadState::None => (),
                            DownloadState::Lock => (),
                            DownloadState::Downloading(_, _) => (),
                            DownloadState::Post => (),//messages.push(format!("{time_elapsed: >5}  Post       | {}", download.title)),
                            DownloadState::Done => messages.push(format!("{time_elapsed: >5}  Downloaded | {}", download.title)),
                            DownloadState::Error(e) => messages.push(format!("{time_elapsed: >5}  {e: >19} | {}", download.title))
                        };

                    }

                    if let Some(msg) = match state {
                        DownloadState::Downloading(r, t) => {
                            exit_flag &= 0;
                            let p = *r as f32 / *t as f32 * 100.0;
                            if p > 100.0 {
                                Some("100%".to_string())
                            } else {
                                Some(format!("{}%", p as i8))
                            }
                        }
                        DownloadState::Post => {
                            exit_flag &= 0;
                            Some("Postprocessing... ".to_string())
                        }
                        DownloadState::None | DownloadState::Lock => {
                            exit_flag &= 0;
                            num_waiting += 1;
                            None
                        }
                        DownloadState::Error(_) => {
                            num_err += 1;
                            None
                        }
                        DownloadState::Done => {
                            num_completed += 1;
                            None
                        }
                    }{
                        current_download_view.push_str(&format!("{: >4} | {}\n", msg, download.title));
                    }

                }

                while messages.len() > 8 {
                    messages.remove(0);
                }

                println!(" - DownOnSpot v{VERSION} -\n");
                
                println!("\nTime elapsed: {} second(s)\n", time_elapsed);

                println!(" Time | Event");
                for message in &messages {
                    println!("{}", message);
                }
                println!("\n\n - Current Downloads -");
                println!("{}", current_download_view);

                println!("");
                println!(" Waiting | Err/Skip | Done | Total");
                println!(" {: <8}| {: <9}| {: <5}| {}", num_waiting, num_err, num_completed, download_states.len());

                time_elapsed = now.elapsed().as_secs();
                if exit_flag == 1 {
                    break 'outer;
                }

                task::sleep(refresh).await
            }
            println!("Finished download(s) in {} second(s).", time_elapsed);
        }
        Err(e) => {
            error!("{} {}", "Handling input failed:".red(), e)
        }
    }
}


// !cargo r https://open.spotify.com/playlist/0CA17tlE54VyDsauutZJE1
