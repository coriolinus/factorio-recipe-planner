use std::collections::HashMap;

use anyhow::{Context, Result};
use data_parser::models::Recipe;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
struct WrappedRecipe {
    #[serde(flatten)]
    _recipe: Recipe,
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

fn main() -> Result<()> {
    const RECIPES: &str = include_str!("../../../prototype-data/recipe.json");

    let recipes: HashMap<String, Value> =
        serde_json::from_str(RECIPES).context("parsing recipes")?;

    let mut recipe_count = 0;
    let mut parse_error = 0;
    let mut parse_incomplete = 0;

    for (name, value) in recipes.into_iter() {
        recipe_count += 1;
        let wrapped_recipe = match serde_json::from_value::<WrappedRecipe>(value) {
            Ok(recipe) => recipe,
            Err(err) => {
                parse_error += 1;
                println!("{name}: parse error:");
                let mut err: Option<&dyn std::error::Error> = Some(&err);
                while let Some(dyn_err) = err {
                    println!("  {dyn_err}");
                    err = dyn_err.source();
                }
                continue;
            }
        };
        if !wrapped_recipe.extra.is_empty() {
            parse_incomplete += 1;
            println!("{name}: incomplete top-level parse");
            for key in wrapped_recipe.extra.keys() {
                println!("  {key}")
            }
        }
    }

    println!();
    println!("validated parsing of {recipe_count} recipes:");
    println!("  {parse_error} parse errors");
    println!("  {parse_incomplete} incomplete parses");

    Ok(())
}
