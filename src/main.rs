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
use mstow::{StowList, UnstowList};
use std::fs::remove_file;
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
    cli.source
        .iter()
        .enumerate()
        .for_each(|s| debug!("main: source[{}] = {}", s.0, s.1.to_string_lossy()));

    macro_rules! gen_list {
        ($list:ident) => {{
            let mut l = $list::new();
            for s in &cli.source {
                if let Err(e) = l.update(s, &cli.target) {
                    error!("Abort operation. No changes committed: {}", e);
                    return;
                };
            }
            l
        }};
    }

    info!("Begin operation.");
    if !cli.unstowing {
        for (ref t, ref s) in gen_list!(StowList) {
            info!("Stow: {} -> {}", t.to_string_lossy(), s.to_string_lossy());
            if let Err(e) = symlink(s, t) {
                warn!(
                    concat!("Failed to create target file {}: {}"),
                    t.to_string_lossy(),
                    e
                );
            }
        }
    } else {
        for ref t in gen_list!(UnstowList) {
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