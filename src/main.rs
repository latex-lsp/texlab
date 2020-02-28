use clap::{app_from_crate, crate_authors, crate_description, crate_name, crate_version, Arg};
use std::error;
use stderrlog::{ColorChoice, Timestamp};
use texlab::tex::Distribution;

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    let _ = Distribution::detect().await;

    let matches = app_from_crate!()
        .author("")
        .arg(
            Arg::with_name("verbosity")
                .short("v")
                .multiple(true)
                .help("Increase message verbosity"),
        )
        .arg(
            Arg::with_name("quiet")
                .long("quiet")
                .short("q")
                .help("No output printed to stderr"),
        )
        .get_matches();

    stderrlog::new()
        .module(module_path!())
        .verbosity(matches.occurrences_of("verbosity") as usize)
        .quiet(matches.is_present("quiet"))
        .timestamp(Timestamp::Off)
        .color(ColorChoice::Never)
        .init()
        .expect("failed to initialize logger");

    Ok(())
}
