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

mod stow;
mod unstow;
pub use stow::new_stow_list;
pub use unstow::new_unstow_list;

use pathdiff::diff_paths;
use std::io;
use std::iter::repeat;
use std::path::{Path, PathBuf};

trait DirTraversal {
    fn init_(&mut self, source: &Path, target: &Path) -> Result<(), io::Error> {
        let diff = SourceTargetDiff::new(source, target);
        self.traverse_(source, target, &diff)
    }

    fn traverse_(
        &mut self,
        source: &Path,
        target: &Path,
        diff: &SourceTargetDiff,
    ) -> Result<(), io::Error> {
        if !target.metadata()?.permissions().readonly() {
            self.reserve_cap_(source.read_dir()?.count());
            return source.read_dir().unwrap().try_for_each(|ent| {
                let ent = ent?;
                self.update_state_(
                    &ent.path(),
                    &target.join(PathBuf::from(ent.file_name())),
                    diff,
                )
            });
        }

        let e = io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!(
                "Target dir {} is not writable (readonly).",
                target.to_string_lossy()
            ),
        );
        Err(e)
    }

    fn update_state_(
        &mut self,
        source: &Path,
        target: &Path,
        diff: &SourceTargetDiff,
    ) -> Result<(), io::Error>;

    fn reserve_cap_(&mut self, s: usize);
}

struct SourceTargetDiff {
    source_root: PathBuf,
    d: PathBuf,
}

impl SourceTargetDiff {
    fn new(source_root: &Path, target_root: &Path) -> Self {
        // TODO: use fs::absolute once API is stabilized
        let d = diff_paths(
            &source_root.canonicalize().unwrap(),
            &target_root.canonicalize().unwrap(),
        )
        .unwrap();
        Self {
            source_root: source_root.into(),
            d,
        }
    }

    // This makes source path relative to target path.
    fn conv_source_rel(&self, source: &Path) -> PathBuf {
        let s = source.strip_prefix(&self.source_root).unwrap();
        let p: PathBuf = repeat("..")
            .take(s.parent().unwrap().components().count())
            .collect();

        p.join(&self.d).join(s)
    }
}
