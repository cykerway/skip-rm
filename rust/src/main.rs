//  ::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
//  skip important files and dirs during rm;
//  ::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::

use expanduser as eu;
use path_absolutize::*;
use regex::Regex;
use serde::Deserialize;
use serde::Serialize;
use std::env;
use std::fs;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

//  ============================================================================
//  config;
//  ============================================================================

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    command: String,
    matcher: String,
    mode: String,
    blacklist: String,
    whitelist: String,
}

//  read config from file;
fn read_config() -> Config {
    //  config files (in search order);
    let config_files = [
        r"~/.config/skip-rm/skip-rm.conf",
        r"/etc/skip-rm/skip-rm.conf",
    ];
    //  use first available config file;
    for config_file in config_files {
        let config_file = match eu::expanduser(config_file) {
            Ok(x) => x,
            Err(_) => continue,
        };
        let config_data = match fs::read_to_string(config_file) {
            Ok(x) => x,
            Err(_) => continue,
        };
        return serde_json::from_str(&config_data).unwrap();
    }
    panic!("no config file");
}

//  ============================================================================
//  matcher;
//  ============================================================================

enum Matcher {
    Str { pattern: String },
    Glob { pattern: Regex },
    Regex { pattern: Regex },
}

impl Matcher {
    //  return true iff input filename matches the pattern defined inside this
    //  `Matcher`; use `make_matcher` to create a `Matcher`;
    fn is_match(&self, fname: &str) -> bool {
        //  `std::path::absolute` has a weird impl that keeps `..`; so we have
        //  to use an alternate impl such as `path_absolutize`;
        let p = Path::new(fname);
        let p = p.absolutize().unwrap();
        let fname = p.to_str().unwrap();
        match self {
            Matcher::Str { pattern } => pattern == fname,
            Matcher::Glob { pattern } => pattern.is_match(fname),
            Matcher::Regex { pattern } => pattern.is_match(fname),
        }
    }
}

//  convert glob pattern to regex pattern; supports `globstar` (`**`); does not
//  support `extglob`; character classes `[:class:]` within `[]` are not tested;
fn glob2regex(globpat: &str) -> String {
    let mut pat = globpat.chars().collect::<Vec<_>>();
    let mut ans = String::new();
    let mut i = 0;
    let n = pat.len();
    while i < n {
        let c = pat[i];
        match c {
            '?' => {
                ans.push_str(r"[^/]");
            }
            '*' => {
                if i + 1 < n && pat[i + 1] == '*' {
                    ans.push_str(r".*");
                    i += 1;
                } else {
                    ans.push_str(r"[^/]*");
                }
            }
            '[' => {
                let mut j = i + 1;
                while j < n && pat[j] != ']' {
                    j += 1;
                }
                if j >= n {
                    ans.push_str(r"\[");
                } else {
                    if pat[i + 1] == '!' {
                        pat[i + 1] = '^';
                    }
                    let s: String = pat[i + 1..j].iter().collect();
                    ans.push_str(r"[");
                    ans.push_str(s.replace(r"\", r"\\").as_str());
                    ans.push_str(r"]");
                    i = j;
                }
            }
            _ => {
                //  here this string literal does not contain `/` because rust
                //  regex engine does not allow escaping `/`; this seems to be
                //  in contrast to many other languages;
                if r"()[]{}?*+-|^$\.&~# ".contains(c) {
                    ans.push('\\');
                }
                ans.push(c);
            }
        }
        i += 1;
    }
    ans
}

//  make a matcher function for given pattern; the pattern is interpreted with
//  the config `matcher` option;
fn make_matcher(config: &Config, pattern: String) -> Matcher {
    match config.matcher.as_str() {
        "string" => Matcher::Str { pattern },
        "glob" => {
            let pattern = eu::expanduser(&pattern).unwrap();
            let re = format!("^{}$", glob2regex(pattern.to_str().unwrap()));
            Matcher::Glob {
                pattern: Regex::new(re.as_str()).unwrap(),
            }
        }
        "regex" => {
            let re = format!("^{}$", pattern);
            Matcher::Regex {
                pattern: Regex::new(re.as_str()).unwrap(),
            }
        }
        x => panic!("invalid pattern: {}", x),
    }
}

//  make matchers for all patterns in given list file; the list file contains
//  one pattern on each line;
fn make_matchers(config: &Config, list_file: PathBuf) -> Vec<Matcher> {
    let mut matchers = vec![];
    let fio = File::open(list_file).unwrap();
    for line in BufReader::new(fio).lines() {
        matchers.push(make_matcher(config, line.unwrap()));
    }
    matchers
}

//  ============================================================================
//  filter;
//  ============================================================================

//  return true iff input filename matches any of these matchers;
fn match_any(matchers: &Vec<Matcher>, fname: &str) -> bool {
    for matcher in matchers {
        if matcher.is_match(fname) {
            return true;
        }
    }
    false
}

//  filter args by blacklist;
fn black_args(config: &Config, args: env::Args) -> Vec<String> {
    let list_file = eu::expanduser(&config.blacklist).unwrap();
    let matchers = make_matchers(&config, list_file);
    //  filter fnames by matchers;
    let mut fnames = vec![];
    let mut proc = |arg| {
        fnames.push(arg);
    };
    let skip = |arg| {
        eprintln!("skipping {}...", arg);
    };
    let mut ddash = false;
    for arg in args.skip(1) {
        match arg.as_str() {
            "-" => {
                if match_any(&matchers, &arg) {
                    skip(arg);
                } else {
                    proc(arg);
                }
            }
            "--" => {
                ddash = true;
                proc(arg);
            }
            x if x.starts_with("-") => {
                if ddash && match_any(&matchers, &arg) {
                    skip(arg);
                } else {
                    proc(arg);
                }
            }
            _ => {
                if match_any(&matchers, &arg) {
                    skip(arg);
                } else {
                    proc(arg);
                }
            }
        }
    }
    fnames
}

//  filter args by whitelist;
fn white_args(config: &Config, args: env::Args) -> Vec<String> {
    let list_file = eu::expanduser(&config.whitelist).unwrap();
    let matchers = make_matchers(&config, list_file);
    //  filter fnames by matchers;
    let mut fnames = vec![];
    let mut proc = |arg| {
        fnames.push(arg);
    };
    let skip = |arg| {
        eprintln!("skipping {}...", arg);
    };
    let mut ddash = false;
    for arg in args.skip(1) {
        match arg.as_str() {
            "-" => {
                if !match_any(&matchers, &arg) {
                    skip(arg);
                } else {
                    proc(arg);
                }
            }
            "--" => {
                ddash = true;
                proc(arg);
            }
            x if x.starts_with("-") => {
                if ddash && !match_any(&matchers, &arg) {
                    skip(arg);
                } else {
                    proc(arg);
                }
            }
            _ => {
                if !match_any(&matchers, &arg) {
                    skip(arg);
                } else {
                    proc(arg);
                }
            }
        }
    }
    fnames
}

//  ============================================================================
//  main;
//  ============================================================================

fn main() {
    //  read config;
    let config = read_config();
    //  filter args;
    let args = match config.mode.as_str() {
        "blacklist" => black_args(&config, env::args()),
        "whitelist" => white_args(&config, env::args()),
        x => panic!("invalid mode: {}", x),
    };
    //  call rm with filtered args;
    Command::new(config.command)
        .args(args)
        .status()
        .expect("failed to execute process");
}

//  ============================================================================
//  test;
//  ============================================================================

#[test]
fn test_expanduser() {
    let i = r"~/.local";
    let o = eu::expanduser(i).unwrap();
    assert!(o.starts_with(r"/home"));
    assert!(o.ends_with(r".local"));
}

#[test]
fn test_glob2regex() {
    let tcases = vec![
        (r"*", r"[^/]*"), //  input, output
        (r"**", r".*"),
        (r"*.[ch]", r"[^/]*\.[ch]"),
        (r"?", r"[^/]"),
        (r"[!a-z]", r"[^a-z]"),
        (r"[^a-z]", r"[^a-z]"),
        (r"[", r"\["),
    ];
    for (tinp, texp) in tcases {
        let tout = glob2regex(tinp);
        assert_eq!(tout.as_str(), texp);
    }
}

#[test]
fn test_path_absolutize() {
    let tcases = vec![
        (PathBuf::from(r"/"), PathBuf::from(r"/")),
        (PathBuf::from(r"/bin"), PathBuf::from(r"/bin")),
        (PathBuf::from(r"/bin/ls"), PathBuf::from(r"/bin/ls")),
        (PathBuf::from(r"."), env::current_dir().unwrap()),
        (
            PathBuf::from(r"/foo/bar/../baz"),
            PathBuf::from(r"/foo/baz"),
        ),
        (
            PathBuf::from(r"/home/nobody"),
            PathBuf::from(r"/home/nobody"),
        ),
        (
            PathBuf::from(r"/home/nobody/.bashrc"),
            PathBuf::from(r"/home/nobody/.bashrc"),
        ),
        (
            PathBuf::from(r"./foo"),
            env::current_dir().unwrap().join(r"foo"),
        ),
        (
            PathBuf::from(r"foo/bar/../baz"),
            env::current_dir().unwrap().join(r"foo/baz"),
        ),
        (
            PathBuf::from(r"./foo/bar/../baz"),
            env::current_dir().unwrap().join(r"foo/baz"),
        ),
    ];
    for (tinp, texp) in tcases {
        let tout = tinp.absolutize().unwrap();
        assert_eq!(tout.to_str().unwrap(), texp.to_str().unwrap());
    }
}
