use std::collections::HashMap;

use anyhow::{Context, Result};
use clap::Parser;
use data_parser::models::Recipe;
use serde_json::Value;

#[derive(Debug, Parser)]
struct Args {
    /// Print debug information about this recipe.
    #[arg(long)]
    examine: Option<String>,

    /// Print the source value for this recipe.
    #[arg(long)]
    examine_value: Option<String>,
}

fn main() -> Result<()> {
    const RECIPES: &str = include_str!("../../../prototype-data/recipe.json");

    let args = Args::parse();

    let recipes: HashMap<String, Value> =
        serde_json::from_str(RECIPES).context("parsing recipes")?;

    let mut recipe_count = 0;
    let mut ok_count = 0;
    let mut parse_value_error = 0;
    let mut parse_str_error = 0;
    let mut parse_str_unexpected_ok = 0;

    let mut examined_value = None;
    let mut examined_recipe = None;

    for (name, value) in recipes.into_iter() {
        recipe_count += 1;

        if let Some(examine) = &args.examine_value {
            if examine == &name {
                examined_value = Some(value.clone());
            }
        }

        let wrapped_recipe = match serde_json::from_value::<Recipe>(value.clone()) {
            Ok(recipe) => {
                ok_count += 1;
                recipe
            }
            Err(err) => {
                parse_value_error += 1;
                println!("{name}: parse error:");
                let mut err: Option<&dyn std::error::Error> = Some(&err);
                while let Some(dyn_err) = err {
                    println!("  {dyn_err}");
                    err = dyn_err.source();
                }

                let compact =
                    serde_json::to_string(&value).context("generating compact str from value")?;
                match serde_json::from_str::<Recipe>(&compact) {
                    Ok(_) => {
                        parse_str_unexpected_ok += 1;
                        println!("but parsing from string was ok!")
                    }
                    Err(_) => {
                        parse_str_error += 1;
                    }
                };

                continue;
            }
        };

        if let Some(examine) = &args.examine {
            if examine == &name {
                examined_recipe = Some(wrapped_recipe);
            }
        }
    }

    println!();
    println!("validated parsing of {recipe_count} recipes:");
    println!("  {ok_count} parsed ok");
    println!("  {parse_value_error} parse errors");
    println!("    {parse_str_error} also failed when parsed as a string");
    println!("    {parse_str_unexpected_ok} succeeded as string parse though");

    if let Some(examine) = &args.examine_value {
        println!();
        match examined_value {
            Some(value) => {
                println!("{examine}:");
                println!("{value:#}");
            }
            None => println!("{examine} not found in recipes list"),
        }
    }

    if let Some(examine) = &args.examine {
        println!();
        match examined_recipe {
            Some(wrap) => {
                println!("{examine}:");
                println!("{wrap:#?}");
            }
            None => println!("{examine} not found in recipes list"),
        }
    }

    Ok(())
}
