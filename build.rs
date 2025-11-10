use std::collections::HashSet;
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

fn get_files_in_dir(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        files.push(path);
    }
    files
}

fn get_file_include_paths(dir: PathBuf) -> Vec<PathBuf> {
    let file_contents = fs::read_to_string(dir.clone());
    if file_contents.is_err() {
        panic!("Failed to read file: {}", dir.to_str().unwrap());
    }
    let file_contents = file_contents.unwrap();

    let includes: Vec<PathBuf> = file_contents
        .lines()
        .filter(|line| line.starts_with("#include"))
        .map(|line| line.split(" ").nth(1).unwrap().trim_matches('"'))
        .map(|path| PathBuf::from(path))
        .collect();

    includes
}

fn get_recursive_include_paths_recursion(
    dir: &Path,
    already_included: &mut HashSet<PathBuf>,
) -> Vec<PathBuf> {
    let mut file_includes = get_file_include_paths(dir.to_path_buf());
    if dir.to_str().unwrap().starts_with("src/opencl/headers") {
        file_includes.extend(get_file_include_paths(PathBuf::from(
            dir.to_str()
                .unwrap()
                .replace("src/opencl/headers", "src/opencl/implementations")
                .replace(".cl.h", ".cl"),
        )));
    }

    let mut all_to_include: Vec<PathBuf> = file_includes
        .clone()
        .into_iter()
        .filter(|path| !already_included.contains(path))
        .collect();

    if all_to_include.len() == 0 {
        return vec![];
    }

    already_included.extend(all_to_include.clone());

    for include_path in file_includes.clone() {
        all_to_include.extend(get_recursive_include_paths_recursion(
            &include_path,
            already_included,
        ));
    }

    let all_to_include_without_duplicates: HashSet<_> = all_to_include.into_iter().collect();
    all_to_include_without_duplicates.into_iter().collect()
}

fn get_recursive_include_paths(dir: &Path) -> Vec<PathBuf> {
    let mut already_included = HashSet::new();
    let paths_to_include = get_recursive_include_paths_recursion(dir, &mut already_included);
    let paths_to_include_remove_duplicates: HashSet<_> = paths_to_include.into_iter().collect();
    paths_to_include_remove_duplicates.into_iter().collect()
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
enum SourceType {
    Header,
    Struct,
    Definition,
    Implementation,
    Kernel,
}

#[derive(Clone)]
struct SourceCode {
    content: String,
    source_type: SourceType,
    path: PathBuf,
}

fn get_source_code(path: PathBuf) -> SourceCode {
    let path_str = path.to_str().unwrap();
    let source_type = if path_str.starts_with("src/opencl/headers") {
        SourceType::Header
    } else if path_str.starts_with("src/opencl/definitions") {
        SourceType::Definition
    } else if path_str.starts_with("src/opencl/implementations") {
        SourceType::Implementation
    } else if path_str.starts_with("src/opencl/structs") {
        SourceType::Struct
    } else if path_str.starts_with("src/opencl/kernels") {
        SourceType::Kernel
    } else {
        panic!("Unknown source type: {}", path_str);
    };

    let content = fs::read_to_string(path.clone())
        .unwrap_or_else(|e| panic!("Failed to read file: {} - Error: {}", path_str, e))
        .lines()
        .filter(|line| !line.starts_with("#include"))
        .collect::<Vec<&str>>()
        .join("\n");

    SourceCode {
        path,
        content,
        source_type,
    }
}

fn get_header_implementation_source_code(header: SourceCode) -> SourceCode {
    if header.source_type != SourceType::Header {
        panic!("Header implementation source code can only be called on header files");
    }
    let path = PathBuf::from(
        header
            .path
            .to_str()
            .unwrap()
            .replace("src/opencl/headers", "src/opencl/implementations")
            .replace(".cl.h", ".cl"),
    );

    get_source_code(path)
}

fn get_compiled_source_code(path: PathBuf) -> String {
    let file_source_code = get_source_code(path.clone());
    let file_includes = get_recursive_include_paths(&path);
    let includes_source_codes = file_includes
        .iter()
        .map(|include| get_source_code(include.clone()))
        .collect::<Vec<SourceCode>>();
    let mut includes_with_implementations = includes_source_codes.clone();

    for include in includes_source_codes.clone() {
        if include.source_type == SourceType::Header {
            includes_with_implementations.push(get_header_implementation_source_code(include));
        }
    }

    let mut ordered_includes = includes_with_implementations.clone();

    ordered_includes.sort_by(|a, b| match (&a.source_type, &b.source_type) {
        (SourceType::Struct, SourceType::Struct) => std::cmp::Ordering::Equal,
        (SourceType::Struct, _) => std::cmp::Ordering::Less,
        (_, SourceType::Struct) => std::cmp::Ordering::Greater,
        (SourceType::Definition, SourceType::Definition) => std::cmp::Ordering::Equal,
        (SourceType::Definition, _) => std::cmp::Ordering::Less,
        (_, SourceType::Definition) => std::cmp::Ordering::Greater,
        (SourceType::Header, SourceType::Header) => std::cmp::Ordering::Equal,
        (SourceType::Header, _) => std::cmp::Ordering::Less,
        (_, SourceType::Header) => std::cmp::Ordering::Greater,
        (SourceType::Implementation, SourceType::Implementation) => std::cmp::Ordering::Equal,
        (SourceType::Implementation, _) => std::cmp::Ordering::Less,
        (_, SourceType::Implementation) => std::cmp::Ordering::Greater,
        (SourceType::Kernel, SourceType::Kernel) => std::cmp::Ordering::Equal,
    });

    let mut combined_source_code = ordered_includes
        .iter()
        .map(|source_code| source_code.content.clone())
        .collect::<Vec<String>>()
        .join("\n");

    combined_source_code.push_str(&file_source_code.content);

    combined_source_code
}

fn main() {
    let kernels_dir = Path::new("src/opencl/kernels");

    println!("cargo:rerun-if-changed=src/opencl");

    let out_dir = env::var("OUT_DIR").unwrap();

    for kernel_path in get_files_in_dir(kernels_dir) {
        let compiled_source_code = get_compiled_source_code(kernel_path.clone());
        let out_path = Path::new(&out_dir).join(
            kernel_path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .replace(".cl", ""),
        );
        save_string_to_file(compiled_source_code, out_path);
    }
}

fn save_string_to_file(string: String, path: PathBuf) {
    let mut file = fs::File::create(path).unwrap();
    file.write_all(string.as_bytes()).unwrap();
}
