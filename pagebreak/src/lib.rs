use kuchiki::ElementData;
use kuchiki::{traits::TendrilSink, NodeDataRef, NodeRef};
use path_clean::PathClean;
use rayon::prelude::*;
use std::path::Component::ParentDir;
use std::{fs, io::Read, path::Path, path::PathBuf};

mod errors;

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

#[derive(Debug)]
struct PagebreakNode {
    element: NodeRef,
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
        let relative_file_path = self.path.strip_prefix(&input_path).unwrap();
        let parsed = self.parse();

        let pagebreak_element = parsed.select("[data-pagebreak]").unwrap().next().unwrap();
        let (mut children, indentation) = find_pagination_children(&pagebreak_element);
        let (page_url_format, per_page) = parse_pagebreak_element(&pagebreak_element);

        let page_count = (children.len() + per_page - 1) / per_page;

        println!(
            "Pagebreak: Found {} items on {:?}; Building {} pages of size {}",
            children.len(),
            relative_file_path,
            page_count,
            per_page
        );

        // Detach all elements from the pagination node.
        pagebreak_element.as_node().children().for_each(|child| {
            child.detach();
        });

        for page_number in 0..page_count {
            let max_count = per_page.min(children.len());

            children.drain(0..max_count).for_each(|element| {
                indent_for_next_element(&pagebreak_element, &indentation);
                pagebreak_element.as_node().append(element.element);
            });
            indent_for_next_element(&pagebreak_element, &indentation);

            let cleaned_file_url = get_file_url(&relative_file_path, &page_url_format, page_number);
            let file_url = match cleaned_file_url {
                Ok(url) => url,
                Err(err) => {
                    eprintln!("{:?}\nPagebreak: Skipping errored page", err);
                    return;
                }
            };

            let output_file_path = output_path.join(file_url);
            fs::create_dir_all(&output_file_path.parent().unwrap()).unwrap();
            write_document_to_disk(&parsed, output_file_path);

            pagebreak_element.as_node().children().for_each(|child| {
                child.detach();
            });
        }
    }
}

fn write_document_to_disk(document: &NodeRef, path: PathBuf) {
    let mut file = std::io::BufWriter::new(std::fs::File::create(path).unwrap());
    document.serialize(&mut file).unwrap();
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

fn find_pagination_children(element: &NodeDataRef<ElementData>) -> (Vec<PagebreakNode>, String) {
    let mut children = vec![];
    let mut nodes = element.as_node().children();

    let first_child = nodes.next().unwrap();
    let mut indentation = "\n".to_string();
    if first_child.as_text().is_some() {
        let val = first_child.as_text().unwrap().borrow();
        indentation = val.to_string();
    } else if first_child.as_element().is_some() {
        children.push(PagebreakNode {
            element: first_child,
        });
    }

    for element in nodes {
        // skip text nodes
        if element.as_element().is_some() {
            children.push(PagebreakNode { element: element });
        }
    }

    (children, indentation)
}

fn parse_pagebreak_element(element: &NodeDataRef<ElementData>) -> (String, usize) {
    let pagination_attributes = element.as_node().as_element().unwrap().attributes.borrow();
    (
        pagination_attributes
            .get("data-pagebreak-url")
            .unwrap_or("./page/:num/")
            .to_string(),
        pagination_attributes
            .get("data-pagebreak")
            .unwrap_or("2")
            .parse::<usize>()
            .unwrap(),
    )
}

fn indent_for_next_element(element: &NodeDataRef<ElementData>, indentation: &String) {
    element
        .as_node()
        .append(NodeRef::new_text(indentation));
}

fn get_file_url(
    relative_file_path: &Path,
    page_url_format: &str,
    page_number: usize,
) -> Result<PathBuf, errors::PageError> {
    match page_number {
        0 => Ok(PathBuf::from(relative_file_path)),
        _ => {
            let page_number = (&page_number + 1).to_string();
            let file_url = page_url_format.replace(":num", &page_number);
            let file_path = PathBuf::from(file_url).join(relative_file_path.file_name().unwrap());
            let cleaned_path = relative_file_path.parent().unwrap().join(file_path).clean();

            match cleaned_path.components().next().unwrap() {
                ParentDir => Err(errors::PageError {
                    code: errors::PageErrorCode::ParentDir,
                    relative_path: relative_file_path.to_str().unwrap().to_string(),
                    message: format!(
                        "Pagination URL resolves outside of output directory: {:?}",
                        cleaned_path
                    ),
                }),
                _ => Ok(cleaned_path),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gfu(input: &PathBuf, url_format: &str, is_path: &str, when_num: usize) {
        assert_eq!(
            PathBuf::from(is_path),
            get_file_url(input, url_format, when_num).unwrap()
        );
    }

    #[test]
    fn test_get_file_url() {
        let input = PathBuf::from("about/index.html");
        let url_format = "./page/:num/";
        gfu(&input, url_format, "about/index.html", 0);
        gfu(&input, url_format, "about/page/2/index.html", 1);

        let input = PathBuf::from("index.html");
        let url_format = "./page/:num/";
        gfu(&input, url_format, "index.html", 0);
        gfu(&input, url_format, "page/2/index.html", 1);

        let input = PathBuf::from("a/b/c/index.html");
        let url_format = "../../page/:num/";
        gfu(&input, url_format, "a/b/c/index.html", 0);
        gfu(&input, url_format, "a/page/2/index.html", 1);
    }

    #[test]
    fn test_bad_file_url() {
        let input = PathBuf::from("index.html");
        let url_format = "../../page/:num/";
        assert_eq!(
            errors::PageErrorCode::ParentDir,
            get_file_url(&input, url_format, 1).unwrap_err().code
        );
    }
}
