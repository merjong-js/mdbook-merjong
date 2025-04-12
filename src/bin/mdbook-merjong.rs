use anyhow::Result;
use clap::{Parser, Subcommand};
use mdbook::preprocess::{CmdPreprocessor, Preprocessor};
use mdbook_merjong::Merjong;
use std::{io, path::PathBuf, process};

#[derive(Parser)]
#[command( version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Supports {
        renderer: String,
    },

    #[cfg(feature = "cli-install")]
    Install {
        dir: Option<PathBuf>,
    },
}

fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    let cli = Cli::parse();
    if let Err(error) = run(cli) {
        log::error!("Fatal error: {}", error);
        for error in error.chain() {
            log::error!("  - {}", error);
        }
        process::exit(1);
    }
}

fn run(cli: Cli) -> Result<()> {
    match cli.command {
        None => handle_preprocessing(),
        Some(Commands::Supports { renderer }) => {
            handle_supports(renderer);
        }

        #[cfg(feature = "cli-install")]
        Some(Commands::Install { dir }) => {
            install::handle_install(dir.unwrap_or_else(|| PathBuf::from(".")))
        }
    }
}

fn handle_preprocessing() -> std::result::Result<(), mdbook::errors::Error> {
    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;

    if ctx.mdbook_version != mdbook::MDBOOK_VERSION {
        eprintln!(
            "Warning: The mdbook-merjong preprocessor was built against version \
             {} of mdbook, but we're being called from version {}",
            mdbook::MDBOOK_VERSION,
            ctx.mdbook_version
        );
    }

    let processed_book = Merjong.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;

    Ok(())
}

fn handle_supports(renderer: String) -> ! {
    let supported = Merjong.supports_renderer(&renderer);

    if supported {
        process::exit(0);
    } else {
        process::exit(1);
    }
}

#[cfg(feature = "cli-install")]
mod install {
    use anyhow::{Context, Result};
    use std::{
        fs::{self, File},
        io::Write,
        path::PathBuf,
    };
    use toml_edit::{self, Array, DocumentMut, Item, Table, Value};

    const MERJONG_JS_FILES: &[(&str, &[u8])] = &[
        ("merjong.min.js", include_bytes!("assets/merjong.min.js")),
        ("merjong-init.js", include_bytes!("assets/merjong-init.js")),
    ];

    trait ArrayExt {
        fn contains_str(&self, value: &str) -> bool;
    }

    impl ArrayExt for Array {
        fn contains_str(&self, value: &str) -> bool {
            self.iter().any(|element| match element.as_str() {
                None => false,
                Some(element_str) => element_str == value,
            })
        }
    }

    pub fn handle_install(proj_dir: PathBuf) -> Result<()> {
        let config = proj_dir.join("book.toml");
        log::info!("Reading configuration file '{}'", config.display());
        let toml = fs::read_to_string(&config)
            .with_context(|| format!("can't read configuration file '{}'", config.display()))?;
        let mut doc = toml
            .parse::<DocumentMut>()
            .context("configuration is not valid TOML")?;

        let _ = preprocessor(&mut doc);

        let mut additional_js = additional_js(&mut doc);
        for (name, content) in MERJONG_JS_FILES {
            let filepath = proj_dir.join(name);

            if let Ok(ref mut additional_js) = additional_js {
                if !additional_js.contains_str(name) {
                    log::info!("Adding '{}' to 'additional-js'", name);
                    additional_js.push(*name);
                }
            } else {
                log::warn!("Unexpected configuration, not updating 'additional-css'");
            }

            log::info!(
                "Copying '{name}' to '{filepath}'",
                filepath = filepath.display()
            );
            let mut file = File::create(&filepath).context("can't open file for writing")?;
            file.write_all(content)
                .context("can't write content to file")?;
        }

        let new_toml = doc.to_string();
        if new_toml != toml {
            log::info!("Saving changed configuration to '{}'", config.display());
            let mut file =
                File::create(config).context("can't open configuration file for writing.")?;
            file.write_all(new_toml.as_bytes())
                .context("can't write configuration")?;
        } else {
            log::info!("Configuration '{}' already up to date", config.display());
        }

        log::info!("mdbook-merjong is now installed. You can start using it in your book.");
        let codeblock = r#"```merjong
111m
```"#;
        log::info!("Add a code block like:\n{}", codeblock);

        Ok(())
    }

    fn additional_js(doc: &mut DocumentMut) -> Result<&mut Array, ()> {
        let doc = doc.as_table_mut();

        let empty_table = Item::Table(Table::default());
        let empty_array = Item::Value(Value::Array(Array::default()));

        doc.entry("output")
            .or_insert(empty_table.clone())
            .as_table_mut()
            .and_then(|item| {
                item.entry("html")
                    .or_insert(empty_table)
                    .as_table_mut()?
                    .entry("additional-js")
                    .or_insert(empty_array)
                    .as_value_mut()?
                    .as_array_mut()
            })
            .ok_or(())
    }

    fn preprocessor(doc: &mut DocumentMut) -> Result<&mut Item, ()> {
        let doc = doc.as_table_mut();

        let empty_table = Item::Table(Table::default());
        let item = doc.entry("preprocessor").or_insert(empty_table.clone());
        let item = item
            .as_table_mut()
            .ok_or(())?
            .entry("merjong")
            .or_insert(empty_table);
        item["command"] = toml_edit::value("mdbook-merjong");
        Ok(item)
    }
}
