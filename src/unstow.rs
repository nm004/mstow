// This program is in the public domain.

use log::debug;
use std::io;
use std::path::{Path, PathBuf};

use crate::{DirTraversal, SourceTargetDiff};

pub fn new_unstow_list(source: &Path, target: &Path) -> Result<Vec<PathBuf>, io::Error> {
    let mut s = vec![];
    s.init_(source, target)?;
    Ok(s)
}

impl DirTraversal for Vec<PathBuf> {
    fn reserve_cap_(&mut self, s: usize) {
        self.reserve(s);
    }

    fn update_state_(
        &mut self,
        source: &Path,
        target: &Path,
        diff: &SourceTargetDiff,
    ) -> Result<(), io::Error> {
        debug!("unstow::DirTraversal::update_: source = {}", source.to_string_lossy());
        debug!("unstow::DirTraversal::update_: target = {}", target.to_string_lossy());
        match (target.is_symlink(), target.is_dir()) {
            (false, false) => {
                if !target.exists() {
                    return Ok(());
                }
                let e = io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    format!(
                        "Target file {} is not symlink nor directory.",
                        target.to_string_lossy()
                    ),
                );
                Err(e)
            }

            (false, true) => self.traverse_(source, target, diff),

            (true, false) => {
                if diff.conv_source_rel(source) == target.read_link()? {
                    self.push(target.into());
                    return Ok(());
                }
                Ok(())
            }

            (true, true) => {
                if source.canonicalize().unwrap() == target.canonicalize().unwrap() {
                    self.push(target.into());
                    return Ok(());
                }
                self.traverse_(source, target, diff)
            }
        }
    }
}
