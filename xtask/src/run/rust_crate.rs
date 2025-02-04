/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/
 */
use std::process::Command;

use camino::Utf8PathBuf;
use clap::Args;

use anyhow::Result;
use ubrn_common::{run_cmd_quietly, CrateMetadata};

#[derive(Debug, Args)]
pub(crate) struct CrateArg {
    /// The path to the crate
    #[clap(long = "crate")]
    pub(crate) crate_dir: Option<Utf8PathBuf>,

    /// Build for release
    #[clap(long, requires = "crate_dir", default_value = "false")]
    pub(crate) release: bool,

    /// Use a specific build profile
    ///
    /// This overrides the release flag if both are specified.
    #[clap(long, requires = "crate_dir", conflicts_with_all = ["release"])]
    pub(crate) profile: Option<String>,

    /// Do not invoke cargo build.
    ///
    /// This is useful when invoking from within a test.
    #[clap(long, requires = "crate_dir", conflicts_with_all = ["clean"], default_value = "false")]
    pub(crate) no_cargo: bool,
}

impl CrateArg {
    pub(crate) fn cargo_build(&self, clean: bool) -> Result<CrateMetadata> {
        let metadata = CrateMetadata::try_from(self.crate_dir.clone().expect("crate has no path"))?;
        let lib_path = metadata.library_path(None, self.profile());
        if lib_path.exists() && clean {
            metadata.cargo_clean()?;
        }
        if !lib_path.exists() || !self.no_cargo {
            cargo_build(&metadata, self.profile())?;
        }
        Ok(metadata)
    }

    pub(crate) fn profile(&self) -> &str {
        CrateMetadata::profile(self.profile.as_deref(), self.release)
    }
}

fn cargo_build(metadata: &CrateMetadata, profile: &str) -> Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.current_dir(metadata.crate_dir());
    cmd.arg("build");
    if profile != "debug" {
        cmd.arg("--profile").arg(profile);
    }
    run_cmd_quietly(&mut cmd)?;
    Ok(())
}
