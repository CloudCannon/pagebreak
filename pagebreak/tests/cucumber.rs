use async_trait::async_trait;
use std::convert::Infallible;
use tempfile;

pub struct PagebreakWorld {
    tmp_dir: Option<tempfile::TempDir>,
}

/// `cucumber::World` needs to be implemented so this World is accepted in `Steps`
#[async_trait(?Send)]
impl cucumber_rust::World for PagebreakWorld {
    // We require some error type
    type Error = Infallible;

    async fn new() -> Result<Self, Infallible> {
        Ok(Self { tmp_dir: None })
    }
}

mod steps;

// This runs before everything else, so you can setup things here
fn main() {
    let runner = cucumber_rust::Cucumber::<PagebreakWorld>::new()
        .features(&["./features"])
        .steps(steps::pagebreak::steps())
        .enable_capture(true);

    futures::executor::block_on(runner.run());
}
