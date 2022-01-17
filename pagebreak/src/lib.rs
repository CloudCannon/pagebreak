use kuchiki::{traits::TendrilSink, NodeRef};
use rayon::prelude::*;
use state::*;
use std::{
    fs::{self, copy, create_dir_all, remove_dir_all, remove_file},
    io::Read,
    path::{Path, PathBuf},
};

mod errors;
mod state;

pub struct PagebreakRunner {
    working_directory: PathBuf,
    source: PathBuf,
    output: PathBuf,
    pages: Option<Vec<SourcePage>>,
}

impl PagebreakRunner {
    pub fn new(working_directory: PathBuf, source: PathBuf, output: PathBuf) -> Self {
        PagebreakRunner {
            working_directory,
            source,
            output,
            pages: None,
        }
    }

    fn full_source_path(&self) -> PathBuf {
        let full_source_path = self.working_directory.join(&self.source);
        match fs::canonicalize(&full_source_path) {
            Ok(path) => path,
            Err(_) => {
                eprintln!(
                    "Pagebreak error: couldn't find source directory: {:?}",
                    full_source_path
                );
                std::process::exit(1);
            }
        }
    }

    fn full_output_path(&self) -> PathBuf {
        let full_output_path = self.working_directory.join(&self.output);
        fs::create_dir_all(&full_output_path).unwrap();
        match fs::canonicalize(&full_output_path) {
            Ok(path) => path,
            Err(_) => {
                eprintln!(
                    "Pagebreak error: couldn't create output directory: {:?}",
                    full_output_path
                );
                std::process::exit(1);
            }
        }
    }

    fn clean_output(&self) {
        let dest = self.full_output_path();
        let dest_globwalker = globwalk::GlobWalkerBuilder::from_patterns(&dest, &["*"])
            .build()
            .unwrap();

        dest_globwalker.for_each(|entry| {
            if let Ok(entry) = entry {
                if entry.file_type().is_dir() {
                    remove_dir_all(entry.path()).expect("Failed to clean directory from output");
                } else {
                    remove_file(entry.path()).expect("Failed to clean file from output");
                }
            }
        });
    }

    fn copy_source_to_output(&self) {
        let source = self.full_source_path();
        let dest = self.full_output_path();

        if source == dest {
            return;
        }

        let globwalker = globwalk::GlobWalkerBuilder::from_patterns(&source, &["**/*", "!*.html"])
            .build()
            .unwrap();

        self.clean_output();

        globwalker.for_each(|entry| {
            if let Ok(entry) = entry {
                if entry.file_type().is_file() {
                    self.copy_file_to_output(entry.path());
                }
            }
        });
    }

    fn copy_file_to_output(&self, path: &Path) {
        let source = self.full_source_path();
        let dest = self.full_output_path();

        if source == dest {
            return;
        }

        let relative_path = pathdiff::diff_paths(path, &source).unwrap();
        let dest_path = dest.join(relative_path);
        if let Some(parent) = dest_path.parent() {
            create_dir_all(parent).expect("Failed to create dir for output");
        }
        copy(path, dest_path).expect("Failed to copy file to output");
    }

    fn read_pages(&mut self) {
        let source = self.full_source_path();
        let pages = read_pages(&source)
            .into_par_iter()
            .filter(|page| {
                if page.contains_pagination() {
                    true
                } else {
                    self.copy_file_to_output(&page.path);
                    false
                }
            })
            .collect();
        self.pages = Some(pages);
    }

    fn paginate(&mut self) {
        let source = self.full_source_path();
        let output = self.full_output_path();
        let mut pages = self.pages.take().unwrap();
        pages.iter_mut().for_each(|page| {
            page.paginate(&source, &output);
        });
    }

    pub fn run(&mut self) {
        self.copy_source_to_output();
        self.read_pages();
        println!(
            "Pagebreak: Found {} pages with pagination",
            self.pages.as_ref().unwrap().len()
        );
        self.paginate();
    }
}

struct SourcePage {
    path: PathBuf,
    source: Option<String>,
}

impl SourcePage {
    fn contains_pagination(&self) -> bool {
        self.source.as_ref().unwrap().contains("data-pagebreak")
    }

    fn parse(&self) -> NodeRef {
        kuchiki::parse_html().one(self.source.as_ref().unwrap().as_str())
    }

    fn paginate(&self, input_path: &Path, output_path: &Path) {
        let file_path = self.path.strip_prefix(&input_path).unwrap();

        let mut state =
            PagebreakState::new(self.parse(), file_path.to_owned(), output_path.to_owned());

        state.hydrate();
        state.log_hydrated();
        state.paginate();
    }
}

fn read_pages(path: &Path) -> Vec<SourcePage> {
    let globwalker = globwalk::GlobWalkerBuilder::from_patterns(&path, &["**/*.html"])
        .build()
        .unwrap();

    let mut pages: Vec<SourcePage> = globwalker
        .map(|entry| {
            let entry = entry.unwrap();
            let path = entry.path().to_owned();

            SourcePage { path, source: None }
        })
        .collect();

    pages.par_iter_mut().for_each(|page| {
        let mut file = fs::File::open(&page.path).unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();

        page.source = Some(content);
    });

    pages
}
