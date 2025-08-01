use std::{
    env,
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use build_data::get_git_dirty;

/// Outputs a readable version number such as
/// 0.4.0 (if git commit is clean)
/// 0.4.0-SNAPSHOT (if git commit is dirty, should not happen in CI/CD builds)
fn version() -> String {
    let version = String::from(env!("CARGO_PKG_VERSION"));
    match get_git_dirty().unwrap() {
        false => version,
        true => {
            format!("{}-SNAPSHOT", version)
        }
    }
}

fn build_cqlmap() {
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("replace_map.rs");
    let mut file = BufWriter::new(File::create(path).unwrap());

    write!(&mut file, r#"
        static REPLACE_MAP: once_cell::sync::Lazy<HashMap<&'static str, &'static str>> = once_cell::sync::Lazy::new(|| {{
        let mut map = HashMap::new();
    "#).unwrap();

    for cqlfile in std::fs::read_dir(Path::new("resources/cql")).unwrap() {
        let cqlfile = cqlfile.unwrap();
        let cqlfilename = cqlfile.file_name().to_str().unwrap().to_owned();
        let cqlcontent = std::fs::read_to_string(cqlfile.path()).unwrap();
        write!(
            &mut file,
            r####"
            map.insert(r###"{cqlfilename}"###, r###"{cqlcontent}"###);
        "####
        )
        .unwrap();
    }

    writeln!(
        &mut file,
        "
        map
    }});"
    )
    .unwrap();
}

fn build_sqlmap() {
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("sql_replace_map.rs");
    let mut file = BufWriter::new(File::create(path).unwrap());

    write!(&mut file, r#"
        static SQL_REPLACE_MAP: once_cell::sync::Lazy<HashMap<&'static str, &'static str>> = once_cell::sync::Lazy::new(|| {{
        let mut map = HashMap::new();
    "#).unwrap();

    for sqlfile in std::fs::read_dir(Path::new("resources/sql")).unwrap() {
        let sqlfile = sqlfile.unwrap();
        let sqlfilename = sqlfile.file_name().to_str().unwrap().to_owned();
        let sqlcontent = std::fs::read_to_string(sqlfile.path()).unwrap();
        write!(
            &mut file,
            r####"
            map.insert(r###"{sqlfilename}"###, r###"{sqlcontent}"###);
        "####
        )
        .unwrap();
    }

    writeln!(
        &mut file,
        "
        map
    }});"
    )
    .unwrap();
}

fn main() {
    build_data::set_GIT_COMMIT_SHORT();
    build_data::set_GIT_DIRTY();
    build_data::set_BUILD_DATE();
    build_data::set_BUILD_TIME();
    // We must always run this build script as otherwise, we would cache old versions of CQL maps
    //build_data::no_debug_rebuilds();
    println!(
        "cargo:rustc-env=SAMPLY_USER_AGENT=Samply.Focus.{}/{}",
        env!("CARGO_PKG_NAME"),
        version()
    );

    build_cqlmap();
    build_sqlmap();
}
