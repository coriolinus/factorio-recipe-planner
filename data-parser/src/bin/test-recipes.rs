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

    /// Print the reserialized value for this recipe.
    #[arg(long)]
    examine_reserialized: Option<String>,
}

fn main() -> Result<()> {
    const RECIPES: &str = include_str!("../../../prototype-data/recipe.json");

    let args = Args::parse();

    let recipes: HashMap<String, Value> =
        serde_json::from_str(RECIPES).context("parsing recipes")?;

    let mut recipe_count = 0;
    let mut ok_count = 0;
    let mut parse_value_error = 0;
    let mut different_reserialization = 0;

    let mut examined_value = None;
    let mut examined_recipe = None;
    let mut examined_reserialized = None;

    for (name, value) in recipes.into_iter() {
        recipe_count += 1;

        if let Some(examine) = &args.examine_value {
            if examine == &name {
                examined_value = Some(value.clone());
            }
        }

        let recipe = match serde_json::from_value::<Recipe>(value.clone()) {
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

                continue;
            }
        };

        let Ok(reserialized) = serde_json::to_string(&recipe) else {
            println!("{name}: reserialization error");
            continue;
        };
        let Ok(deserialized_value) = serde_json::from_str::<Value>(&reserialized) else {
            println!("{name}: failed to deserialize reserialized value");
            continue;
        };

        if value != deserialized_value {
            different_reserialization += 1;
            println!("{name}: different `serde_json::Value`; possible missing fields");
        }

        if let Some(examine) = &args.examine {
            if examine == &name {
                examined_recipe = Some(recipe);
            }
        }

        if let Some(examine) = &args.examine_reserialized {
            if examine == &name {
                examined_reserialized = Some(deserialized_value);
            }
        }
    }

    println!();
    println!("validated parsing of {recipe_count} recipes:");
    println!("  {ok_count} parsed ok");
    println!("  {parse_value_error} parse errors");
    println!("  {different_reserialization} possible missing fields");

    if let Some(examine) = &args.examine_value {
        println!();
        match examined_value {
            Some(value) => {
                println!("{examine} (original value):");
                println!("{value:#}");
            }
            None => println!("{examine} not found in recipes list"),
        }
    }

    if let Some(examine) = &args.examine_reserialized {
        println!();
        match examined_reserialized {
            Some(value) => {
                println!("{examine} (reserialized value):");
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
