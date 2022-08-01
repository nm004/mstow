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
use pathdiff::diff_paths;
use std::collections::HashMap;
use std::io;
use std::iter::repeat;
use std::path::{Path, PathBuf};

macro_rules! update_recurse {
    ($member:ident.$reserve:ident) => {
        fn update_recurse(&mut self, source: &Path, target: &Path) -> Result<(), io::Error> {
            if !target.metadata()?.permissions().readonly() {
                self.$member.$reserve(source.read_dir()?.count());
                return source.read_dir().unwrap().try_for_each(|ent| {
                    let ent = ent?;
                    self.update_(&ent.path(), &target.join(PathBuf::from(ent.file_name())))
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
    };
}

type PathMap = HashMap<PathBuf, PathBuf>;

pub struct StowList {
    // target -> source
    hm: PathMap,
    d: Option<SourceTargetDiff>,
}

impl IntoIterator for StowList {
    type Item = <PathMap as IntoIterator>::Item;
    type IntoIter = <PathMap as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.hm.into_iter()
    }
}

impl StowList {
    pub fn new() -> Self {
        Self {
            hm: HashMap::with_capacity(1),
            d: None,
        }
    }

    pub fn update(&mut self, source: &Path, target: &Path) -> Result<(), io::Error> {
        self.d = Some(SourceTargetDiff::new(source, target));
        self.update_recurse(source, target)?;

        Ok(())
    }

    update_recurse!(hm.reserve);

    fn update_(&mut self, source: &Path, target: &Path) -> Result<(), io::Error> {
        debug!("StowList::update_: source = {}", source.to_string_lossy());
        debug!("StowList::update_: target = {}", target.to_string_lossy());
        match (target.exists(), target.is_dir()) {
            (false, _) => {
                let r = self.hm.insert(
                    target.into(),
                    self.d.as_ref().unwrap().conv_source_rel(source),
                );
                if let None = r {
                    return Ok(());
                }
                let e = io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    format!(
                        "Conflict source files {} & {}.",
                        self.hm.get(target).unwrap().to_string_lossy(),
                        r.unwrap().to_string_lossy()
                    ),
                );
                Err(e)
            }

            (true, false) => {
                if self.d.as_ref().unwrap().conv_source_rel(source) == target.read_link()? {
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

                self.update_recurse(source, target)
            }
        }
    }
}

pub struct UnstowList {
    v: Vec<PathBuf>,
    d: Option<SourceTargetDiff>,
}

impl IntoIterator for UnstowList {
    type Item = <Vec<PathBuf> as IntoIterator>::Item;
    type IntoIter = <Vec<PathBuf> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.v.into_iter()
    }
}

impl UnstowList {
    pub fn new() -> Self {
        Self { v: vec![], d: None }
    }

    pub fn update(&mut self, source: &Path, target: &Path) -> Result<(), io::Error> {
        self.d = Some(SourceTargetDiff::new(source, target));
        self.update_recurse(source, target)?;

        Ok(())
    }

    fn update_(&mut self, source: &Path, target: &Path) -> Result<(), io::Error> {
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

            (false, true) => self.update_recurse(source, target),

            (true, false) => {
                if self.d.as_ref().unwrap().conv_source_rel(source) == target.read_link()? {
                    self.v.push(target.into());
                    return Ok(());
                }
                Ok(())
            }

            (true, true) => {
                if source.canonicalize().unwrap() == target.canonicalize().unwrap() {
                    self.v.push(target.into());
                    return Ok(());
                }
                self.update_recurse(source, target)
            }
        }
    }

    update_recurse!(v.reserve_exact);
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
