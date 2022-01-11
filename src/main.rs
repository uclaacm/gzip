use std::{env, path::PathBuf};

use clap::{Parser, ErrorKind, IntoApp};

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
    #[clap(short = '1', long, conflicts_with = "best")]
    fast: bool,

    /// Compress better
    #[clap(short = '9', long, conflicts_with = "fast")]
    best: bool,

    /// Set compression level
    #[clap(skip)]
    compression_level: usize,

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
    // Parse and validate arguments.
    let (level, cleaned_args) = clean_args(env::args());
    let mut args = Args::parse_from(cleaned_args.iter());
    args.compression_level = match (args.fast, args.best, level) {
        (_, true, _) => 9,
        (_, _, Some(0)) => {
            println!("{}", Args::into_app().error(ErrorKind::ArgumentNotFound, "invalid option -- '0'"));
            return;
        }
        (_, _, Some(d)) => d,
        (true, _, None) => 1,
        _ => DEFAULT_COMPRESSION_LEVEL,
    };
}

/// Consume an iterator of arguments into the final level flag `(-1, -2, ..., -9)`, if
/// present, and its arguments after removal.
///
/// For example, `["-1dc3", "--fast"]` becomes `(Some(3), ["-dc", "--fast"])`.
/// 
/// This function makes no guarantee as to whether the returned level flag is valid in
/// the program context. In the case of gzip, -0 is invalid, and must be handled
/// separately.
fn clean_args<I: IntoIterator<Item = String>>(args: I) -> (Option<usize>, Vec<String>) {
    let (mut level, mut cleaned_args) = (None, vec![]);
    args.into_iter().for_each(|s| {
        let mut cleaned = String::new();
        if s.starts_with("-") && !s.starts_with("--") {
            s.chars().for_each(|c| {
                if c == '=' {
                    return;
                }
                if let Some(d) = c.to_digit(10) {
                    level = Some(d as usize);
                } else {
                    cleaned.push(c);
                }
            });
            if !["", "-"].contains(&cleaned.as_str()) {
                cleaned_args.push(cleaned);
            }
        } else {
            cleaned_args.push(s.clone());
        }
    });
    (level, cleaned_args)
}

#[cfg(test)]
mod test {
    use crate::clean_args;

    #[test]
    fn test_clean_args() {
        let (level, args) = clean_args("--test -cat".split(" ").map(|s| s.to_string()));
        assert_eq!(level, None);
        assert_eq!(args, vec!["--test", "-cat"]);
    }
}
