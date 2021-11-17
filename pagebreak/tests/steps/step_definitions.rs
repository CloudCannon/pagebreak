use cucumber::gherkin::Step;
use cucumber::{given, then, when};
use std::io::{Read, Write};
use std::{collections::HashMap, fs, path::PathBuf};

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
