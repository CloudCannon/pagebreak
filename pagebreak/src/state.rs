use crate::errors;
use kuchiki::{ElementData, NodeDataRef, NodeRef};
use lexiclean::Lexiclean;
use std::cell::RefCell;
use std::path::Component;
use std::rc::Rc;
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

enum PagebreakChange {
    Content(NodeRef, String),
    Attribute(NodeRef, String, String),
}

#[derive(Debug, PartialEq)]
enum PagebreakElementType {
    Next,
    Previous,
    NoNext,
    NoPrevious,
    Current,
    Total,
    None,
}
struct PagebreakElement {
    element: NodeRef,
    parent: Option<NodeRef>,
    previous_sibling: Option<NodeRef>,
    element_type: PagebreakElementType,
}

impl PagebreakElement {
    pub fn new(
        element: NodeRef,
        element_type: PagebreakElementType,
        parent: Option<NodeRef>,
        previous_sibling: Option<NodeRef>,
    ) -> Self {
        PagebreakElement {
            element,
            parent,
            previous_sibling,
            element_type,
        }
    }
}

pub struct PagebreakState {
    pub document: NodeRef,
    file_path: PathBuf,
    output_path: PathBuf,
    page_container: Option<NodeDataRef<ElementData>>,
    page_items: Option<Rc<RefCell<Vec<PagebreakNode>>>>,
    page_count: Option<usize>,
    per_page: Option<usize>,
    page_url_format: String,
    page_meta_format: String,
    dom_indentation: String,
    pagebreak_elements: Option<Vec<PagebreakElement>>,
    changes: Vec<PagebreakChange>,
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
            pagebreak_elements: None,
            changes: Vec::default(),
        }
    }

    pub fn hydrate(&mut self) {
        self.find_pagebreak_node();
        if self.page_container.is_some() {
            self.read_meta_format();
            self.read_pagebreak_node();
            self.find_pagination_children();
            self.find_pagebreak_elements();
            self.page_count = Some(
                (self.page_items.as_ref().unwrap().borrow().len() + self.per_page.unwrap() - 1)
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
            let remaining_items = self.page_items.as_ref().unwrap().clone();
            let mut remaining_items = remaining_items.borrow_mut();
            let max_count = self.per_page.unwrap().min(remaining_items.len());

            remaining_items.drain(0..max_count).for_each(|element| {
                self.indent_for_next_element();
                self.reattach_child(element);
            });

            self.indent_for_next_element();

            let meta_format = &self.page_meta_format.clone();
            self.update_tag_content(meta_format, "title", page_number, |_| true);
            self.update_tag_attribute(
                meta_format,
                "[property=\"og:title\"]",
                "content",
                page_number,
                |_| true,
            );
            self.update_tag_attribute(
                meta_format,
                "[property=\"twitter:title\"]",
                "content",
                page_number,
                |_| true,
            );

            self.update_tag_attribute(
                ":rel-from:content",
                "[href]",
                "href",
                page_number,
                |element| {
                    let attributes = element.as_node().as_element().unwrap().attributes.borrow();
                    let url = attributes.get("href").unwrap();
                    !url.starts_with("http://")
                        && !url.starts_with("https://")
                        && !url.starts_with('/')
                },
            );

            self.update_tag_attribute(
                ":content:rel-to",
                "[rel=\"canonical\"]",
                "href",
                page_number,
                |_| true,
            );

            self.update_tag_attribute(
                ":content:rel-to",
                "[property=\"og:url\"]",
                "content",
                page_number,
                |_| true,
            );

            self.update_elements_for_page(page_number, self.page_count.unwrap());

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

            self.reattach_elements();
            self.revert_changes();
        }
    }

    pub fn revert_changes(&mut self) {
        for change in &self.changes {
            match change {
                PagebreakChange::Content(node, original_content) => {
                    node.children().for_each(|child| child.detach());
                    node.append(NodeRef::new_text(original_content))
                }
                PagebreakChange::Attribute(node, attribute, value) => {
                    let mut attributes = node
                        .as_element()
                        .expect("Editted node should be an element")
                        .attributes
                        .borrow_mut();

                    attributes.remove(attribute.as_str());
                    attributes.insert(attribute.as_str(), value.clone());
                }
            }
        }
        self.changes.clear();
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

        self.page_items = Some(Rc::new(RefCell::new(children)));
    }

    fn find_pagebreak_elements(&mut self) {
        let mut elements = vec![];
        self.document
            .select("[data-pagebreak-control], [data-pagebreak-label]")
            .unwrap()
            .for_each(|element| {
                let element_node = element.as_node();
                let mut element_attributes =
                    element_node.as_element().unwrap().attributes.borrow_mut();
                let (attribute, value) =
                    if let Some(control) = element_attributes.get("data-pagebreak-control") {
                        ("data-pagebreak-control", control)
                    } else if let Some(label) = element_attributes.get("data-pagebreak-label") {
                        ("data-pagebreak-label", label)
                    } else {
                        unreachable!("Couldn't get attribute")
                    };
                let element_type = match (attribute, value) {
                    ("data-pagebreak-control", "next") => PagebreakElementType::Next,
                    ("data-pagebreak-control", "prev") => PagebreakElementType::Previous,
                    ("data-pagebreak-control", "!next") => PagebreakElementType::NoNext,
                    ("data-pagebreak-control", "!prev") => PagebreakElementType::NoPrevious,
                    ("data-pagebreak-label", "current") => PagebreakElementType::Current,
                    ("data-pagebreak-label", "total") => PagebreakElementType::Total,
                    _ => PagebreakElementType::None,
                };
                element_attributes.remove(attribute);
                elements.push(PagebreakElement::new(
                    element_node.clone(),
                    element_type,
                    element_node.parent(),
                    element_node.previous_sibling(),
                ));
            });
        self.pagebreak_elements = Some(elements);
    }

    fn resolve_format(&mut self, format: &str, page_index: usize, content: &str) -> String {
        let path_to = self.relative_path_between_pages(0, page_index);
        let path_from = self.relative_path_between_pages(page_index, 0);

        format
            .replace(":num", &format!("{}", page_index + 1))
            .replace(":content", content)
            .replace(":rel-from", &path_from)
            .replace(":rel-to", &path_to)
    }

    fn update_tag_content(
        &mut self,
        format: &str,
        selector: &str,
        page_index: usize,
        filter: fn(&NodeDataRef<ElementData>) -> bool,
    ) {
        if page_index == 0 {
            return;
        }

        if let Ok(elements) = self.document.select(selector) {
            elements.filter(filter).for_each(|element| {
                self.changes.push(PagebreakChange::Content(
                    element.as_node().clone(),
                    element.text_contents(),
                ));
                let resolved_content =
                    self.resolve_format(format, page_index, &element.text_contents());

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

    fn update_tag_attribute(
        &mut self,
        format: &str,
        selector: &str,
        attribute: &str,
        page_index: usize,
        filter: fn(&NodeDataRef<ElementData>) -> bool,
    ) {
        if page_index == 0 {
            return;
        }

        if let Ok(elements) = self.document.select(selector) {
            elements.filter(filter).for_each(|element| {
                let mut attributes = element
                    .as_node()
                    .as_element()
                    .unwrap()
                    .attributes
                    .borrow_mut();

                if let Some(content) = attributes.get(attribute) {
                    self.changes.push(PagebreakChange::Attribute(
                        element.as_node().clone(),
                        attribute.to_string(),
                        String::from(content),
                    ));
                    let resolved_content = self.resolve_format(format, page_index, content);
                    attributes.remove(attribute);
                    attributes.insert(attribute, resolved_content);
                }
            });
        }
    }

    fn update_elements_for_page(&mut self, page_index: usize, total_pages: usize) {
        self.update_element_text(PagebreakElementType::Current, (page_index + 1).to_string());
        self.update_element_text(PagebreakElementType::Total, total_pages.to_string());

        if page_index == 0 {
            self.detach_element(PagebreakElementType::Previous);
        } else {
            let relative_href = self.relative_path_between_pages(page_index, page_index - 1);
            self.update_element_href(PagebreakElementType::Previous, relative_href);
            self.detach_element(PagebreakElementType::NoPrevious);
        }

        if page_index == self.page_count.unwrap() - 1 {
            self.detach_element(PagebreakElementType::Next);
        } else {
            let relative_href = self.relative_path_between_pages(page_index, page_index + 1);
            self.update_element_href(PagebreakElementType::Next, relative_href);
            self.detach_element(PagebreakElementType::NoNext);
        }
    }

    fn detach_element(&mut self, element_type: PagebreakElementType) {
        self.pagebreak_elements
            .as_ref()
            .unwrap()
            .iter()
            .filter(|element| element.element_type == element_type)
            .for_each(|element| {
                element.element.detach();
            });
    }

    fn reattach_elements(&mut self) {
        self.pagebreak_elements
            .as_ref()
            .unwrap()
            .iter()
            .for_each(|element| {
                if element.previous_sibling.is_some() {
                    element
                        .previous_sibling
                        .as_ref()
                        .unwrap()
                        .insert_after(element.element.clone())
                } else if element.parent.is_some() {
                    element
                        .parent
                        .as_ref()
                        .unwrap()
                        .prepend(element.element.clone());
                }
            });
    }

    fn update_element_href(&mut self, element_type: PagebreakElementType, new_href: String) {
        self.pagebreak_elements
            .as_ref()
            .unwrap()
            .iter()
            .filter(|element| element.element_type == element_type)
            .for_each(|element| {
                let mut attributes = element
                    .element
                    .as_element()
                    .unwrap()
                    .attributes
                    .borrow_mut();
                attributes.remove("href");
                attributes.insert("href", new_href.clone());
            });
    }

    fn update_element_text(&mut self, element_type: PagebreakElementType, new_text: String) {
        self.pagebreak_elements
            .as_ref()
            .unwrap()
            .iter()
            .filter(|element| element.element_type == element_type)
            .for_each(|element| {
                let node_ref = &element.element;
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
                let file_stem = self.file_path.file_stem().unwrap().to_str().unwrap();
                let file_path = if !file_stem.eq("index") {
                    PathBuf::from(file_stem).join(file_url).join("index.html")
                } else {
                    PathBuf::from(file_url).join("index.html")
                };
                let cleaned_path = self.file_path.parent().unwrap().join(file_path).lexiclean();
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
        // relative_path.strip_prefix(base)
        if let Component::CurDir = relative_path.components().next().unwrap() {
            relative_path = relative_path
                .strip_prefix(".")
                .expect("Prefix was checked")
                .to_path_buf();
        }
        format!(
            "{}/",
            relative_path
                .to_str()
                .expect("valid characters")
                .replace("\\", "/")
        )
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
            self.page_items.as_ref().unwrap().borrow().len(),
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
        assert_eq!("page/2/", state.relative_path_between_pages(0, 1));
        assert_eq!("../../", state.relative_path_between_pages(1, 0));
        assert_eq!("../3/", state.relative_path_between_pages(1, 2));

        state.file_path = PathBuf::from("file/main/index.html");
        state.page_url_format = "../pages/:num/page/".to_string();
        assert_eq!("../pages/2/page/", state.relative_path_between_pages(0, 1));
        assert_eq!("../../../main/", state.relative_path_between_pages(1, 0));
        assert_eq!("../../3/page/", state.relative_path_between_pages(1, 2));

        state.file_path = PathBuf::from("index.html");
        state.page_url_format = "./:num/".to_string();
        assert_eq!("2/", state.relative_path_between_pages(0, 1));
        assert_eq!("../", state.relative_path_between_pages(1, 0));
    }
}
