use cucumber_rust::{Steps};
use tempfile::tempdir;
use std::{collections::HashMap, fs, path::PathBuf};
use std::io::{Write, Read};
use pagebreak::PagebreakRunner;
use crate::PagebreakWorld;

pub fn steps() -> Steps<crate::PagebreakWorld> {
    let mut steps: Steps<crate::PagebreakWorld> = Steps::new();

    steps
        .given_regex(r#"^I have an? "(.*)" file with content:$"#, |mut world, ctx| {
            if world.tmp_dir.is_none() {
                world.tmp_dir = Some(tempdir().unwrap());
            }
            let filename = &ctx.matches[1];
            let contents = ctx.step.docstring.as_ref().unwrap();
            let path = world.tmp_dir.as_ref().unwrap().path().join(filename);

            fs::create_dir_all(path.parent().unwrap()).unwrap();
            let mut file = std::fs::File::create(&path).unwrap();
            file.write_all(contents.as_bytes()).unwrap();

            world
        })

        .when_regex(r#"^I run Pagebreak with options:$"#, |world, ctx| {
            let table = ctx.step.table.as_ref().unwrap();

            let mut options: HashMap<&str, &str> =
                [("source", "source"),
                ("output", "output")]
                .iter().cloned().collect();
            for col in 0..table.row_width() {
                options.insert(table.rows[0][col].as_str(), table.rows[1][col].as_str());
            }

            let mut runner = PagebreakRunner::new(
                PathBuf::from(world.tmp_dir.as_ref().unwrap().path()),
                PathBuf::from(options.get("source").unwrap()),
                PathBuf::from(options.get("output").unwrap()),
            );
            runner.run();

            world
        })

        .then_regex(r#"I should see "(.*)" in "(.*)"$"#, |world, ctx| {
            let contents = read_file(&world,&ctx.matches[2]);
            assert!(contents.contains(&ctx.matches[1]));

            world
        })

        .then_regex(r#"I should not see "(.*)" in "(.*)"$"#, |world, ctx| {
            let contents = read_file(&world,&ctx.matches[2]);
            assert!(!contents.contains(&ctx.matches[1]));

            world
        })
        
        .then_regex(r#"I should see the file "(.*)"$"#, |world, ctx| {
            assert!(check_file_exists(&world, &ctx.matches[1]));

            world
        })
        
        .then_regex(r#"I should not see the file "(.*)"$"#, |world, ctx| {
            assert!(!check_file_exists(&world, &ctx.matches[1]));

            world
        });
    steps
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