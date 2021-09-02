use crate::errors;
use kuchiki::{ElementData, NodeDataRef, NodeRef};
use path_clean::PathClean;
use std::path::Component;
use std::{fs, path::PathBuf};

#[derive(Debug)]
pub struct PagebreakNode {
    element: Option<NodeRef>,
}

impl PagebreakNode {
    pub fn new(element: NodeRef) -> Self {
        PagebreakNode {
            element: Some(element),
        }
    }
}

pub struct PagebreakState {
    pub document: NodeRef,
    file_path: PathBuf,
    output_path: PathBuf,
    page_container: Option<NodeDataRef<ElementData>>,
    page_items: Option<Vec<PagebreakNode>>,
    page_count: Option<usize>,
    per_page: Option<usize>,
    page_url_format: String,
    dom_indentation: String,
}

impl PagebreakState {
    pub fn new(document: NodeRef, file_path: PathBuf, output_path: PathBuf) -> Self {
        PagebreakState {
            document: document,
            file_path: file_path,
            output_path: output_path,
            page_container: None,
            page_items: None,
            page_count: None,
            per_page: None,
            page_url_format: "./page/:num/".to_string(),
            dom_indentation: "\n".to_string(),
        }
    }

    pub fn hydrate(&mut self) {
        self.find_pagebreak_node();
        if self.page_container.is_some() {
            self.read_pagebreak_node();
            self.find_pagination_children();
            self.page_count = Some(
                (self.page_items.as_ref().unwrap().len() + self.per_page.unwrap() - 1)
                    / self.per_page.unwrap(),
            );
        }
    }

    pub fn paginate(&mut self) {
        for page_number in 0..self.page_count.unwrap() {
            self.detach_children();
            let remaining_items: &mut Vec<PagebreakNode> = self.page_items.as_mut().unwrap();
            let max_count = self.per_page.unwrap().min(remaining_items.len());

            let current_page: Vec<PagebreakNode> = remaining_items.drain(0..max_count).collect();
            current_page.into_iter().for_each(|element| {
                self.indent_for_next_element();
                self.reattach_child(element);
            });
            self.indent_for_next_element();

            let cleaned_file_url = self.get_file_url(page_number);
            let file_url = match cleaned_file_url {
                Ok(url) => url,
                Err(err) => {
                    eprintln!("{:?}\nPagebreak: Skipping errored page", err);
                    return;
                }
            };

            let output_file_path = self.output_path.join(file_url);
            fs::create_dir_all(&output_file_path.parent().unwrap()).unwrap();
            self.write_current_document_to_disk(output_file_path);
        }
    }

    pub fn detach_children(&mut self) {
        self.page_container
            .as_ref()
            .unwrap()
            .as_node()
            .children()
            .for_each(|child| {
                child.detach();
            });
    }

    pub fn reattach_child(&mut self, mut child: PagebreakNode) {
        self.page_container
            .as_ref()
            .unwrap()
            .as_node()
            .append(child.element.take().unwrap());
    }

    fn find_pagebreak_node(&mut self) {
        self.page_container = self.document.select("[data-pagebreak]").unwrap().next();
    }

    fn read_pagebreak_node(&mut self) {
        let pagination_attributes = self
            .page_container
            .as_ref()
            .unwrap()
            .as_node()
            .as_element()
            .unwrap()
            .attributes
            .borrow();
        self.page_url_format = pagination_attributes
            .get("data-pagebreak-url")
            .unwrap_or("./page/:num/")
            .to_string();
        self.per_page = Some(
            pagination_attributes
                .get("data-pagebreak")
                .unwrap_or("2")
                .parse::<usize>()
                .unwrap(),
        );
    }

    fn find_pagination_children(&mut self) {
        let mut nodes = self.page_container.as_ref().unwrap().as_node().children();
        let mut children = vec![];

        let first_child = nodes.next().unwrap();
        if first_child.as_text().is_some() {
            let val = first_child.as_text().unwrap().borrow();
            self.dom_indentation = val.to_string();
        } else if first_child.as_element().is_some() {
            children.push(PagebreakNode::new(first_child));
        }

        for element in nodes {
            // skip text nodes
            if element.as_element().is_some() {
                children.push(PagebreakNode::new(element));
            }
        }

        self.page_items = Some(children);
    }

    fn indent_for_next_element(&mut self) {
        self.page_container
            .as_ref()
            .unwrap()
            .as_node()
            .append(NodeRef::new_text(&self.dom_indentation));
    }

    fn get_file_url(&mut self, page_number: usize) -> Result<PathBuf, errors::PageError> {
        match page_number {
            0 => Ok(PathBuf::from(&self.file_path)),
            _ => {
                let page_number = (&page_number + 1).to_string();
                let file_url = self.page_url_format.replace(":num", &page_number);
                let file_path = PathBuf::from(file_url).join(self.file_path.file_name().unwrap());
                let cleaned_path = self.file_path.parent().unwrap().join(file_path).clean();

                match cleaned_path.components().next().unwrap() {
                    Component::ParentDir => Err(errors::PageError {
                        code: errors::PageErrorCode::ParentDir,
                        relative_path: self.file_path.to_str().unwrap().to_string(),
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

    fn write_current_document_to_disk(&self, path: PathBuf) {
        let mut file = std::io::BufWriter::new(std::fs::File::create(path).unwrap());
        self.document.serialize(&mut file).unwrap();
    }
}

pub trait PagebreakStatusLogging {
    fn log_hydrated(&self);
}

impl PagebreakStatusLogging for PagebreakState {
    fn log_hydrated(&self) {
        println!(
            "Pagebreak: Found {} items on {:?}; Building {} pages of size {}",
            self.page_items.as_ref().unwrap().len(),
            self.file_path,
            self.page_count.unwrap(),
            self.per_page.unwrap()
        );
    }
}
