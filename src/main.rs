use std::path::PathBuf;
use std::{path::Path, sync::Arc};
use swc::{self, config::Options};
use swc_common::{
    errors::{ColorConfig, Handler},
    SourceMap,
};
use swc_ecma_ast::{ImportSpecifier, ModuleDecl, ModuleItem, Program};
use swc_ecma_parser::token::Keyword::Default_;
use swc_ecma_parser::TsConfig;

fn main() {
    // Prints each argument on a separate line
    for argument in std::env::args().skip(1) {
        parse(argument)
    }
}

fn parse(pb: impl Into<PathBuf>) {
    let cm = Arc::<SourceMap>::default();
    let handler = Arc::new(Handler::with_tty_emitter(
        ColorConfig::Auto,
        true,
        false,
        Some(cm.clone()),
    ));
    let c = swc::Compiler::new(cm.clone(), handler.clone());

    let fm = cm.load_file(&pb.into()).expect("failed to load file");

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
    run(p);
}

fn run(p: swc_ecma_ast::Program) {
    match p {
        Program::Module(m) => {
            m.body.iter().for_each(|item| match item {
                swc_ecma_ast::ModuleItem::ModuleDecl(m) => {
                    match m {
                        ModuleDecl::Import(imp) => {
                            println!("from '{}'", imp.src.value);
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
                    println!("non-moduledecrl")
                }
            })
        }
        Program::Script(_) => todo!("script not supported"),
    }
}
