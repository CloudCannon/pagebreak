use crate::PagebreakWorld;
use cucumber_rust::Steps;
use pagebreak::PagebreakRunner;
use std::io::{Read, Write};
use std::{collections::HashMap, fs, path::PathBuf};
use tempfile::tempdir;

pub fn steps() -> Steps<crate::PagebreakWorld> {
    let mut steps: Steps<crate::PagebreakWorld> = Steps::new();

    steps
        .given_regex(
            r#"^I have an? (?:"|')(.*)(?:"|') file with the content:$"#,
            |mut world, ctx| {
                if world.tmp_dir.is_none() {
                    world.tmp_dir = Some(tempdir().unwrap());
                }
                write_file(
                    &world,
                    &ctx.matches[1],
                    ctx.step.docstring.as_ref().unwrap(),
                );

                world
            },
        )
        .given_regex(
            r#"^I have an? (?:"|')(.*)(?:"|') file with the body:$"#,
            |mut world, ctx| {
                if world.tmp_dir.is_none() {
                    world.tmp_dir = Some(tempdir().unwrap());
                }
                let file_contents = template_file(ctx.step.docstring.as_ref().unwrap());
                write_file(&world, &ctx.matches[1], &file_contents);

                world
            },
        )
        .when_regex(r#"^I run Pagebreak$"#, |world, ctx| {
            let options = get_pagebreak_options(&ctx);
            run_pagebreak(&world, &options);

            world
        })
        .when_regex(r#"^I run Pagebreak with options:$"#, |world, ctx| {
            let options = get_pagebreak_options(&ctx);
            run_pagebreak(&world, &options);

            world
        })
        .then_regex(
            r#"I should see (?:"|')(.*)(?:"|') in (?:"|')(.*)(?:"|')$"#,
            |world, ctx| {
                let contents = read_file(&world, &ctx.matches[2]);
                assert!(dbg!(contents).contains(&ctx.matches[1]));

                world
            },
        )
        .then_regex(
            r#"I should not see (?:"|')(.*)(?:"|') in (?:"|')(.*)(?:"|')$"#,
            |world, ctx| {
                let contents = read_file(&world, &ctx.matches[2]);
                assert!(!contents.contains(&ctx.matches[1]));

                world
            },
        )
        .then_regex(
            r#"I should see the file (?:"|')(.*)(?:"|')$"#,
            |world, ctx| {
                assert!(check_file_exists(&world, &ctx.matches[1]));

                world
            },
        )
        .then_regex(
            r#"I should not see the file (?:"|')(.*)(?:"|')$"#,
            |world, ctx| {
                assert!(!check_file_exists(&world, &ctx.matches[1]));

                world
            },
        );
    steps
}

fn write_file(world: &PagebreakWorld, file: &String, contents: &String) {
    let file_path = PathBuf::from(file);
    let full_path = world.tmp_dir.as_ref().unwrap().path().join(file_path);
    fs::create_dir_all(full_path.parent().unwrap()).unwrap();
    let mut file = std::fs::File::create(&full_path).unwrap();
    file.write_all(contents.as_bytes()).unwrap();
}

fn read_file(world: &PagebreakWorld, file: &String) -> String {
    let file_path = PathBuf::from(file);
    let full_path = world.tmp_dir.as_ref().unwrap().path().join(file_path);
    let mut file = std::fs::File::open(&full_path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    contents
}

fn check_file_exists(world: &PagebreakWorld, file: &String) -> bool {
    let file_path = PathBuf::from(file);
    let full_path = world.tmp_dir.as_ref().unwrap().path().join(file_path);
    full_path.exists()
}

fn get_pagebreak_options(step: &cucumber_rust::StepContext) -> HashMap<&str, &str> {
    // Defaults:
    let mut options: HashMap<&str, &str> = [("source", "source"), ("output", "output")]
        .iter()
        .cloned()
        .collect();

    if step.step.table.as_ref().is_some() {
        let table = step.step.table.as_ref().unwrap();
        for col in 0..table.row_width() {
            options.insert(table.rows[0][col].as_str(), table.rows[1][col].as_str());
        }
    }
    options
}

fn run_pagebreak(world: &PagebreakWorld, options: &HashMap<&str, &str>) {
    let mut runner = PagebreakRunner::new(
        PathBuf::from(world.tmp_dir.as_ref().unwrap().path()),
        PathBuf::from(options.get("source").unwrap()),
        PathBuf::from(options.get("output").unwrap()),
    );
    runner.run();
}

fn template_file(body_contents: &String) -> String {
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
