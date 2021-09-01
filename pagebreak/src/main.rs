use rayon::prelude::*;
use kuchiki::{NodeRef, traits::TendrilSink};
use clap::{Arg, App};
use std::{env, fs, io::Read, path::PathBuf};
use std::time::{Instant};

fn main() {
    let start = Instant::now();
    let matches = App::new("Pagebreak")
        .version("1.0")
        .author("CloudCannon")
        .about("Framework agnostic website pagination")
        .arg(Arg::with_name("source")
            .short("s")
            .long("source")
            .value_name("PATH")
            .help("Sets the source directory of the website to parse")
            .takes_value(true))
        .arg(Arg::with_name("output")
            .short("o")
            .long("output")
            .value_name("PATH")
            .help("Sets the output directory")
            .required(true)
            .takes_value(true))
        .get_matches();

    let cwd = env::current_dir().unwrap();
    let source_path = PathBuf::from(matches.value_of("source").unwrap_or("."));
    let full_source_path = cwd.join(source_path);
    let source = match fs::canonicalize(&full_source_path) {
        Ok(path) => path,
        Err(_) => {
            eprintln!("Pagebreak error: couldn't find source directory: {:?}", full_source_path);
            std::process::exit(1);
        },
    };
    let output_path = PathBuf::from(matches.value_of("output").unwrap());

    // copy the source directory to the output directory
    // TODO: This also needs to clean up the output directory
    // fs::create_dir_all(&output_path).unwrap();
    // let options = CopyOptions::new();
    // copy(&source, &output_path, &options).unwrap();

    let mut pages: Vec<SourcePage> = read_pages(&source)
        .into_par_iter()
        .filter(|page| page.contains_pagination())
        .collect();

    println!("Pagebreak: Found {} pages with pagination", pages.len());

    pages.iter_mut().for_each(|page| {
        page.paginate(&source, &output_path);
    });

    let duration = start.elapsed();
    println!("Pagebreak: Finished in {}.{} seconds", duration.as_secs(), duration.subsec_millis());
}

#[derive(Debug)]
struct PagebreakNode {
    element: NodeRef,
    parent: NodeRef,
}

struct SourcePage {
    path: PathBuf,
    source: String,
}

impl SourcePage {
    fn contains_pagination(&self) -> bool {
        self.source.contains("data-pagebreak")
    }

    fn parse(&self) -> NodeRef {
        kuchiki::parse_html().one(self.source.as_str())
    }

    fn paginate(&self, input_path: &PathBuf, output_path: &PathBuf) {
        let relative_file_path = self.path.strip_prefix(&input_path).unwrap();
        let parsed = self.parse();

        let mut elements = vec![];
        let pagebreak_element = parsed.select("[data-pagebreak]").unwrap().next().unwrap();
        let children = pagebreak_element.as_node().children();
        for element in children {
            // skip text nodes
            let is_element = element.as_element().is_some();
            if is_element {
                let parent = element.parent().unwrap();
                elements.push(PagebreakNode { element: element, parent: parent });
            }
        }

        let pagination_attributes = &pagebreak_element.as_node().as_element().unwrap().attributes.borrow();
        let page_url_format = pagination_attributes
            .get("data-pagebreak-url")
            .unwrap_or(":url/page/:num/");
        let per_page = pagination_attributes
            .get("data-pagebreak")
            .unwrap_or("2")
            .parse::<usize>()
            .unwrap();
        let page_count = (elements.len() + per_page - 1) / per_page;

        println!("Pagebreak: Found {} items on {:?}; Building {} pages of size {}", elements.len(), relative_file_path, page_count, per_page);

        // Detach all elements from their parents.
        elements.iter_mut().for_each(|element| {
            element.element.detach()
        });
        // let mut stdout = Box::new(std::io::stdout()) as Box<dyn std::io::Write>;
        // self.parsed.as_ref().unwrap().serialize(&mut stdout).unwrap();

        for page_number in 0..page_count {
            let mut paginated_elements = vec![];
            let max_count = per_page.min(elements.len());

            elements.drain(0..max_count).for_each(|element| {
                element.parent.append(element.element);
                paginated_elements.push(element.parent.children().last().unwrap());
            });

            let mut file_url: &str = &page_url_format
                .replace(":url", relative_file_path.parent().unwrap().to_str().unwrap())
                .replace(":num", &page_number.to_string());
            if file_url[0..1] == "/".to_string() {
                file_url = &file_url[1..];
            }
            let page_directory = match page_number {
                0 => output_path.join(relative_file_path.parent().unwrap()),
                _ => output_path.join(file_url),
            };
            let output_file_path = page_directory.join(relative_file_path.file_name().unwrap());
            fs::create_dir_all(&page_directory).unwrap();
            serialize(&parsed, output_file_path);

            paginated_elements.iter_mut().for_each(|element| {
                element.detach();
            });
        }
    }
}

fn serialize(document: &NodeRef, path: PathBuf) {
    let mut file = std::io::BufWriter::new(std::fs::File::create(path).unwrap());
    document.serialize(&mut file).unwrap();
}

fn read_pages(path: &PathBuf) -> Vec<SourcePage> {
    let globwalker = globwalk::GlobWalkerBuilder::from_patterns(&path, &["*.html"])
        .build()
        .unwrap();
    let mut pages = vec![];
    for entry in globwalker {
        let entry = entry.unwrap();
        let path = entry.path();

        let mut file = fs::File::open(path).unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();

        pages.push(SourcePage {
            path: path.to_path_buf(),
            source: content,
        });
    }
    pages
}
