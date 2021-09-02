use kuchiki::{traits::TendrilSink, NodeRef};
use rayon::prelude::*;
use state::*;
use std::{fs, io::Read, path::PathBuf};

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

    fn copy_source_to_output(&self) {
        // Not Yet Implemented
        // copy the source directory to the output directory
        // TODO: This also needs to clean up the output directory
        // fs::create_dir_all(&output_path).unwrap();
        // let options = CopyOptions::new();
        // copy(&source, &output_path, &options).unwrap();
    }

    fn read_pages(&mut self) {
        let source = self.full_source_path();
        let pages = read_pages(&source)
            .into_par_iter()
            .filter(|page| page.contains_pagination())
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

    fn paginate(&self, input_path: &PathBuf, output_path: &PathBuf) {
        let file_path = self.path.strip_prefix(&input_path).unwrap();

        let mut state = PagebreakState::new(
            Some(self.parse()),
            file_path.to_owned(),
            output_path.to_owned(),
        );

        state.hydrate();
        state.log_hydrated();
        state.paginate();
    }
}

fn read_pages(path: &PathBuf) -> Vec<SourcePage> {
    let globwalker = globwalk::GlobWalkerBuilder::from_patterns(&path, &["*.html"])
        .build()
        .unwrap();

    let mut pages = vec![];
    globwalker.for_each(|entry| {
        let entry = entry.unwrap();
        let path = entry.path().to_owned();

        pages.push(SourcePage {
            path: path.to_path_buf(),
            source: None,
        });
    });

    pages.par_iter_mut().for_each(|page| {
        let mut file = fs::File::open(&page.path).unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();

        page.source = Some(content);
    });

    pages
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     fn gfu(input: &PathBuf, url_format: &str, is_path: &str, when_num: usize) {
//         assert_eq!(
//             PathBuf::from(is_path),
//             get_file_url(input, url_format, when_num).unwrap()
//         );
//     }

//     #[test]
//     fn test_get_file_url() {
//         let input = PathBuf::from("about/index.html");
//         let url_format = "./page/:num/";
//         gfu(&input, url_format, "about/index.html", 0);
//         gfu(&input, url_format, "about/page/2/index.html", 1);

//         let input = PathBuf::from("index.html");
//         let url_format = "./page/:num/";
//         gfu(&input, url_format, "index.html", 0);
//         gfu(&input, url_format, "page/2/index.html", 1);

//         let input = PathBuf::from("a/b/c/index.html");
//         let url_format = "../../page/:num/";
//         gfu(&input, url_format, "a/b/c/index.html", 0);
//         gfu(&input, url_format, "a/page/2/index.html", 1);
//     }

//     #[test]
//     fn test_bad_file_url() {
//         let input = PathBuf::from("index.html");
//         let url_format = "../../page/:num/";
//         assert_eq!(
//             errors::PageErrorCode::ParentDir,
//             get_file_url(&input, url_format, 1).unwrap_err().code
//         );
//     }
// }
