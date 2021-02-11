use std::io::Error;
use std::path::PathBuf;

pub fn resolve(cwd: &PathBuf, target: &str) -> Result<PathBuf, Error> {
    let joined = cwd.join(PathBuf::from(target));
    println!(
        "looking for {:?}, {:?}",
        joined.clone().display(),
        joined.canonicalize()
    );
    Ok(PathBuf::from("./"))
}
