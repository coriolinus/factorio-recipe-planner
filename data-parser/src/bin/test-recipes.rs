use std::collections::HashMap;

use anyhow::{Context, Result};
use data_parser::models::Recipe;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct WrappedRecipe {
    #[serde(flatten)]
    _recipe: Recipe,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

fn main() -> Result<()> {
    const RECIPES: &str = include_str!("../../../prototype-data/recipe.json");

    let deserializer = &mut serde_json::Deserializer::from_str(RECIPES);
    let recipes: HashMap<String, WrappedRecipe> =
        serde_path_to_error::deserialize(deserializer).context("parsing recipes")?;

    let mut parse_incomplete = 0;

    for (name, wrap) in recipes.iter() {
        if !wrap.extra.is_empty() {
            parse_incomplete += 1;
            println!("{name}:");
            for key in wrap.extra.keys() {
                println!("  {key}")
            }
        }
    }

    println!(
        "validated parsing of {} recipes; found {} incomplete",
        recipes.len(),
        parse_incomplete
    );

    Ok(())
}
