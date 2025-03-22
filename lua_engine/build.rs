// lua_engine/build.rs
use regex::Regex;
use std::env;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

fn main() {
    // Path to logic package source
    let core_src = "../logic/src/api";

    // Output file in lua_engine
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("api_docs.rs");
    let mut output = File::create(&dest_path).unwrap();

    // Parse all Rust files to extract documentation
    let api_files = find_api_files(core_src);
    let mut all_docs = Vec::new();

    println!("cargo:warning=Found API files: {:?}", api_files);
    for file in api_files {
        let docs = extract_docs_from_file(&file);
        all_docs.extend(docs);
    }

    // Generate Rust code for the documentation
    writeln!(output, "// Generated API documentation").unwrap();
    writeln!(
        output,
        "pub fn get_api_docs() -> std::collections::HashMap<String, ApiModuleDocs> {{"
    )
    .unwrap();
    writeln!(
        output,
        "    let mut docs = std::collections::HashMap::new();"
    )
    .unwrap();

    // Group by API module
    let mut modules = std::collections::HashMap::new();
    println!("cargo:warning=Extracted docs: {:?}", all_docs);
    for doc in all_docs {
        modules
            .entry(doc.module.clone())
            .or_insert_with(Vec::new)
            .push(doc);
    }

    // Generate code for each module
    for (module, methods) in modules {
        writeln!(output, "    {{").unwrap();
        writeln!(
            output,
            "        let mut module_docs = ApiModuleDocs::new(\"{}\");",
            module
        )
        .unwrap();

        for method in methods {
            writeln!(output, "        {{").unwrap();
            writeln!(output, "            let mut method_doc = MethodDoc::new();").unwrap();
            writeln!(
                output,
                "            method_doc.name = \"{}\".to_string();",
                method.name
            )
            .unwrap();
            writeln!(
                output,
                "            method_doc.description = \"{}\".to_string();",
                method.description.replace("\"", "\\\"")
            )
            .unwrap();

            // Parameters
            writeln!(output, "            {{").unwrap();
            for param in method.params {
                writeln!(output, "                method_doc.params.push(ParamDoc {{").unwrap();
                writeln!(
                    output,
                    "                    name: \"{}\".to_string(),",
                    param.name
                )
                .unwrap();
                writeln!(
                    output,
                    "                    type_name: \"{}\".to_string(),",
                    param.type_name
                )
                .unwrap();
                writeln!(
                    output,
                    "                    description: \"{}\".to_string(),",
                    param.description.replace("\"", "\\\"")
                )
                .unwrap();
                writeln!(output, "                }});").unwrap();
            }
            writeln!(output, "            }}").unwrap();

            writeln!(
                output,
                "            method_doc.returns = \"{}\".to_string();",
                method.returns.replace("\"", "\\\"")
            )
            .unwrap();
            writeln!(
                output,
                "            module_docs.methods.insert(\"{}\".to_string(), method_doc);",
                method.name
            )
            .unwrap();
            writeln!(output, "        }}").unwrap();
        }

        writeln!(
            output,
            "        docs.insert(\"{}\".to_string(), module_docs);",
            module
        )
        .unwrap();
        writeln!(output, "    }}").unwrap();
    }

    writeln!(output, "    docs").unwrap();
    writeln!(output, "}}").unwrap();

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", core_src);
}

// Helper function to find all API files
fn find_api_files(dir: &str) -> Vec<String> {
    let mut result = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    let mut subdir_files = find_api_files(path.to_str().unwrap());
                    result.append(&mut subdir_files);
                } else if let Some(ext) = path.extension() {
                    if ext == "rs" && path.to_str().unwrap().contains("api") {
                        result.push(path.to_str().unwrap().to_string());
                    }
                }
            }
        }
    }
    result
}

// Documentation data structures
#[derive(Debug)]
struct MethodDoc {
    module: String,
    name: String,
    description: String,
    params: Vec<ParamDoc>,
    returns: String,
}

#[derive(Debug)]
struct ParamDoc {
    name: String,
    type_name: String,
    description: String,
}

// Extract documentation from a file
fn extract_docs_from_file(file_path: &str) -> Vec<MethodDoc> {
    let file = File::open(file_path).unwrap();
    let reader = BufReader::new(file);
    let mut docs = Vec::new();

    // Extract module name from file path
    let module_name = Path::new(file_path)
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .replace("_api", "");

    let mut current_doc = String::new();
    let mut in_doc = false;
    let mut next_is_method = false;

    // Method signature regex
    let method_re = Regex::new(r"pub fn (\w+)\s*\((?:&self,?\s*)?(.+)?\)\s*->\s*(.+)").unwrap();
    let param_re = Regex::new(r"(\w+):\s*([^,]+)").unwrap();

    for line in reader.lines() {
        let line = line.unwrap();

        if line.trim().starts_with("///") {
            // Inside a doc comment
            in_doc = true;
            current_doc.push_str(&line.trim()[3..].trim());
            current_doc.push('\n');
            next_is_method = true;
        } else if in_doc && !line.trim().starts_with("///") {
            // End of doc comment, check if it's a method
            in_doc = false;

            if next_is_method && line.trim().starts_with("pub fn") {
                if let Some(captures) = method_re.captures(&line) {
                    let method_name = captures.get(1).unwrap().as_str();
                    let params_str = captures.get(2).map_or("", |m| m.as_str());
                    let return_type = captures.get(3).map_or("", |m| m.as_str());

                    // Parse parameters
                    let mut params = Vec::new();
                    for cap in param_re.captures_iter(params_str) {
                        params.push(ParamDoc {
                            name: cap[1].to_string(),
                            type_name: cap[2].trim().to_string(),
                            description: "".to_string(), // We don't have parameter descriptions in this example
                        });
                    }

                    // Create method doc
                    let doc = MethodDoc {
                        module: module_name.clone(),
                        name: method_name.to_string(),
                        description: current_doc.trim().to_string(),
                        params,
                        returns: return_type.trim().to_string(),
                    };

                    docs.push(doc);
                }
            }

            current_doc.clear();
            next_is_method = false;
        }
    }

    docs
}
