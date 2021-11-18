use cucumber::gherkin::Table;
use pagebreak::PagebreakRunner;
use std::convert::Infallible;
use std::io::{Read, Write};
use std::{fs, path::PathBuf};
use tempfile::tempdir;

use async_trait::async_trait;
use cucumber::{World, WorldInit};

struct PagebreakOptions {
    source: String,
    output: String,
}

impl Default for PagebreakOptions {
    fn default() -> Self {
        Self {
            source: "source".to_string(),
            output: "output".to_string(),
        }
    }
}

impl From<&Table> for PagebreakOptions {
    fn from(step_table: &Table) -> Self {
        let mut options = PagebreakOptions::default();
        for row in &step_table.rows {
            match row[0].as_ref() {
                "source" => options.source = row[1].clone(),
                "output" => options.output = row[1].clone(),
                _ => panic!("Unknown Pagebreak option {}", row[1]),
            }
        }
        options
    }
}

#[derive(Debug, WorldInit)]
struct PagebreakWorld {
    tmp_dir: Option<tempfile::TempDir>,
}

impl PagebreakWorld {
    fn tmp_dir(&mut self) -> PathBuf {
        if self.tmp_dir.is_none() {
            self.tmp_dir = Some(tempdir().expect("testing on a system with a temp dir"));
        }
        self.tmp_dir
            .as_ref()
            .expect("just created")
            .path()
            .to_path_buf()
    }

    fn tmp_file_path(&mut self, filename: &str) -> PathBuf {
        let tmp_dir = self.tmp_dir();
        tmp_dir.join(PathBuf::from(filename))
    }

    fn write_file(&mut self, filename: &str, contents: &str) {
        let file_path = self.tmp_file_path(filename);
        fs::create_dir_all(file_path.parent().unwrap()).unwrap();

        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(contents.as_bytes()).unwrap();
    }

    fn read_file(&mut self, filename: &str) -> String {
        let file_path = self.tmp_file_path(filename);
        let mut file = std::fs::File::open(&file_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        contents
    }

    fn check_file_exists(&mut self, filename: &str) -> bool {
        self.tmp_file_path(filename).exists()
    }

    fn run_pagebreak(&mut self, options: PagebreakOptions) {
        let mut runner = PagebreakRunner::new(
            self.tmp_dir(),
            PathBuf::from(options.source),
            PathBuf::from(options.output),
        );
        runner.run();
    }
}

/// `cucumber::World` needs to be implemented so this World is accepted in `Steps`
#[async_trait(?Send)]
impl World for PagebreakWorld {
    // We require some error type
    type Error = Infallible;

    async fn new() -> Result<Self, Infallible> {
        Ok(Self { tmp_dir: None })
    }
}

mod steps;

// This runs before everything else, so you can setup things here
fn main() {
    futures::executor::block_on(PagebreakWorld::run("features"));
}
