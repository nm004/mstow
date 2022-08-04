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
        self.reserve_exact(s);
    }

    fn update_state_(
        &mut self,
        source: &Path,
        target: &Path,
        diff: &SourceTargetDiff,
    ) -> Result<(), io::Error> {
        debug!("UnstowList::update_: source = {}", source.to_string_lossy());
        debug!("UnstowList::update_: target = {}", target.to_string_lossy());
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
