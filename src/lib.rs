// This program is in the public domain.

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
