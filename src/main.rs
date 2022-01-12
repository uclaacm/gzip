use std::path::PathBuf;

use clap::Parser;

const DEFAULT_COMPRESSION_LEVEL: usize = 6;

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
    #[clap(
        name = "level_1",
        short = '1',
        long = "fast",
        overrides_with_all = &[
            "level_1",
            "level_2",
            "level_3",
            "level_4",
            "level_5",
            "level_6",
            "level_7",
            "level_8",
            "level_9"
        ],
    )]
    level_1: bool,

    #[clap(
        name = "level_2",
        short = '2',
        overrides_with_all = &[
            "level_1",
            "level_2",
            "level_3",
            "level_4",
            "level_5",
            "level_6",
            "level_7",
            "level_8",
            "level_9"
        ],
        hide = true
    )]
    level_2: bool,

    #[clap(
        name = "level_3",
        short = '3',
        overrides_with_all = &[
            "level_1",
            "level_2",
            "level_3",
            "level_4",
            "level_5",
            "level_6",
            "level_7",
            "level_8",
            "level_9"
        ],
        hide = true
    )]
    level_3: bool,

    #[clap(
        name = "level_4",
        short = '4',
        overrides_with_all = &[
            "level_1",
            "level_2",
            "level_3",
            "level_4",
            "level_5",
            "level_6",
            "level_7",
            "level_8",
            "level_9"
        ],
        hide = true
    )]
    level_4: bool,

    #[clap(
        name = "level_5",
        short = '5',
        overrides_with_all = &[
            "level_1",
            "level_2",
            "level_3",
            "level_4",
            "level_5",
            "level_6",
            "level_7",
            "level_8",
            "level_9"
        ],
        hide = true
    )]
    level_5: bool,

    #[clap(
        name = "level_6",
        short = '6',
        overrides_with_all = &[
            "level_1",
            "level_2",
            "level_3",
            "level_4",
            "level_5",
            "level_6",
            "level_7",
            "level_8",
            "level_9"
        ],
        hide = true
    )]
    level_6: bool,

    #[clap(
        name = "level_7",
        short = '7',
        overrides_with_all = &[
            "level_1",
            "level_2",
            "level_3",
            "level_4",
            "level_5",
            "level_6",
            "level_7",
            "level_8",
            "level_9"
        ],
        hide = true
    )]
    level_7: bool,

    #[clap(
        name = "level_8",
        short = '8',
        overrides_with_all = &[
            "level_1",
            "level_2",
            "level_3",
            "level_4",
            "level_5",
            "level_6",
            "level_7",
            "level_8",
            "level_9"
        ],
        hide = true
    )]
    level_8: bool,

    /// Compress better
    #[clap(
        name = "level_9",
        short = '9',
        long = "best",
        multiple_occurrences = true,
        overrides_with_all = &[
            "level_1",
            "level_2",
            "level_3",
            "level_4",
            "level_5",
            "level_6",
            "level_7",
            "level_8",
            "level_9"
        ],
    )]
    level_9: bool,

    /// Use suffix SUF on compressed files
    #[clap(short = 'S', long, default_value = "")]
    suffix: String,

    /// Make rsync-friendly archive
    #[clap(long)]
    rsyncable: bool,

    /// Synchronous output (safer if system crashes, but slower)
    #[clap(long)]
    synchronous: bool,

    #[clap(value_name = "FILE")]
    files: Vec<PathBuf>,
}

fn main() {
    let _args = Args::parse();
}
