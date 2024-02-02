pub mod generic_transform;

/// Parse the definitions into a `serde_json::Value`.
pub fn parse_generic(mut prototype_data: &str) -> Result<serde_json::Value, Error> {
    const EXPECTED_HEADER: &str = "Script @__DataRawSerpent__/data-final-fixes.lua:1: ";

    // strip the header if it appears; don't change the input otherwise
    prototype_data = match prototype_data.strip_prefix(EXPECTED_HEADER) {
        Some(data) => data,
        None => prototype_data,
    };

    let data = {
        let local_assignment = "local data = ";
        let mut d = String::with_capacity(local_assignment.len() + prototype_data.len());
        d.push_str(local_assignment);
        d.push_str(prototype_data);
        d
    };

    let parsed_ast = full_moon::parse(&data)?;
    debug_assert_eq!(
        parsed_ast.nodes().stmts().count(),
        1,
        "expect a single assignment statement"
    );

    let stmt = parsed_ast
        .nodes()
        .stmts()
        .next()
        .expect("given a parsed AST with a partial statement, we must have at least 1 stmt");

    let full_moon::ast::Stmt::LocalAssignment(assignment) = stmt else {
        panic!("we created a local assignment statement but statement was not assignment");
    };

    debug_assert_eq!(
        assignment.expressions().len(),
        1,
        "expect a single assignment expression"
    );

    let expression = assignment
        .expressions()
        .first()
        .expect("we created a single-expression assignment but no expressions found")
        .value();

    let gt_value = generic_transform::parse_value(expression)?;

    Ok(gt_value.into())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("parsing lua input")]
    Lua(#[from] full_moon::Error),
    #[error("performing generic tranform on input data")]
    GenericTransform(#[from] generic_transform::Error),
}
