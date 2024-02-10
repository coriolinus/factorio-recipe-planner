//! # Factorio Recipe Planner: Data Parser
//!
//! This crate implements low-level parsing of Factorio data. It offers three
//! key facilities:
//!
//! ## `into-json` script
//!
//! This script converts a raw Factorio data Lua dump into one or many JSON
//! files.
//!
//! Example usage:
//!
//! ```sh
//! cargo run --bin into-json -- --split-toplevel prototype-data.lua prototype-data
//! ```
//!
//! This will produce a `prototype-data` directory containing a JSON file for
//! each top-level key of the input Lua definition. Using the `--split-toplevel`
//! flag is recommended, as otherwise the output JSON file is massive.
//!
//! ## [`parse_lua`] function
//!
//! This function offers a programmatic interface to producing JSON from
//! Factorio Lua prototypes.
//!
//! ## [`models`] module
//!
//! This module contains low-level Serde-compatible models which can be used to
//! parse Factorio prototypes.
//!
//! This module is intentionally incomplete, as Factorio uses a very flexible
//! prototype system which is presumably quite nice to write by hand but which
//! is a pain to parse precisely. For now, the only type which is complete is
//! the [`Recipe`][models::Recipe] struct, as that is the focus of the
//! downstream tooling for which this library was initially written.
//!
//! Models which have been indicated to be complete parse losslessly from
//! Factorio definitions. They do not quite reserialize identically to the
//! original definitions, but the reserialization preserves identical semantics.
//! (The exception is the `duration` field--serialized as
//! `energy_required`--which always reserializes as a float even when it has an
//! integral value.)
//!
//! Because they have been optimized for lossless conversion from Factorio
//! definitions, these models can be a pain to work with in Rust code. It is
//! recommended to convert them into higher-level models before executing the
//! main logic of your program. In the future, a crate in this workspace will
//! provide appropriate higher-level models.

pub mod generic_transform;
pub mod models;

/// Parse the definitions into a `serde_json::Value`.
pub fn parse_lua(mut prototype_data: &str) -> Result<serde_json::Value, Error> {
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

    let parsed_ast = full_moon::parse(&data).map_err(Box::new)?;
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
    Lua(#[from] Box<full_moon::Error>),
    #[error("performing generic tranform on input data")]
    GenericTransform(#[from] generic_transform::Error),
}
