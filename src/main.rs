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

use clap::Parser;
use clap_verbosity_flag::{Verbosity, WarnLevel};
use log::{debug, error, info, warn};
use mstow::{new_stow_list, new_unstow_list};
use std::collections::HashMap;
use std::fs::remove_file;
use std::io;
use std::os::unix::fs::symlink;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(value_parser, multiple = true, required = true)]
    source: Vec<PathBuf>,

    #[clap(
        value_parser,
        short,
        required = true,
        help = "Target which (un)stowing to"
    )]
    target: PathBuf,

    #[clap(value_parser, short = 'D', help = "Do unstowing")]
    unstowing: bool,

    #[clap(flatten)]
    verbose: Verbosity<WarnLevel>,
}

fn main() {
    let cli = Cli::parse();
    if let Some(l) = cli.verbose.log_level_filter().to_level() {
        simple_logger::init_with_level(l).unwrap();
    };
    debug!("main: unstowing = {}", cli.unstowing);
    debug!("main: target = {}", cli.target.to_string_lossy());
    for (i, j) in cli.source.iter().enumerate() {
        debug!("main: source[{}] = {}", i, j.to_string_lossy());
    }

    if !cli.target.exists() {
        error!("Target directory does not exist");
        return;
    }

    for s in &cli.source {
        if !s.exists() {
            error!("Source directory {} does not exist", s.to_string_lossy());
            return;
        }
    }

    macro_rules! ok_or_abort {
        ($r:expr) => {
            if let Err(e) = $r {
                error!("Abort operation. No changes committed: {}", e);
                return;
            }
        };
    }

    info!("Begin operation.");
    if !cli.unstowing {
        let ll: Result<Box<_>, _> = cli
            .source
            .iter()
            .map(|s| new_stow_list(s, &cli.target))
            .collect();
        ok_or_abort!(ll);

        let ll = ll.unwrap();
        let mut ll = ll.iter().flatten();

        let maybe_not_conflicted = ll.clone().try_fold(HashMap::new(), |mut acc, i| {
            let r = acc.insert(i.0.clone(), i.1.clone());
            if let None = r {
                return Ok(acc);
            }
            let e = io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!(
                    "Conflict source files {} & {}.",
                    acc.get(&i.0).unwrap().to_string_lossy(),
                    r.unwrap().to_string_lossy()
                ),
            );
            Err(e)
        });
        ok_or_abort!(maybe_not_conflicted);

        for (ref t, ref s) in ll {
            info!("Stow: {} -> {}", t.to_string_lossy(), s.to_string_lossy());
            if let Err(e) = symlink(s, t) {
                warn!(
                    "Failed to create target file {}: {}",
                    t.to_string_lossy(),
                    e
                );
            }
        }
    } else {
        let ll: Result<Box<_>, _> = cli
            .source
            .iter()
            .map(|s| new_unstow_list(s, &cli.target))
            .collect();
        ok_or_abort!(ll);

        let ll = ll.unwrap();
        let ll = ll.iter().flatten();

        for ref t in ll {
            info!(
                "Unstow: {} -> {}",
                t.to_string_lossy(),
                t.read_link().unwrap().to_string_lossy()
            );
            if let Err(e) = remove_file(t) {
                warn!(
                    "Failed to remove target file {}: {}",
                    t.to_string_lossy(),
                    e
                );
            }
        }
    }
    info!("End operation.");
}
