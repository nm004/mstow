/*
 * Mstow - Miyamori's minimal GNU Stow implementation
 *
 * Copyright (c) 2022 Nozomi Miyamori
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of this
 * software and associated documentation files (the "Software"), to deal in the Software
 * without restriction, including without limitation the rights to use, copy, modify,
 * merge, publish, distribute, sublicense, and/or sell copies of the Software, and to
 * permit persons to whom the Software is furnished to do so.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED,
 * INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
 * PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT
 * HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
 * OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
 * SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */

use log::debug;
use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};

use crate::{DirTraversal, SourceTargetDiff};

// target -> source
type PathMap = HashMap<PathBuf, PathBuf>;

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
        debug!("StowList::update_: source = {}", source.to_string_lossy());
        debug!("StowList::update_: target = {}", target.to_string_lossy());
        match (target.exists(), target.is_dir()) {
            (false, _) => {
                let r = self.insert(target.into(), diff.conv_source_rel(source));
                if let None = r {
                    return Ok(());
                }
                let e = io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    format!(
                        "Conflict source files {} & {}.",
                        self.get(target).unwrap().to_string_lossy(),
                        r.unwrap().to_string_lossy()
                    ),
                );
                Err(e)
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
