mod resolve;

use std::env::current_dir;
use std::path::PathBuf;
use std::{path::Path, sync::Arc};
use structopt::StructOpt;
use swc::{self, config::Options};
use swc_common::{
    errors::{ColorConfig, Handler},
    SourceMap,
};
use swc_ecma_ast::{ImportSpecifier, ModuleDecl, ModuleItem, Program};
use swc_ecma_parser::token::Keyword::Default_;
use swc_ecma_parser::TsConfig;

/// A basic example
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    #[structopt(short, long)]
    cwd: Option<PathBuf>,

    /// Files to process
    #[structopt(name = "FILE", parse(from_os_str))]
    files: Vec<PathBuf>,
}

fn main() {
    // Prints each argument on a separate line
    let mut opts: Opt = Opt::from_args();
    if opts.files.is_empty() {
        eprintln!("no files provided");
        std::process::exit(1);
    }
    if opts.cwd.is_none() {
        opts.cwd = Some(current_dir().expect("can see current"))
    }
    from_opt(opts);
}

fn from_opt(opt: Opt) {
    if let Some(cwd) = &opt.cwd {
        for argument in &opt.files {
            parse(cwd, argument)
        }
    }
}

fn parse(cwd: impl Into<PathBuf>, pb: impl Into<PathBuf>) {
    let cwd = cwd.into();
    let subject_file = cwd.join(pb.into());
    let cm = Arc::<SourceMap>::default();
    let handler = Arc::new(Handler::with_tty_emitter(
        ColorConfig::Auto,
        true,
        false,
        Some(cm.clone()),
    ));
    let c = swc::Compiler::new(cm.clone(), handler.clone());
    let fm = cm.load_file(&subject_file).expect("failed to load file");

    let p = c
        .parse_js(
            fm,
            Default::default(),
            swc_ecma_parser::Syntax::Typescript(TsConfig {
                tsx: true,
                ..Default::default()
            }),
            true,
            false,
        )
        .expect("failed to process file");
    run(&cwd, p);
}

fn run(cwd: &PathBuf, p: swc_ecma_ast::Program) {
    match p {
        Program::Module(m) => {
            m.body.iter().for_each(|item| match item {
                swc_ecma_ast::ModuleItem::ModuleDecl(m) => {
                    match m {
                        ModuleDecl::Import(imp) => {
                            println!("from '{}'", imp.src.value);
                            resolve::resolve(cwd, &imp.src.value);
                            for s in &imp.specifiers {
                                match s {
                                    ImportSpecifier::Named(n) => {
                                        println!("   {}", n.local.sym);
                                    }
                                    ImportSpecifier::Default(def) => {
                                        // println!("def={:?}", def)
                                        println!("   {}", def.local.sym);
                                    }
                                    ImportSpecifier::Namespace(ns) => {
                                        // println!("ns={:?}", ns)
                                        println!("   {}", ns.local.sym);
                                    }
                                }
                            }
                        }
                        _ => {
                            // noop
                        }
                    }
                }
                _ => {
                    println!("--")
                }
            })
        }
        Program::Script(_) => todo!("script not supported"),
    }
}
