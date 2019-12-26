use url::Host;
use gleam_finder::*;
use std::thread;
use std::time::Duration;
use std::env;
use progress_bar::progress_bar::ProgressBar;
use progress_bar::color::{Color, Style};
use clap::*;
use url::{Url, ParseError};
use std::collections::HashMap;
use std::time::Instant;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Record {
    city: String,
    region: String,
    country: String,
    population: Option<u64>,
}

struct IntermediaryUrl {
    url: String,
    domain: Option<Url>
}

impl IntermediaryUrl {
    fn new_from_vec(urls: Vec<String>) -> Vec<Self> {
        let mut result: Vec<Self> = Vec::new();
        for url in urls {
            result.push(IntermediaryUrl::new(url));
        }
        result
    }

    fn new(url: String) -> Self {
        let mut result = Self {
            url,
            domain: None,
        };
        result.init();
        result
    }

    fn init(&mut self) {
        let domain = Url::parse(&self.url).unwrap();
        self.domain = Some(domain);
    }

    fn get_host(&self) -> Host<&str> {
        self.domain.as_ref().unwrap().host().unwrap()
    }

    fn get_url(&self) -> &str {
        &self.url
    }
}

fn main() {
    let matches = App::new("Gleam finder")
        .version("1.1")
        .author("Mubelotix <mubelotix@gmail.com>")
        .about("Search for gleam links on the web.")
        .arg(
            Arg::with_name("minimal")
                .long("minimal")
                .short("m")
                .help("Enables simplified mode: only results urls are printed; no progress bar and log informations")
        )
        .arg(
            Arg::with_name("force-cooldown")
                .long("force-cooldown")
                .short("f")
                .help("Force to sleep between every request, even between two differents website.")
        )
        .arg(
            Arg::with_name("cooldown")
                .long("cooldown")
                .takes_value(true)
                .min_values(0)
                .max_values(86400)
                .default_value("6")
                .help("Set the waiting time in seconds between two request to the same website.")
        )
        .arg(
            Arg::with_name("timeout")
                .long("timeout")
                .takes_value(true)
                .min_values(0)
                .max_values(3600)
                .default_value("6")
                .help("Set the timeout for a request.")
        )
        .get_matches();

    let minimal: bool = if matches.occurrences_of("minimal") > 0 {
        true
    } else {
        false
    };

    let cooldown: u64 = matches.value_of("cooldown").unwrap_or("6").parse().unwrap_or(6);
    env::set_var("MINREQ_TIMEOUT", matches.value_of("timeout").unwrap_or("6"));

    if !minimal {
        let mut progress_bar = ProgressBar::new(10);
        progress_bar.set_action("Searching", Color::White, Style::Normal);

        let mut results = Vec::new();
        let mut page = 0;
        loop {
            progress_bar.print_info("Getting", &format!("the results page {}", page), Color::Blue, Style::Normal);
            let new_results = google::search(page);
            if new_results.len() > 0 {
                results.append(&mut IntermediaryUrl::new_from_vec(new_results));
                page += 1;
                progress_bar.inc();
                progress_bar.print_info("Sleeping", &format!("for {} seconds", cooldown), Color::Yellow, Style::Normal);
                thread::sleep(Duration::from_secs(cooldown));
            } else {
                break;
            }
        }

        let mut progress_bar = ProgressBar::new(results.len());
        let mut timeout_check = HashMap::new();
        progress_bar.set_action("Loading", Color::White, Style::Normal);
        for link_idx in 0..results.len() {
            // verifying if the cooldown is respected
            if let Some(last_load_time) = timeout_check.get(&results[link_idx].get_host()) {
                let time_since_last_load = Instant::now() - *last_load_time;
                if time_since_last_load < Duration::from_secs(cooldown) {
                    let time_to_sleep = Duration::from_secs(cooldown) - time_since_last_load;
                    progress_bar.print_info("Sleeping", &format!("for {} seconds", time_to_sleep.as_secs() + 1), Color::Yellow, Style::Normal);
                    thread::sleep(time_to_sleep);
                }
            }
            
            progress_bar.print_info("Loading", results[link_idx].get_url(), Color::Blue, Style::Normal);
            for gleam_link in intermediary::resolve(results[link_idx].get_url()) {
                progress_bar.print_info("Found", &gleam_link, Color::LightGreen, Style::Bold);
            }
            progress_bar.inc();
            timeout_check.insert(results[link_idx].get_host(), Instant::now());
        }
    } else {
        let mut results = Vec::new();
        let mut page = 0;
        loop {
            let new_results = google::search(page);
            if new_results.len() > 0 {
                results.append(&mut IntermediaryUrl::new_from_vec(new_results));
                page += 1;
                thread::sleep(Duration::from_secs(cooldown));
            } else {
                break;
            }
        }

        let mut timeout_check = HashMap::new();
        for link_idx in 0..results.len() {
            // verifying if the cooldown is respected
            if let Some(last_load_time) = timeout_check.get(&results[link_idx].get_host()) {
                let time_since_last_load = Instant::now() - *last_load_time;
                if time_since_last_load < Duration::from_secs(cooldown) {
                    let time_to_sleep = Duration::from_secs(cooldown) - time_since_last_load;
                    thread::sleep(time_to_sleep);
                }
            }
            
            for gleam_link in intermediary::resolve(results[link_idx].get_url()) {
                println!("{}", gleam_link);
            }
            timeout_check.insert(results[link_idx].get_host(), Instant::now());
        }
    }
    
}