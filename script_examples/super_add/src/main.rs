use irust_api::{Command, OutputEvent, Shutdown};
use rscript::scripting::Scripter;
use rscript::{Hook, VersionReq};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use syn::{visit_mut::VisitMut, File, Item, Visibility};

// Cargo.toml dependencies needed:
// [dependencies]
// irust_api = "*"
// rscript = "*"
// syn = { version = "2.0", features = ["full", "visit-mut"] }
// prettyplease = "0.2"

#[derive(Debug)]
struct TempCrateModifier {
    temp_path: PathBuf,
}

impl TempCrateModifier {
    /// Create a new temporary copy of the crate with all items made public
    fn new(source_path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let temp_path = create_temp_copy(source_path)?;
        dbg!("created", &temp_path);
        make_all_items_public(&temp_path)?;

        Ok(TempCrateModifier { temp_path })
    }

    /// Get the path to the temporary crate
    fn path(&self) -> &Path {
        &self.temp_path
    }
}

impl Drop for TempCrateModifier {
    fn drop(&mut self) {}
}

fn create_temp_copy(source: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let temp_base = env::temp_dir();
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let temp_name = format!("irust_super_add_{}", timestamp);
    let temp_path = temp_base.join(temp_name);

    fs::create_dir_all(&temp_path)?;
    copy_dir_recursive(source, &temp_path)?;

    Ok(temp_path)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            // Skip common directories that shouldn't be copied
            if let Some(dir_name) = src_path.file_name() {
                let dir_str = dir_name.to_string_lossy();
                if matches!(
                    dir_str.as_ref(),
                    "target" | ".git" | "node_modules" | ".cargo"
                ) {
                    continue;
                }
            }
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

fn make_all_items_public(dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    visit_rust_files(dir, &mut |file_path| process_rust_file(file_path))
}

fn visit_rust_files<F>(dir: &Path, callback: &mut F) -> Result<(), Box<dyn std::error::Error>>
where
    F: FnMut(&Path) -> Result<(), Box<dyn std::error::Error>>,
{
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            visit_rust_files(&path, callback)?;
        } else if path.extension().map_or(false, |ext| ext == "rs") {
            callback(&path)?;
        }
    }
    Ok(())
}

fn process_rust_file(file_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(file_path)?;

    // Parse the Rust file into an AST
    let mut ast: File = match syn::parse_str(&content) {
        Ok(ast) => ast,
        Err(_) => {
            // Skip files that can't be parsed (might be proc macro files, etc.)
            return Ok(());
        }
    };

    // Apply the visibility transformer
    let mut transformer = PublicityTransformer::new();
    transformer.visit_file_mut(&mut ast);

    // Only rewrite if changes were made
    if transformer.changes_made > 0 {
        let modified_content = prettyplease::unparse(&ast);
        fs::write(file_path, modified_content)?;
    }

    Ok(())
}

struct PublicityTransformer {
    changes_made: usize,
}

impl PublicityTransformer {
    fn new() -> Self {
        Self { changes_made: 0 }
    }

    fn make_public(&mut self, vis: &mut Visibility) {
        if !matches!(vis, Visibility::Public(_)) {
            *vis = Visibility::Public(syn::token::Pub::default());
            self.changes_made += 1;
        }
    }
}

impl VisitMut for PublicityTransformer {
    fn visit_item_mut(&mut self, item: &mut Item) {
        match item {
            Item::Const(item_const) => {
                self.make_public(&mut item_const.vis);
            }
            Item::Enum(item_enum) => {
                self.make_public(&mut item_enum.vis);
            }
            Item::Fn(item_fn) => {
                self.make_public(&mut item_fn.vis);
            }
            Item::Mod(item_mod) => {
                self.make_public(&mut item_mod.vis);
            }
            Item::Static(item_static) => {
                self.make_public(&mut item_static.vis);
            }
            Item::Struct(item_struct) => {
                self.make_public(&mut item_struct.vis);
                // Also make struct fields public
                if let syn::Fields::Named(ref mut fields) = item_struct.fields {
                    for field in &mut fields.named {
                        self.make_public(&mut field.vis);
                    }
                }
            }
            Item::Trait(item_trait) => {
                self.make_public(&mut item_trait.vis);
            }
            Item::Type(item_type) => {
                self.make_public(&mut item_type.vis);
            }
            Item::Union(item_union) => {
                self.make_public(&mut item_union.vis);
                // Make union fields public too
                for field in &mut item_union.fields.named {
                    self.make_public(&mut field.vis);
                }
            }
            Item::Use(item_use) => {
                self.make_public(&mut item_use.vis);
            }
            _ => {}
        }

        // Continue visiting nested items
        syn::visit_mut::visit_item_mut(self, item);
    }
}

#[derive(Debug, Default)]
struct SuperAdd {
    temp_crates: Vec<TempCrateModifier>,
}

impl Scripter for SuperAdd {
    fn name() -> &'static str {
        "SuperAdd"
    }

    fn script_type() -> rscript::ScriptType {
        rscript::ScriptType::Daemon
    }

    fn hooks() -> &'static [&'static str] {
        &[OutputEvent::NAME, Shutdown::NAME]
    }

    fn version_requirement() -> rscript::VersionReq {
        VersionReq::parse(">=1.50.0").expect("correct version requirement")
    }
}

fn main() {
    let _ = SuperAdd::execute(&mut |hook_name| SuperAdd::run(&mut SuperAdd::default(), hook_name));
}

impl SuperAdd {
    fn run(&mut self, hook_name: &str) {
        match hook_name {
            OutputEvent::NAME => {
                let hook: OutputEvent = Self::read();
                let input = hook.1.trim();

                if input.starts_with(":add") && input.contains("--path") {
                    // Parse the path from the command
                    let path_str = if let Some(path_part) = input.split("--path").nth(1) {
                        path_part.trim().split_whitespace().next().unwrap_or("")
                    } else {
                        ""
                    };

                    if !path_str.is_empty() {
                        let source_path = PathBuf::from(path_str);

                        if source_path.exists() {
                            match TempCrateModifier::new(&source_path) {
                                Ok(temp_crate) => {
                                    let temp_path = temp_crate.path().to_string_lossy();

                                    // Create the modified command with the temp path
                                    let modified_command = input.replace(path_str, &temp_path);

                                    // Store the temp crate to keep it alive
                                    self.temp_crates.push(temp_crate);

                                    // Send the modified command
                                    let cmd = Command::CargoAddCommand(modified_command);
                                    Self::write::<OutputEvent>(&Some(cmd));
                                    return;
                                }
                                Err(e) => {
                                    eprintln!("SuperAdd: Failed to create temp crate copy: {}", e);
                                }
                            }
                        } else {
                            eprintln!("SuperAdd: Path does not exist: {}", path_str);
                        }
                    }
                }

                // If we get here, either it's not an :add --path command or something went wrong
                Self::write::<OutputEvent>(&None);
            }

            Shutdown::NAME => {
                // Clean up all temp crates
                self.temp_crates.clear();
                Self::write::<Shutdown>(&None);
            }

            _ => unreachable!(),
        }
    }
}
