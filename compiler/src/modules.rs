use crate::ast::{Item, Program};
use crate::errors::CompileError;
use crate::lexer::tokenize;
use crate::parser::parse_program;
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct PackageMeta {
    pub name: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub functions: Vec<String>,
}

/// Resolves `@blueprint import modules/<libname>` annotations in the program.
/// For each module imported, it reads `package.bls`, `package.bl`, and the function files
/// in `functions/` (or designated in `package.bl`), parses them into AST items,
/// and merges them into the main Program.
pub fn resolve_modules(base_dir: &Path, program: &mut Program) -> Result<(), CompileError> {
    let mut resolved_modules = HashSet::new();

    let module_targets: Vec<String> = program
        .annotations
        .iter()
        .filter(|ann| ann.name == "blueprint" && ann.target.starts_with("modules/"))
        .map(|ann| ann.target.trim_start_matches("modules/").to_string())
        .collect();

    for mod_name in module_targets {
        if mod_name.is_empty() {
            continue;
        }
        if !resolved_modules.contains(&mod_name) {
            resolved_modules.insert(mod_name.clone());
            load_module(base_dir, &mod_name, program)?;
        }
    }

    Ok(())
}

fn load_module(base_dir: &Path, mod_name: &str, program: &mut Program) -> Result<(), CompileError> {
    let raw_name = mod_name.trim_end_matches(".blm");

    // 0. Check if a precompiled .blm file exists in:
    //    - modules/<name>.blm
    //    - modules/<name>
    //    - <name>.blm (in project root)
    //    - modules/<name>/<name>.blm
    //    - modules/<name>/package.blm
    let candidates = [
        base_dir.join("modules").join(format!("{}.blm", raw_name)),
        base_dir.join("modules").join(mod_name),
        base_dir.join(format!("{}.blm", raw_name)),
        base_dir.join(mod_name),
        base_dir.join("modules").join(raw_name).join(format!("{}.blm", raw_name)),
        base_dir.join("modules").join(raw_name).join("package.blm"),
    ];

    let mut blm_path = None;
    for cand in &candidates {
        if cand.is_file() {
            blm_path = Some(cand.clone());
            break;
        }
    }

    if let Some(blm_file) = blm_path {
        let blm_code = fs::read_to_string(&blm_file).map_err(|e| {
            CompileError::new(&format!(
                "Failed to read compiled module '{}': {}",
                blm_file.display(),
                e
            ))
        })?;
        let blm_code = blm_code.strip_prefix('\u{FEFF}').unwrap_or(&blm_code);
        let tokens = tokenize(blm_code)?;
        let parsed = parse_program(blm_code, tokens)?;
        for item in parsed.items {
            match item {
                Item::Func(_) | Item::Struct(_) | Item::Var(_) => {
                    program.items.push(item);
                }
                _ => {}
            }
        }
        return Ok(());
    }

    let dir_candidates = [
        base_dir.join("modules").join(mod_name),
        base_dir.join("modules").join(raw_name),
        base_dir.join(mod_name),
        base_dir.join(raw_name),
    ];

    let mod_dir = dir_candidates.iter().find(|d| d.is_dir());
    let mod_dir = match mod_dir {
        Some(d) => d,
        None => {
            return Err(CompileError::new(&format!(
                "Module not found: neither compiled module '{}.blm' nor directory 'modules/{}' exists",
                raw_name,
                raw_name
            )));
        }
    };

    // 1. Verify package.bls if present (module manifest)
    let package_bls = mod_dir.join("package.bls");
    let mut declared_functions = Vec::new();
    if package_bls.is_file() {
        let meta_str = fs::read_to_string(&package_bls).map_err(|e| {
            CompileError::new(&format!(
                "Failed to read '{}': {}",
                package_bls.display(),
                e
            ))
        })?;
        if let Ok(meta) = serde_json::from_str::<PackageMeta>(&meta_str) {
            declared_functions = meta.functions;
        }
    }

    // 2. Parse package.bl if present (main package entrypoint)
    let package_bl = mod_dir.join("package.bl");
    if package_bl.is_file() {
        let pkg_code = fs::read_to_string(&package_bl).map_err(|e| {
            CompileError::new(&format!(
                "Failed to read package file '{}': {}",
                package_bl.display(),
                e
            ))
        })?;
        let pkg_code = pkg_code.strip_prefix('\u{FEFF}').unwrap_or(&pkg_code);
        let tokens = tokenize(pkg_code)?;
        let parsed_pkg = parse_program(pkg_code, tokens)?;
        for item in parsed_pkg.items {
            match item {
                // Ignore top-level Setup/Stmt from library package.bl, merge structs & funcs & vars
                Item::Func(_) | Item::Struct(_) | Item::Var(_) => {
                    program.items.push(item);
                }
                _ => {}
            }
        }
    }

    // 3. Scan and load function files in functions/ folder
    let functions_dir = mod_dir.join("functions");
    if functions_dir.is_dir() {
        let entries = fs::read_dir(&functions_dir).map_err(|e| {
            CompileError::new(&format!(
                "Failed to read directory '{}': {}",
                functions_dir.display(),
                e
            ))
        })?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("bl") {
                let func_code = fs::read_to_string(&path).map_err(|e| {
                    CompileError::new(&format!(
                        "Failed to read function file '{}': {}",
                        path.display(),
                        e
                    ))
                })?;
                let func_code = func_code.strip_prefix('\u{FEFF}').unwrap_or(&func_code);
                let tokens = tokenize(func_code)?;
                let parsed_func = parse_program(func_code, tokens)?;
                for item in parsed_func.items {
                    match item {
                        Item::Func(_) | Item::Struct(_) | Item::Var(_) => {
                            program.items.push(item);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // 4. Fallback: if declared_functions from package.bls are not in functions/, check if file is named <func>.bl
    for func_name in declared_functions {
        let func_file = functions_dir.join(format!("{}.bl", func_name));
        if func_file.is_file() {
            // Already loaded above in functions_dir scan
            continue;
        }
        let direct_file = mod_dir.join(format!("{}.bl", func_name));
        if direct_file.is_file() {
            let code = fs::read_to_string(&direct_file).map_err(|e| {
                CompileError::new(&format!(
                    "Failed to read '{}': {}",
                    direct_file.display(),
                    e
                ))
            })?;
            let code = code.strip_prefix('\u{FEFF}').unwrap_or(&code);
            let tokens = tokenize(code)?;
            let parsed = parse_program(code, tokens)?;
            for item in parsed.items {
                match item {
                    Item::Func(_) | Item::Struct(_) | Item::Var(_) => {
                        program.items.push(item);
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
