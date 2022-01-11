use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(
    about = r"Compress or uncompress FILEs (by default, compress FILEs in-place).

Mandatory arguments to long options are mandatory for short options too.",
    version,
    after_help = r"
With no FILE, or when FILE is -, read standard input.

Report bugs to <bug-gzip@gnu.org>."
)]
struct Args {
    /// Convert end-of-lines to local OS conventions
    #[clap(short, long)]
    ascii: bool,

    /// Output to stdout
    #[clap(short = 'c', long = "stdout")]
    to_stdout: bool,

    /// Decompress
    #[clap(short, long)]
    decompress: bool,

    /// Don't ask questions, compress links
    #[clap(short, long)]
    force: bool,

    /// Keep (don't delete) input files
    #[clap(short, long)]
    keep: bool,

    /// Don't save or restore the original file name
    #[clap(short, long)]
    no_name: bool,

    /// Don't save or restore the original file time
    #[clap(short = 'm')]
    no_time: bool,

    /// Save or restore the original modification time
    #[clap(short = 'M', long)]
    time: bool,

    /// Recurse through directories
    #[clap(short, long)]
    recursive: bool,

    /// List the file contents
    #[clap(short, long)]
    list: bool,

    /// Be verbose
    #[clap(short, long)]
    verbose: bool,

    /// Be very quiet
    #[clap(short, long)]
    quiet: bool,

    /// Test .gz file integrity
    #[clap(short, long)]
    test: bool,

    /// Compress faster
    #[clap(short = '1', long)]
    fast: bool,

    /// Compress better
    #[clap(short = '9', long)]
    best: bool,

    #[clap(value_name = "FILE")]
    files: Vec<PathBuf>,
}

fn main() {
    let _args = Args::parse();
}
