use crate::ts_config::UserTsConfig;

use std::io::{Error, ErrorKind};
use std::path::{Component, PathBuf};

pub fn resolve(
    subject_file: impl Into<PathBuf>,
    target_import: impl Into<PathBuf>,
) -> Result<PathBuf, Error> {
    let target_import = target_import.into();
    let subject_file = subject_file.into();
    let subject_dir: PathBuf = if subject_file.extension().is_some() {
        subject_file.parent().expect("always has a parent").into()
    } else {
        subject_file
    };
    log::trace!(
        "subject_dir=`{}`, target=`{}`",
        subject_dir.display(),
        target_import.display()
    );
    let joined = subject_dir.join(target_import);
    log::trace!("joined=`{}`", joined.display());
    let joined_real = realpath(&joined);
    log::trace!("joined_real=`{}`", joined_real.display());
    let resolved = joined_real.canonicalize();
    match resolved {
        Ok(pb) => expand_path_with_index_or_extension(&pb).ok_or(Error::from(ErrorKind::NotFound)),
        Err(e) => match e.kind() {
            ErrorKind::NotFound => expand_path_with_index_or_extension(&joined_real)
                .ok_or(Error::from(ErrorKind::NotFound)),
            _ => Err(e),
        },
    }
}

pub fn apply_alias(config: &UserTsConfig, target_pb: &PathBuf) -> Option<PathBuf> {
    let is_absolute = target_pb.is_absolute();

    if is_absolute {
        return Some(target_pb.clone());
    }

    let first = target_pb.components().nth(0)?;
    let paths = config.compiler_options.as_ref()?.paths.as_ref()?;
    if let Component::Normal(_) = first {
        paths.keys().find_map(|key| {
            let pb = PathBuf::from(key);
            let (index, before) = before_star(&pb)?;
            log::trace!(
                "key={}, before={}, index={}, target={}",
                key,
                before.display(),
                index,
                target_pb.display()
            );
            let split_target = target_pb.components().take(index).collect::<PathBuf>();
            if split_target != before {
                return None;
            }
            let hs = paths.get(key)?;
            hs.iter().find_map(|item| {
                let (index, before) = before_star(item)?;
                let target_with_alias_prefix =
                    target_pb.components().skip(index - 1).collect::<PathBuf>();
                Some(before.join(target_with_alias_prefix))
            })
        })
    } else {
        Some(target_pb.clone())
    }
}

fn before_star(pb: impl Into<PathBuf>) -> Option<(usize, PathBuf)> {
    let pb = pb.into();
    pb.components()
        .position(|c| match c {
            Component::Normal(str) => str == "*",
            _ => false,
        })
        .map(|index| (index, pb.components().take(index).collect::<PathBuf>()))
}

pub fn resolve_target(
    cwd: &PathBuf,
    subject_file: &PathBuf,
    target_import: &PathBuf,
    ts_config: &UserTsConfig,
) -> Result<PathBuf, std::io::Error> {
    let alias_result = apply_alias(ts_config, &target_import);
    if let Some(alias) = alias_result {
        // do aliases always resolve from the cwd?
        resolve(&cwd, alias)
    } else {
        resolve(&subject_file, &target_import)
    }
}

#[cfg(test)]
mod resolve_tests {
    use super::*;
    use std::env::current_dir;

    #[test]
    fn test_resolve_alias() -> Result<(), std::io::Error> {
        let cwd = current_dir()?.join("fixtures/ts");
        let _subject = cwd.join("src").join("index.ts");
        let target = PathBuf::from("app-src/01/02/03/index.ts");
        let input = r#"
            {
              "compilerOptions": {
                "baseUrl": ".",
                "paths": {
                  "~/src/*": ["./src/scripts/*"],
                  "app-src/*": ["./__other-src/*"],
                  "ui/components/*": ["./components/*"],
                  "not-supported": ["./something/*"]
                }
              }
            }
        "#;
        let ts_config: UserTsConfig = serde_json::from_str(input).unwrap();
        let alias = apply_alias(&ts_config, &target);
        assert_eq!(
            alias,
            Some(PathBuf::from("./__other-src/01/02/03/index.ts"))
        );
        Ok(())
    }

    #[test]
    fn test_resolve_alias_index() -> Result<(), std::io::Error> {
        let input = r#"
            {
              "compilerOptions": {
                "baseUrl": ".",
                "paths": {
                  "app-src/*": ["./app-src/*"]
                }
              }
            }
        "#;

        let cwd = current_dir()?.join("fixtures/ts");
        let subject_file = cwd.join("src").join("index.ts");
        let target = PathBuf::from("app-src");
        let ts_config: UserTsConfig = serde_json::from_str(input).unwrap();
        let resolved = resolve_target(&cwd, &subject_file, &target, &ts_config)?;

        let expected = current_dir()?.join("fixtures/ts/app-src/index.tsx");
        assert_eq!(resolved, expected);
        Ok(())
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

    #[test]
    fn test_resolve_relative() -> Result<(), std::io::Error> {
        let cwd = current_dir()?.join("fixtures/ts");
        let subject = cwd.join("src/index.ts");
        let target = "../app-src/utils";
        let resolved = resolve(&subject, target)?;
        assert!(resolved.exists());
        assert_eq!(cwd.join("app-src/utils.ts"), resolved);
        Ok(())
    }
}
