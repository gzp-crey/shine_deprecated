#![feature(step_trait)]
#![feature(crate_visibility_modifier)]
#![feature(try_from)]
#![feature(tool_lints)]

#[macro_use]
extern crate log;
extern crate rand;

#[cfg(test)]
extern crate shine_testutils;

mod builder;
mod checker;
mod graph;
mod query;
mod tagging_locator;
mod triangulation;

pub mod geometry;
pub mod indexing;
pub mod types;

pub use self::builder::*;
pub use self::checker::*;
pub use self::graph::*;
pub use self::query::*;
pub use self::tagging_locator::*;
pub use self::triangulation::*;
