// This program is in the public domain.

use log::debug;
use std::io;
use std::path::{Path, PathBuf};

use crate::{DirTraversal, SourceTargetDiff};

// target -> source
type PathMap = Vec<(PathBuf, PathBuf)>;

pub fn new_stow_list(source: &Path, target: &Path) -> Result<PathMap, io::Error> {
    let mut s = PathMap::new();
    s.init_(source, target)?;
    Ok(s)
}

impl DirTraversal for PathMap {
    fn reserve_cap_(&mut self, s: usize) {
        self.reserve(s)
    }

    fn update_state_(
        &mut self,
        source: &Path,
        target: &Path,
        diff: &SourceTargetDiff,
    ) -> Result<(), io::Error> {
        debug!("stow::DirTraversal::update_: source = {}", source.to_string_lossy());
        debug!("stow::DirTraversal::update_: target = {}", target.to_string_lossy());
        match (target.exists(), target.is_dir()) {
            (false, _) => {
                self.push((target.into(), diff.conv_source_rel(source)));
                Ok(())
            }

            (true, false) => {
                if diff.conv_source_rel(source) == target.read_link()? {
                    return Ok(());
                }
                let e = io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    format!("Target file {} already exists.", target.to_string_lossy()),
                );
                Err(e)
            }

            (true, true) => {
                if source.canonicalize().unwrap() == target.canonicalize().unwrap() {
                    return Ok(());
                }

                self.traverse_(source, target, diff)
            }
        }
    }
}
