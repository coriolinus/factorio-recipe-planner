use anyhow::{anyhow, bail, Context, Result};
use factorio_recipe_planner::generic_transform;
use full_moon::ast;

fn main() -> Result<()> {
    const DATA: &str = include_str!("../prototype-data.lua");
    const EXPECTED_HEADER: &str = "Script @__DataRawSerpent__/data-final-fixes.lua:1: ";

    let data = {
        let mut d = String::from("local data = ");
        let proper_data = DATA
            .strip_prefix(EXPECTED_HEADER)
            .ok_or(anyhow!("expected prefix not found"))?;
        d.push_str(proper_data);
        d
    };

    let parsed_ast = full_moon::parse(&data).context("parsing lua")?;
    debug_assert_eq!(
        parsed_ast.nodes().stmts().count(),
        1,
        "expect a single assignment statement"
    );

    let stmt = parsed_ast
        .nodes()
        .stmts()
        .next()
        .ok_or(anyhow!("no statement in lua expression"))?;

    let ast::Stmt::LocalAssignment(assignment) = stmt else {
        let json = serde_json::to_string_pretty(stmt).unwrap();
        std::fs::write("err.json", json).unwrap();
        bail!("statement was not assignment");
    };

    debug_assert_eq!(
        assignment.expressions().len(),
        1,
        "expect a single assignment expression"
    );

    let expression = assignment
        .expressions()
        .first()
        .ok_or(anyhow!("no expression in assignment"))?
        .value();

    let generic_transform::Value::Table(table) =
        generic_transform::parse_value(expression).context("parsing top-level value")?
    else {
        bail!("assignment expression was not table");
    };

    let mut keys = table.keys().collect::<Vec<_>>();
    keys.sort_unstable();
    for key in keys {
        println!("{key}");
    }

    Ok(())
}
