use clap::{App, Arg};
use pagebreak::PagebreakRunner;
use std::time::Instant;
use std::{env, path::PathBuf};

fn main() {
    let start = Instant::now();
    let matches = App::new("Pagebreak")
        .version("1.0")
        .author("CloudCannon")
        .about("Framework agnostic website pagination")
        .arg(
            Arg::with_name("source")
                .short("s")
                .long("source")
                .value_name("PATH")
                .help("Sets the source directory of the website to parse")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("PATH")
                .help("Sets the output directory")
                .required(true)
                .takes_value(true),
        )
        .get_matches();

    let mut runner = PagebreakRunner::new(
        env::current_dir().unwrap(),
        PathBuf::from(matches.value_of("source").unwrap_or(".")),
        PathBuf::from(matches.value_of("output").unwrap()),
    );

    runner.run();

    let duration = start.elapsed();
    println!(
        "Pagebreak: Finished in {}.{} seconds",
        duration.as_secs(),
        duration.subsec_millis()
    );
}
