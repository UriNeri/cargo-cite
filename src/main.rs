use gumdrop::Options;
use std::env;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use chrono::Datelike;
use serde::Deserialize;

const CARGO_FILE: &str = "Cargo.toml";
const CITATION_FILE: &str = "CITATION.bib";

#[derive(Debug, Deserialize)]
struct ManifestInfo {
    package: PackageInfo,
    dependencies: Option<std::collections::BTreeMap<String, DependencyInfo>>,
}

#[derive(Debug, Deserialize)]
struct PackageInfo {
    name: String,
    version: String,
    #[serde(default)]
    authors: Vec<String>,
    description: Option<String>,
    repository: Option<String>,
    keywords: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum DependencyInfo {
    Simple(String),
    Detailed {
        version: Option<String>,
        path: Option<String>,
        git: Option<String>,
    },
}

#[derive(Debug, Options)]
struct CitationOption {
    #[options(free)]
    free: Vec<String>,

    #[options(help = "print help message", short = "h")]
    help: bool,

    #[options(help = "Generate CITATION.bib file", short = "g")]
    generate: bool,

    #[options(help = "Over-write existing CITATION.bib file", short = "o")]
    overwrite: bool,

    #[options(help = "Append a \"Citing\" section to the README", short = "r")]
    readme_append: bool,

    #[options(help = "Path to the crate, default to current directory. If not specified, will use current directory and recursively search all subdirectories for Cargo.toml files", short = "p")]
    path: Option<String>,

    #[options(help = "Citation file to add, default to CITATION.bib (recommended). \"STDOUT\" for outputing to standard output.", short = "f")]
    filename: Option<String>,

    #[options(help = "Generate BibTeX entries for all explicit dependencies", short = "d")]
    dependencies: bool,

    #[options(help = "Maximum depth for recursive search (default: unlimited). 0 means only current directory, -1 means unlimited depth.", short = "m")]
    max_depth: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct CratesIoResponse {
    #[serde(rename = "crate")]
    crate_info: CrateInfo,
}

#[derive(Debug, Deserialize)]
struct CrateInfo {
    description: Option<String>,
    repository: Option<String>,
    homepage: Option<String>,
    authors: Option<Vec<String>>,
}

impl PackageInfo {
    pub fn build_bibtex(&self) -> String {
        let t = chrono::prelude::Local::now();
        let description_part = self.description.as_ref()
            .map(|s| format!(": {}", s))
            .unwrap_or_default();

        format!(
            "@misc{{{name},\n\
             \ttitle={{{name}{desc}}},\n\
             \tauthor={{{authors}}},\n\
             \tversion = {{{version}}},\n\
             \tmonth = {month},\n\
             \tyear = {year},\n\
             {repository}\
             {keywords}\
             }}\n",
            name = self.name,
            desc = description_part,
            authors = self.authors.join(" and "),
            version = self.version,
            month = t.month(),
            year = t.year(),
            repository = self.repository.as_ref()
                .map(|url| format!("\turl = {{{}}},\n", url))
                .unwrap_or_default(),
            keywords = self.keywords.as_ref()
                .map(|k| format!("\tkeywords = {{{}}}\n", k.join(", ")))
                .unwrap_or_default()
        )
    }

    fn readme_section(&self) -> String {
        String::from(
"
## Citing

If you found this software useful consider citing it. See CITATION.bib for the recommended BibTeX entry.
"
        )
    }
}

impl DependencyInfo {
    fn get_version(&self) -> Option<String> {
        match self {
            DependencyInfo::Simple(v) => Some(v.clone()),
            DependencyInfo::Detailed { version, .. } => version.clone(),
        }
    }

    fn get_source_info(&self) -> (Option<String>, Option<String>) {
        match self {
            DependencyInfo::Simple(_) => (None, None),
            DependencyInfo::Detailed { path, git, .. } => (path.clone(), git.clone()),
        }
    }
}

impl ManifestInfo {
    async fn fetch_crate_metadata(crate_name: &str) -> Option<CrateInfo> {
        let client = reqwest::Client::new();
        let url = format!("https://crates.io/api/v1/crates/{}", crate_name);
        
        match client.get(&url)
            .header("User-Agent", "cargo-cite")
            .send()
            .await {
                Ok(response) => {
                    if let Ok(data) = response.json::<CratesIoResponse>().await {
                        Some(data.crate_info)
                    } else {
                        None
                    }
                }
                Err(_) => None
            }
    }

    async fn build_dependencies_bibtex(&self) -> String {
        let mut result = String::new();
        if let Some(deps) = &self.dependencies {
            for (name, info) in deps {
                result.push_str("@misc{");
                result.push_str(&format!("rust-{},\n", name));
                result.push_str(&format!("\ttitle={{{}}},\n", name));
                
                // Try to fetch metadata for crates.io dependencies
                let (path_source, git_source) = info.get_source_info();
                let is_regular_dependency = path_source.is_none() && git_source.is_none();
                
                if let Some(path) = path_source {
                    result.push_str(&format!("\tnote = {{Local dependency from path: {}}},\n", path));
                } else if let Some(git) = git_source {
                    result.push_str(&format!("\turl = {{{}}},\n", git));
                    result.push_str("\tnote = {Git dependency},\n");
                } else {
                    // Regular crates.io dependency
                    if let Some(metadata) = Self::fetch_crate_metadata(name).await {
                        if let Some(desc) = metadata.description {
                            result.push_str(&format!("\tnote = {{{}}},\n", desc));
                        }
                        
                        if let Some(authors) = metadata.authors {
                            if !authors.is_empty() {
                                result.push_str(&format!("\tauthor = {{{}}},\n", authors.join(" and ")));
                            }
                        }

                        // Prefer repository URL, fallback to homepage
                        if let Some(url) = metadata.repository.or(metadata.homepage) {
                            result.push_str(&format!("\turl = {{{}}},\n", url));
                        }
                    }
                }

                if let Some(version) = info.get_version() {
                    result.push_str(&format!("\tversion = {{{}}},\n", version));
                }
                
                let t = chrono::prelude::Local::now();
                result.push_str(&format!("\tyear = {},\n", t.year()));
                result.push_str(&format!("\tmonth = {},\n", t.month()));
                
                // Only add crates.io link for regular dependencies
                if is_regular_dependency {
                    result.push_str(&format!("\thowpublished = {{https://crates.io/crates/{}}},\n", name));
                }
                
                result.push_str("}\n\n");
            }
        }
        result
    }
}

fn find_cargo_files(start_dir: &Path, max_depth: Option<i32>) -> Vec<PathBuf> {
    let walker = WalkDir::new(start_dir).follow_links(true);
    
    // Apply max depth if specified, otherwise unlimited
    let walker = match max_depth {
        Some(depth) if depth >= 0 => walker.max_depth(depth as usize),
        Some(_) => walker, // negative means unlimited
        None => walker, // default to unlimited
    };

    walker
        .into_iter()
        .filter_map(|e| match e {
            Ok(entry) => Some(entry),
            Err(err) => {
                println!("Warning: Error accessing path: {}", err);
                None
            }
        })
        .filter(|e| e.file_type().is_file() && e.file_name() == CARGO_FILE)
        .map(|e| {
            println!("Found Cargo.toml at: {:?}", e.path());
            e.path().to_path_buf()
        })
        .collect()
}

async fn process_cargo_file(cargo_path: &Path, opt: &CitationOption) -> Result<(bool, String), Box<dyn std::error::Error>> {
    println!("\nProcessing {:?}", cargo_path);
    
    let mut cargo_file = match fs::File::open(cargo_path) {
        Ok(file) => file,
        Err(e) => {
            println!("Warning: Could not open {:?}: {}", cargo_path, e);
            println!("         Skipping this file.");
            return Ok((false, String::new()));
        }
    };

    let mut cargo_content = String::new();
    if let Err(e) = cargo_file.read_to_string(&mut cargo_content) {
        println!("Warning: Could not read {:?}: {}", cargo_path, e);
        println!("         Skipping this file.");
        return Ok((false, String::new()));
    }

    let manifest: ManifestInfo = match toml::from_str(&cargo_content) {
        Ok(manifest) => manifest,
        Err(e) => {
            println!("Warning: Invalid Cargo.toml at {:?}:", cargo_path);
            println!("         {}", e);
            println!("         Skipping this file.");
            return Ok((false, String::new()));
        }
    };
    
    if opt.dependencies {
        let deps_bibtex = manifest.build_dependencies_bibtex().await;
        return Ok((true, deps_bibtex));
    }

    if opt.readme_append {
        let parent_dir = cargo_path.parent().unwrap();
        for dir_entry in (fs::read_dir(parent_dir)?).flatten() {
            let p = dir_entry.path();
            if p.to_string_lossy().contains("README") {
                println!("Appending to readme file: {:?}", p);
                let mut readme_file = fs::OpenOptions::new().append(true).open(&p)?;
                let readme_section = manifest.package.readme_section();
                readme_file.write_all(readme_section.as_bytes())?;
            }
        }
    }

    let r = manifest.package.build_bibtex();
    let output_file = if let Some(o) = &opt.filename {
        o.clone()
    } else {
        String::from(CITATION_FILE)
    };

    let file_path = cargo_path.parent().unwrap().join(PathBuf::from(&output_file));
    if file_path.exists() && !opt.overwrite {
        println!("Note: Citation file already exists at {:?}.", &file_path);
        println!("      Use --overwrite to replace it.");
        return Ok((false, String::new()));
    }
    
    fs::write(&file_path, r.as_bytes())?;
    println!("Created citation file at {:?}", file_path);
    Ok((true, String::new()))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = CitationOption::parse_args_default_or_exit();

    let start_dir = if let Some(ref s) = opt.path {
        PathBuf::from(s)
    } else {
        match env::current_dir() {
            Ok(dir) => dir,
            Err(e) => {
                println!("Error: Could not access current directory: {}", e);
                return Ok(());
            }
        }
    };

    // Check if the start directory exists
    if !start_dir.exists() {
        println!("Error: Directory {:?} does not exist.", start_dir);
        return Ok(());
    }

    let cargo_files = if opt.dependencies {
        // Only do directory walking for dependencies option
        println!("Searching for Cargo.toml files in {:?}{}", 
            start_dir,
            match opt.max_depth {
                Some(depth) if depth < 0 => String::from(" and all subdirectories"),
                Some(0) => String::from(" (current directory only)"),
                Some(depth) => format!(" (max depth: {})", depth),
                None => String::from(" (searching all subdirectories)"),
            }
        );
        find_cargo_files(&start_dir, opt.max_depth)
    } else {
        // For other operations, just look in the current directory
        let cargo_path = start_dir.join(CARGO_FILE);
        if cargo_path.exists() {
            vec![cargo_path]
        } else {
            println!("Error: No Cargo.toml found in {:?}.", start_dir);
            return Ok(());
        }
    };
    
    if cargo_files.is_empty() {
        if opt.max_depth == Some(0) {
            println!("No Cargo.toml found in {:?}.", start_dir);
            println!("Note: You can use --max-depth N to search subdirectories (N levels deep)");
            println!("      or --max-depth -1 to search all subdirectories.");
        } else {
            println!("No Cargo.toml files found in {:?} or its subdirectories{}", 
                start_dir,
                match opt.max_depth {
                    Some(depth) if depth < 0 => String::new(),
                    Some(depth) if depth > 0 => format!(" (searched {} level{} deep)", 
                        depth,
                        if depth == 1 { "" } else { "s" }
                    ),
                    None => String::new(),
                    _ => String::new(),
                }
            );
        }
        return Ok(());
    }

    println!("\nFound {} Cargo.toml file{}", 
        cargo_files.len(),
        if cargo_files.len() == 1 { "" } else { "s" }
    );

    let mut processed = 0;
    let mut skipped = 0;
    let mut all_dependencies = String::new();

    for cargo_path in cargo_files {
        match process_cargo_file(&cargo_path, &opt).await {
            Ok((success, deps_bibtex)) => {
                if success {
                    processed += 1;
                    if opt.dependencies {
                        all_dependencies.push_str(&deps_bibtex);
                    }
                } else {
                    skipped += 1;
                }
            }
            Err(e) => {
                println!("Warning: Error processing {:?}: {}", cargo_path, e);
                println!("         Skipping this file.");
                skipped += 1;
            }
        }
    }

    // Write combined dependencies to a single file
    if opt.dependencies && !all_dependencies.is_empty() {
        let output_file = if let Some(o) = &opt.filename {
            o.clone()
        } else {
            String::from("DEPENDENCIES.bib")
        };

        if output_file == "STDOUT" {
            print!("{}", all_dependencies);
        } else {
            let file_path = start_dir.join(&output_file);
            if file_path.exists() && !opt.overwrite {
                println!("Note: Dependencies citation file already exists at {:?}.", &file_path);
                println!("      Use --overwrite to replace it.");
            } else {
                fs::write(&file_path, all_dependencies.as_bytes())?;
                println!("Created combined dependencies citation file at {:?}", file_path);
            }
        }
    }

    if processed > 0 || skipped > 0 {
        println!("\nSummary:");
        if processed > 0 {
            println!("- Successfully processed: {} file{}", 
                processed,
                if processed == 1 { "" } else { "s" }
            );
        }
        if skipped > 0 {
            println!("- Skipped due to errors: {} file{}", 
                skipped,
                if skipped == 1 { "" } else { "s" }
            );
        }
    }
    Ok(())
}
