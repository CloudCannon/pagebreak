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

#[derive(Debug, PartialEq)]
enum PagebreakControlType {
    Next,
    Previous,
    NoNext,
    NoPrevious,
    Current,
    Total,
    None,
}
struct PagebreakControl {
    element: NodeRef,
    parent: Option<NodeRef>,
    previous_sibling: Option<NodeRef>,
    control_type: PagebreakControlType,
}

impl PagebreakControl {
    pub fn new(
        element: NodeRef,
        control_type: PagebreakControlType,
        parent: Option<NodeRef>,
        previous_sibling: Option<NodeRef>,
    ) -> Self {
        PagebreakControl {
            element,
            parent,
            previous_sibling,
            control_type,
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
    page_meta_format: String,
    dom_indentation: String,
    controls: Option<Vec<PagebreakControl>>,
}

impl PagebreakState {
    pub fn new(document: NodeRef, file_path: PathBuf, output_path: PathBuf) -> Self {
        PagebreakState {
            document,
            file_path,
            output_path,
            page_container: None,
            page_items: None,
            page_count: None,
            per_page: None,
            page_url_format: "./page/:num/".to_string(),
            page_meta_format: ":content | Page :num".to_string(),
            dom_indentation: "\n".to_string(),
            controls: None,
        }
    }

    pub fn hydrate(&mut self) {
        self.find_pagebreak_node();
        if self.page_container.is_some() {
            self.read_meta_format();
            self.read_pagebreak_node();
            self.find_pagination_children();
            self.find_pagebreak_controls();
            self.page_count = Some(
                (self.page_items.as_ref().unwrap().len() + self.per_page.unwrap() - 1)
                    / self.per_page.unwrap(),
            );
        }
    }

    pub fn paginate(&mut self) {
        if self.page_container.is_none() {
            return;
        };
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

            self.update_tag("title", page_number);
            self.update_meta_tag("og:title", page_number);
            self.update_meta_tag("twitter:title", page_number);

            self.update_controls_for_page(page_number, self.page_count.unwrap());

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

            self.reattach_controls();
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

    fn read_meta_format(&mut self) {
        let mut attributes = self
            .page_container
            .as_ref()
            .unwrap()
            .as_node()
            .as_element()
            .unwrap()
            .attributes
            .borrow_mut();

        if let Some(format) = attributes.get("data-pagebreak-meta") {
            self.page_meta_format = format.to_string();
            attributes.remove("data-pagebreak-meta");
        }
    }

    fn find_pagebreak_node(&mut self) {
        self.page_container = self.document.select("[data-pagebreak]").unwrap().next();
    }

    fn read_pagebreak_node(&mut self) {
        let mut pagination_attributes = self
            .page_container
            .as_ref()
            .unwrap()
            .as_node()
            .as_element()
            .unwrap()
            .attributes
            .borrow_mut();
        self.page_url_format = pagination_attributes
            .get("data-pagebreak-url")
            .unwrap_or("./page/:num/")
            .to_string();
        pagination_attributes.remove("data-pagebreak-url");
        self.per_page = Some(
            pagination_attributes
                .get("data-pagebreak")
                .unwrap_or("2")
                .parse::<usize>()
                .unwrap(),
        );
        pagination_attributes.remove("data-pagebreak");
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

    fn find_pagebreak_controls(&mut self) {
        let mut controls = vec![];
        self.document
            .select("[data-pagebreak-control]")
            .unwrap()
            .for_each(|element| {
                let element_node = element.as_node();
                let mut element_attributes =
                    element_node.as_element().unwrap().attributes.borrow_mut();
                let element_type = match element_attributes
                    .get("data-pagebreak-control")
                    .unwrap_or("none")
                {
                    "next" => PagebreakControlType::Next,
                    "prev" => PagebreakControlType::Previous,
                    "!next" => PagebreakControlType::NoNext,
                    "!prev" => PagebreakControlType::NoPrevious,
                    "current" => PagebreakControlType::Current,
                    "total" => PagebreakControlType::Total,
                    _ => PagebreakControlType::None,
                };
                element_attributes.remove("data-pagebreak-control");
                controls.push(PagebreakControl::new(
                    element_node.clone(),
                    element_type,
                    element_node.parent(),
                    element_node.previous_sibling(),
                ));
            });
        self.controls = Some(controls);
    }

    fn update_tag(&mut self, name: &str, page_index: usize) {
        if page_index == 0 {
            return;
        }

        if let Ok(elements) = self.document.select(name) {
            elements.for_each(|element| {
                let resolved_content = self
                    .page_meta_format
                    .replace(":num", &format!("{}", page_index + 1))
                    .replace(":content", &element.text_contents());

                element
                    .as_node()
                    .children()
                    .for_each(|child| child.detach());
                element
                    .as_node()
                    .append(NodeRef::new_text(&resolved_content))
            });
        }
    }

    fn update_meta_tag(&mut self, name: &str, page_index: usize) {
        if page_index == 0 {
            return;
        }

        if let Ok(elements) = self
            .document
            .select(&format!("meta[property=\"{}\"]", name))
        {
            elements.for_each(|meta_tag| {
                let mut meta_attributes = meta_tag
                    .as_node()
                    .as_element()
                    .unwrap()
                    .attributes
                    .borrow_mut();

                if let Some(content) = meta_attributes.get("content") {
                    let resolved_content = self
                        .page_meta_format
                        .replace(":num", &format!("{}", page_index + 1))
                        .replace(":content", content);
                    meta_attributes.remove("content");
                    meta_attributes.insert("content", resolved_content);
                }
            });
        }
    }

    fn update_controls_for_page(&mut self, page_index: usize, total_pages: usize) {
        self.update_control_text(PagebreakControlType::Current, (page_index + 1).to_string());
        self.update_control_text(PagebreakControlType::Total, total_pages.to_string());

        if page_index == 0 {
            self.detach_control(PagebreakControlType::Previous);
        } else {
            let relative_href = self.relative_path_between_pages(page_index, page_index - 1);
            self.update_control_href(PagebreakControlType::Previous, relative_href);
            self.detach_control(PagebreakControlType::NoPrevious);
        }

        if page_index == self.page_count.unwrap() - 1 {
            self.detach_control(PagebreakControlType::Next);
        } else {
            let relative_href = self.relative_path_between_pages(page_index, page_index + 1);
            self.update_control_href(PagebreakControlType::Next, relative_href);
            self.detach_control(PagebreakControlType::NoNext);
        }
    }

    fn detach_control(&mut self, control_type: PagebreakControlType) {
        self.controls
            .as_ref()
            .unwrap()
            .iter()
            .filter(|control| control.control_type == control_type)
            .for_each(|control| {
                control.element.detach();
            });
    }

    fn reattach_controls(&mut self) {
        self.controls.as_ref().unwrap().iter().for_each(|control| {
            if control.previous_sibling.is_some() {
                control
                    .previous_sibling
                    .as_ref()
                    .unwrap()
                    .insert_after(control.element.clone())
            } else if control.parent.is_some() {
                control
                    .parent
                    .as_ref()
                    .unwrap()
                    .prepend(control.element.clone());
            }
        });
    }

    fn update_control_href(&mut self, control_type: PagebreakControlType, new_href: String) {
        self.controls
            .as_ref()
            .unwrap()
            .iter()
            .filter(|control| control.control_type == control_type)
            .for_each(|control| {
                let mut attributes = control
                    .element
                    .as_element()
                    .unwrap()
                    .attributes
                    .borrow_mut();
                attributes.remove("href");
                attributes.insert("href", new_href.clone());
            });
    }

    fn update_control_text(&mut self, control_type: PagebreakControlType, new_text: String) {
        self.controls
            .as_ref()
            .unwrap()
            .iter()
            .filter(|control| control.control_type == control_type)
            .for_each(|control| {
                let node_ref = &control.element;
                node_ref.children().for_each(|child| child.detach());
                node_ref.append(NodeRef::new_text(&new_text));
            });
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

    fn relative_path_between_pages(&mut self, from: usize, to: usize) -> String {
        let from_path = self.get_file_url(from).unwrap();
        let to_path = self.get_file_url(to).unwrap();
        let mut relative_path =
            pathdiff::diff_paths(to_path.parent().unwrap(), from_path.parent().unwrap()).unwrap();
        if let Component::Normal(_) = relative_path.components().next().unwrap() {
            relative_path = PathBuf::from("./").join(relative_path);
        }
        format!("{}/", relative_path.to_str().unwrap())
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

#[cfg(test)]
mod tests {
    use kuchiki::traits::TendrilSink;

    use super::*;

    fn new_state() -> PagebreakState {
        PagebreakState::new(
            kuchiki::parse_html().one(""),
            PathBuf::from("index.html"),
            PathBuf::from("output"),
        )
    }

    #[test]
    fn test_get_file_url() {
        let mut state = new_state();
        assert_eq!(PathBuf::from("index.html"), state.get_file_url(0).unwrap(),);
        assert_eq!(
            PathBuf::from("page/2/index.html"),
            state.get_file_url(1).unwrap(),
        );

        state.file_path = PathBuf::from("about/index.html");
        assert_eq!(
            PathBuf::from("about/index.html"),
            state.get_file_url(0).unwrap(),
        );
        assert_eq!(
            PathBuf::from("about/page/2/index.html"),
            state.get_file_url(1).unwrap(),
        );

        state.file_path = PathBuf::from("a/b/c/index.html");
        state.page_url_format = "../../page/:num/".to_string();
        assert_eq!(
            PathBuf::from("a/page/2/index.html"),
            state.get_file_url(1).unwrap(),
        );
    }

    #[test]
    fn test_bad_file_url() {
        let mut state = new_state();
        state.page_url_format = "../page/:num/".to_string();
        assert_eq!(
            errors::PageErrorCode::ParentDir,
            state.get_file_url(1).unwrap_err().code,
        );
    }

    #[test]
    fn test_relative_pagination_urls() {
        let mut state = new_state();
        assert_eq!(
            "./page/2/".to_string(),
            state.relative_path_between_pages(0, 1)
        );
        assert_eq!(
            "../../".to_string(),
            state.relative_path_between_pages(1, 0)
        );
        assert_eq!("../3/".to_string(), state.relative_path_between_pages(1, 2));

        state.file_path = PathBuf::from("file/main/index.html");
        state.page_url_format = "../pages/:num/page/".to_string();
        assert_eq!(
            "../pages/2/page/".to_string(),
            state.relative_path_between_pages(0, 1)
        );
        assert_eq!(
            "../../../main/".to_string(),
            state.relative_path_between_pages(1, 0)
        );
        assert_eq!(
            "../../3/page/".to_string(),
            state.relative_path_between_pages(1, 2)
        );
    }
}
