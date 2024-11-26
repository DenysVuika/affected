use std::rc::Rc;
use swc_common::{sync::Lrc, FileName, SourceMap};
use swc_ecma_ast::{CallExpr, Callee, EsVersion, Expr, ModuleDecl, ModuleItem};
use swc_ecma_parser::{lexer::Lexer, Capturing, Parser, StringInput, Syntax, TsSyntax};

pub fn execute_esm() {
    // Step 1: Set up a SourceMap and FileName
    let cm: Lrc<SourceMap> = Default::default();

    // Example TypeScript code (you can replace this with file input)
    let ts_code = r#"
        import { A } from './a';
        import B from './b';
        import * as C from './c';
        import './side-effect-only';
    "#;

    // Step 2: Create a SourceFile
    let fm = cm.new_source_file(
        Rc::new(FileName::Custom("example.ts".into())),
        ts_code.into(),
    );

    // Step 3: Configure the lexer for TypeScript
    let syntax = Syntax::Typescript(TsSyntax {
        tsx: false,        // Set to true if using TSX (React-style syntax)
        decorators: false, // Enable if using decorators
        ..Default::default()
    });

    let input = StringInput::from(&*fm);

    let lexer = Lexer::new(syntax, EsVersion::Es2022, input, None);

    // Optionally, wrap the lexer in a capturing wrapper
    let capturing_lexer = Capturing::new(lexer);

    // Step 4: Create the parser
    let mut parser = Parser::new_from(capturing_lexer);

    // Step 5: Parse the TypeScript code
    let module = parser.parse_module().expect("Failed to parse TypeScript");

    // Step 6: Analyze the parsed module
    println!("Imports found in the file:");
    for item in module.body {
        if let ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl)) = item {
            let module_specifier = &import_decl.src.value; // The string path of the module
            println!("  - Import: {}", module_specifier);

            // Optional: Inspect specific parts of the import
            if let Some(specifiers) = import_decl.specifiers.first() {
                match specifiers {
                    swc_ecma_ast::ImportSpecifier::Named(named) => {
                        println!("    Named Import: {}", named.local.sym);
                    }
                    swc_ecma_ast::ImportSpecifier::Default(default) => {
                        println!("    Default Import: {}", default.local.sym);
                    }
                    swc_ecma_ast::ImportSpecifier::Namespace(ns) => {
                        println!("    Namespace Import: {}", ns.local.sym);
                    }
                }
            }
        }
    }
}

pub fn execute_common_js() {
    // Step 1: Set up a SourceMap and FileName
    let cm: Lrc<SourceMap> = Default::default();

    // Example TypeScript code with both `import` and `require`
    let ts_code = r#"
        import { Injectable } from '@angular/core';
        const uuidv4 = require('uuid/v4');
    "#;

    // Step 2: Create a SourceFile
    let fm = cm.new_source_file(
        Rc::new(FileName::Custom("example.ts".into())),
        ts_code.into(),
    );

    // Step 3: Configure the lexer for TypeScript
    let syntax = Syntax::Typescript(TsSyntax {
        tsx: false,
        decorators: false,
        ..Default::default()
    });

    let input = StringInput::from(&*fm);

    let lexer = Lexer::new(syntax, EsVersion::Es2022, input, None);

    // Optionally, wrap the lexer in a capturing wrapper
    let capturing_lexer = Capturing::new(lexer);

    // Step 4: Create the parser
    let mut parser = Parser::new_from(capturing_lexer);

    // Step 5: Parse the TypeScript code
    let module = parser.parse_module().expect("Failed to parse TypeScript");

    // Step 6: Analyze the parsed module
    println!("Dependencies found in the file:");
    for item in module.body {
        match item {
            // Handle `import` statements
            ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl)) => {
                let module_specifier = &import_decl.src.value; // The string path of the module
                println!("  - Import: {}", module_specifier);

                // Optional: Inspect specific parts of the import
                for specifier in &import_decl.specifiers {
                    match specifier {
                        swc_ecma_ast::ImportSpecifier::Named(named) => {
                            println!("    Named Import: {}", named.local.sym);
                        }
                        swc_ecma_ast::ImportSpecifier::Default(default) => {
                            println!("    Default Import: {}", default.local.sym);
                        }
                        swc_ecma_ast::ImportSpecifier::Namespace(ns) => {
                            println!("    Namespace Import: {}", ns.local.sym);
                        }
                    }
                }
            }
            // Handle `require` calls
            ModuleItem::Stmt(stmt) => {
                if let swc_ecma_ast::Stmt::Decl(decl) = stmt {
                    if let swc_ecma_ast::Decl::Var(var_decl) = decl {
                        for decl in &var_decl.decls {
                            if let Some(init) = &decl.init {
                                if let Expr::Call(CallExpr { callee, args, .. }) = &**init {
                                    // Match `callee` explicitly
                                    if let Callee::Expr(expr) = callee {
                                        if let Expr::Ident(ident) = &**expr {
                                            if ident.sym == *"require" {
                                                if let Some(arg) = args.get(0) {
                                                    if let Expr::Lit(swc_ecma_ast::Lit::Str(
                                                        module_specifier,
                                                    )) = &*arg.expr
                                                    {
                                                        println!(
                                                            "  - Require: {}",
                                                            module_specifier.value
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
