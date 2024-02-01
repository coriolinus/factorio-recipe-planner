use std::{
    io::{BufWriter, Read},
    path::PathBuf,
};

use anyhow::{Context, Result};
use clap::Parser;
use factorio_recipe_planner::parse_generic;

/// Convert a set of Lua definitions into an equivalent JSON format.
#[derive(Debug, Parser)]
struct Args {
    /// Input path
    ///
    /// When `-`, reads from stdin.
    #[arg(default_value = "-")]
    input: String,

    /// Output path
    ///
    /// When `-`, writes to stdout.
    #[arg(default_value = "-")]
    output: String,

    /// Split the output by top-level key
    ///
    /// This is only meaningful when the output is a particular path, not stdout.
    /// It changes the semantics of the output path: instead of being the path to
    /// a particular file which contains the entire output, it becomes the path to
    /// a directory.
    ///
    /// Into this directory, the output is divided into files by top-level key.
    /// In the normal case where the input is an object, this produces files named
    /// `accumulator.json`, `ammo.json`, etc.
    ///
    /// In case of unusual input: arrays are named `0.json`, `1.json`, etc. Primitives
    /// are emitted into a json file named for the last directory in the path.
    #[arg(short = 'S', long)]
    split_toplevel: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut data = String::new();
    if args.input == "-" {
        std::io::stdin()
            .lock()
            .read_to_string(&mut data)
            .context("reading data from stdin")?;
    } else {
        data = std::fs::read_to_string(args.input).context("reading data from file")?;
    }

    let value = parse_generic(&data).context("parsing top-level value")?;

    if args.split_toplevel && args.output != "-" {
        let dir = PathBuf::from(args.output);
        std::fs::create_dir_all(&dir).context("creating output directory")?;

        let emit_item = |name: &str, value: &serde_json::Value| -> Result<()> {
            let path = dir.join(format!("{name}.json"));
            let outf = std::fs::File::create(path).context("creating output file")?;
            let mut writer = BufWriter::new(outf);
            serde_json::to_writer_pretty(&mut writer, value).context("serialzing to json file")?;
            Ok(())
        };

        match value {
            serde_json::Value::Object(object) => {
                for (name, value) in object.iter() {
                    emit_item(name, value).context(format!("emitting item \"{name}\""))?;
                }
            }
            serde_json::Value::Array(array) => {
                for (idx, value) in array.iter().enumerate() {
                    let name = idx.to_string();
                    emit_item(&name, value).context(format!("emitting item with idx {idx}"))?;
                }
            }
            _ => {
                let name = match dir.file_name() {
                    Some(name) => name.to_string_lossy(),
                    None => "value".into(),
                };
                emit_item(&name, &value).context("emitting sole item")?;
            }
        };
    } else {
        let writer: Box<dyn std::io::Write> = if args.output == "-" {
            let stdout = std::io::stdout().lock();
            Box::new(stdout)
        } else {
            let path = PathBuf::from(args.output);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .context("creating parent directories for output file")?;
            }
            let outf = std::fs::File::create(&path).context("creating output file")?;
            Box::new(outf)
        };
        let mut writer = BufWriter::new(writer);

        serde_json::to_writer_pretty(&mut writer, &value).context("serializing data to json")?;
    }

    Ok(())
}
