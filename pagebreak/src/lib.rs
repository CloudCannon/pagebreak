use rayon::prelude::*;
use kuchiki::{NodeRef, traits::TendrilSink};
use std::{fs, io::Read, path::PathBuf};
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
                eprintln!("Pagebreak error: couldn't find source directory: {:?}", full_source_path);
                std::process::exit(1);
            },
        }
    }

    fn full_output_path(&self) -> PathBuf {
        let full_output_path = self.working_directory.join(&self.output);
        fs::create_dir_all(&full_output_path).unwrap();
        match fs::canonicalize(&full_output_path) {
            Ok(path) => path,
            Err(_) => {
                eprintln!("Pagebreak error: couldn't create output directory: {:?}", full_output_path);
                std::process::exit(1);
            },
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
        println!("Pagebreak: Found {} pages with pagination", self.pages.as_ref().unwrap().len());
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
        
        let mut elements = vec![];
        let pagebreak_element = parsed.select("[data-pagebreak]").unwrap().next().unwrap();
        let mut children = pagebreak_element.as_node().children();
        let first_child = children.next().unwrap();
        
        let mut separator = "\n".to_string();
        if first_child.as_text().is_some() {
            let val = first_child.as_text().unwrap().borrow();
            separator = val.to_string();
        } else if first_child.as_element().is_some() {
            elements.push(PagebreakNode { element: first_child });
        }
        
        for element in children {
            // skip text nodes
            if element.as_element().is_some() {
                elements.push(PagebreakNode { element: element });
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
        pagebreak_element.as_node().children().for_each(|child| {
            child.detach();
        });
        // let mut stdout = Box::new(std::io::stdout()) as Box<dyn std::io::Write>;
        // self.parsed.as_ref().unwrap().serialize(&mut stdout).unwrap();
        
        for page_number in 0..page_count {
            let max_count = per_page.min(elements.len());
            
            pagebreak_element.as_node().append(NodeRef::new_text(&separator));
            elements.drain(0..max_count).for_each(|element| {
                pagebreak_element.as_node().append(element.element);
                pagebreak_element.as_node().append(NodeRef::new_text(&separator));
            });
            
            let mut file_url: &str = &page_url_format
            .replace(":url", relative_file_path.parent().unwrap().to_str().unwrap())
            .replace(":num", &(&page_number + 1).to_string());
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
            
            pagebreak_element.as_node().children().for_each(|child| {
                child.detach();
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
