use std::cell::RefCell;

use cucumber::gherkin::Step;
use cucumber::{given, then, when};
use kuchiki::iter::{Descendants, Elements, Select};
use kuchiki::traits::TendrilSink;
use kuchiki::{Attributes, ElementData, NodeDataRef, NodeRef};

use crate::{PagebreakOptions, PagebreakWorld};

// GIVENS

#[given(regex = "^I have an? (?:\"|')(.*)(?:\"|') file with the content:$")]
fn new_file(world: &mut PagebreakWorld, step: &Step, filename: String) {
    match &step.docstring {
        Some(contents) => {
            world.write_file(&filename, contents);
        }
        None => panic!("`{}` step expected a docstring", step.value),
    }
}

#[given(regex = "^I have an? (?:\"|')(.*)(?:\"|') file with the body:$")]
fn new_templated_file(world: &mut PagebreakWorld, step: &Step, filename: String) {
    match &step.docstring {
        Some(contents) => {
            world.write_file(&filename, &template_file(contents));
        }
        None => panic!("`{}` step expected a docstring", step.value),
    }
}

// WHENS

#[when("I run Pagebreak")]
fn run_pagebreak(world: &mut PagebreakWorld) {
    let options = PagebreakOptions::default();
    world.run_pagebreak(options);
}

#[when("I run Pagebreak with options:")]
fn run_pagebreak_with_options(world: &mut PagebreakWorld, step: &Step) {
    match &step.table {
        Some(table) => {
            let options = PagebreakOptions::from(table);
            world.run_pagebreak(options);
        }
        None => panic!("`{}` step expected a docstring", step.value),
    }
}

// THENS

#[then(regex = "^I should see (?:\"|')(.*)(?:\"|') in (?:\"|')(.*)(?:\"|')$")]
fn file_does_contain(world: &mut PagebreakWorld, expected: String, filename: String) {
    assert!(world.check_file_exists(&filename));
    assert!(world.read_file(&filename).contains(&expected));
}

#[then(regex = "^I should not see (?:\"|')(.*)(?:\"|') in (?:\"|')(.*)(?:\"|')$")]
fn file_does_not_contain(world: &mut PagebreakWorld, expected: String, filename: String) {
    assert!(world.check_file_exists(&filename));
    assert!(!world.read_file(&filename).contains(&expected));
}

#[then(regex = "^I should see the file (?:\"|')(.*)(?:\"|')$")]
fn file_does_exist(world: &mut PagebreakWorld, filename: String) {
    assert!(world.check_file_exists(&filename));
}

#[then(regex = "^I should not see the file (?:\"|')(.*)(?:\"|')$")]
fn file_does_not_exist(world: &mut PagebreakWorld, filename: String) {
    assert!(!world.check_file_exists(&filename));
}

#[then(regex = "^I should see a selector (?:\"|')(.*)(?:\"|') in (?:\"|')(\\S*)(?:\"|')$")]
fn selector_exists(world: &mut PagebreakWorld, selector: String, filename: String) {
    assert!(world.check_file_exists(&filename));
    let parsed_file = parse_file(&world.read_file(&filename));
    assert!(select_nodes(&parsed_file, &selector).next().is_some());
}

#[then(
    regex = "^I should see a selector (?:\"|')(.*)(?:\"|') in (?:\"|')(.*)(?:\"|') with the attributes:$"
)]
fn selector_attributes(
    world: &mut PagebreakWorld,
    step: &Step,
    selector: String,
    filename: String,
) {
    assert!(world.check_file_exists(&filename));
    let parsed_file = parse_file(&world.read_file(&filename));

    'nodes: for node in select_nodes(&parsed_file, &selector) {
        let atts = node_attributes(&node);
        let attributes = atts.borrow_mut();
        for row in &step
            .table
            .as_ref()
            .expect("This step requires a table")
            .rows
        {
            match attributes.get(AsRef::<str>::as_ref(&row[0])) {
                Some(value) => {
                    if value != row[1] {
                        continue 'nodes;
                    }
                }
                None => continue 'nodes,
            }
        }
        return;
    }
    panic!("No nodes found that match all provided attributes");
}

#[then(
    regex = "^I should see a selector (?:\"|')(.*)(?:\"|') in (?:\"|')(.*)(?:\"|') with the content (?:\"|')(.*)(?:\"|')$"
)]
fn selector_content(
    world: &mut PagebreakWorld,
    selector: String,
    filename: String,
    expected: String,
) {
    assert!(world.check_file_exists(&filename));
    let parsed_file = parse_file(&world.read_file(&filename));

    for node in select_nodes(&parsed_file, &selector) {
        let content = node.text_contents();
        if content == expected {
            return;
        }
    }
    panic!("No nodes found that contained the content");
}

// HELPERS

fn parse_file(html: &str) -> NodeRef {
    kuchiki::parse_html().one(html)
}

fn select_nodes(parsed_file: &NodeRef, selector: &str) -> Select<Elements<Descendants>> {
    parsed_file
        .select(selector)
        .expect("Valid selector was given")
}

fn node_attributes(node: &NodeDataRef<ElementData>) -> RefCell<Attributes> {
    node.as_node()
        .as_element()
        .expect("Given selector was an element")
        .attributes
        .clone()
}

fn template_file(body_contents: &str) -> String {
    format!(
        r#"
<!DOCTYPE html>
<html>
    <head>
    </head>
    <body>
        {}
    </body>
</html>
"#,
        body_contents
    )
}
