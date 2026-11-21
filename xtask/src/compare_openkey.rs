//! `cargo xtask compare-openkey`: compares the typing engine with `OpenKey` as a black box, through a tool in `tools/`
//! that is its own Cargo project outside the workspace (ADR-0014).

use anyhow::{Result, bail};

/// `args` are handed to the comparison tool once it exists; until then the command fails and says why.
pub(crate) fn run(args: &[String]) -> Result<bool> {
    let _ = args;
    bail!("the comparison tool is not built yet")
}

#[cfg(test)]
mod tests;
