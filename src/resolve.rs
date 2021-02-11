use std::io::{Error, ErrorKind};
use std::path::{PathBuf, Component};



pub fn resolve(subject_file: &PathBuf, target: &str) -> Result<PathBuf, Error> {
    let subject_dir = subject_file.parent().expect("always has a parent");
    let joined = subject_dir.join(target);
    let joined_real = realpath(&joined);
    let resolved = joined_real.canonicalize();
    match resolved {
        Ok(pb) => {
            expand_path_with_index_or_extension(&pb).ok_or(Error::from(ErrorKind::NotFound))
        },
        Err(e) => match e.kind() {
            ErrorKind::NotFound => {
                expand_path_with_index_or_extension(&joined_real).ok_or(Error::from(ErrorKind::NotFound))
            }
            _ => Err(e)
        }
    }
}

fn realpath(input: &PathBuf) -> PathBuf {
    let mut realpath = PathBuf::new();
    for c in input.components() {
        match c {
            Component::RootDir => realpath.push("/"),
            Component::ParentDir => realpath = realpath.parent().unwrap().to_path_buf(),
            Component::Normal(c) => realpath.push(c),
            _ => (),
        }
    }

    realpath
}

fn expand_path_with_index_or_extension(pb: &PathBuf) -> Option<PathBuf> {
    if pb.is_file() {
        return Some(pb.clone());
    }
    if pb.is_dir() {
        let maybe = pb.join("index.ts");
        if maybe.is_file() {
            log::debug!("added `index.ts` to {}", pb.display());
            return Some(maybe);
        }
        let maybe = pb.join("index.tsx");
        if maybe.is_file() {
            log::debug!("added `index.tsx` to {}", pb.display());
            return Some(maybe);
        }
    }

    let maybe_ext = pb.with_extension("tsx");
    if maybe_ext.is_file() {
        log::debug!("added ext `tsx` to {}", pb.display());
        return Some(maybe_ext);
    }

    let maybe_ext2 = pb.with_extension("ts");
    if maybe_ext2.is_file() {
        log::debug!("added ext `ts` to {}", pb.display());
        return Some(maybe_ext2);
    }

    None
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::env::current_dir;

    #[test]
    fn test_resolve_dir() -> Result<(), std::io::Error> {
        let cwd = current_dir()?.join("fixtures/ts");
        let subject = cwd.join("src").join("index.ts");
        let target = "../components";
        let resolved = resolve(&subject, target)?;
        assert!(resolved.exists());
        assert_eq!(cwd.join("components/index.tsx"), resolved);
        Ok(())
    }

    #[test]
    fn test_resolve_file_without_ext() -> Result<(), std::io::Error> {
        let cwd = current_dir()?.join("fixtures/ts");
        let subject = cwd.join("src").join("index.ts");
        let target = "../components/button";
        let resolved = resolve(&subject, target)?;
        assert!(resolved.exists());
        assert_eq!(cwd.join("components/button.tsx"), resolved);
        Ok(())
    }

    #[test]
    fn test_realpath() {
        let input = "app-src/index.tsx";
        let p = realpath(&PathBuf::from(input));
        assert_eq!(p, PathBuf::from(input))
    }
}