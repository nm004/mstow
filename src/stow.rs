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

use crate::{List, SourceTargetDiff};

type PathMap = HashMap<PathBuf, PathBuf>;

// target -> source
pub struct StowList(PathMap);

impl IntoIterator for StowList {
    type Item = <PathMap as IntoIterator>::Item;
    type IntoIter = <PathMap as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl StowList {
    pub fn new(source: &Path, target: &Path) -> Result<Self, io::Error> {
        let mut s = Self(HashMap::new());
        s.init(source, target)?;
        Ok(s)
    }
}

impl List for StowList {
    fn reserve(&mut self, s: usize) {
        self.0.reserve(s)
    }

    fn update(
        &mut self,
        source: &Path,
        target: &Path,
        diff: &SourceTargetDiff,
    ) -> Result<(), io::Error> {
        debug!("StowList::update_: source = {}", source.to_string_lossy());
        debug!("StowList::update_: target = {}", target.to_string_lossy());
        match (target.exists(), target.is_dir()) {
            (false, _) => {
                let r = self.0.insert(target.into(), diff.conv_source_rel(source));
                if let None = r {
                    return Ok(());
                }
                let e = io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    format!(
                        "Conflict source files {} & {}.",
                        self.0.get(target).unwrap().to_string_lossy(),
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

                self.traverse_fs(source, target, diff)
            }
        }
    }
}
