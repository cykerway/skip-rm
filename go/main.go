//  ::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
//  skip important files and dirs during rm;
//  ::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::

package main

import (
    "bufio"
    "encoding/json"
    "errors"
    "fmt"
    "log"
    "os"
    "os/exec"
    "path/filepath"
    "regexp"
    "strings"
)

//  decoded json config;
var conf struct {
    Command   string `json:"command"`
    Matcher   string `json:"matcher"`
    Mode      string `json:"mode"`
    Blacklist string `json:"blacklist"`
    Whitelist string `json:"whitelist"`
}

//  read first available config file;
func readConfig() {
    //  config files (in search order);
    var confFiles = []string{
        "~/.config/skip-rm/skip-rm.conf",
        "/etc/skip-rm/skip-rm.conf",
    }
    for _, confFile := range confFiles {
        data, err := os.ReadFile(expanduser(confFile))
        if err != nil {
            continue
        }
        err = json.Unmarshal(data, &conf)
        if err != nil {
            log.Fatal(err)
        }
        return
    }
    log.Fatal(errors.New("No config file"))
}

//  expand tilde in path; only works with current user (ie: `~` not `~other`);
func expanduser(path string) string {
    if path == "~" {
        home, err := os.UserHomeDir()
        if err != nil {
            log.Fatal(err)
        }
        path = home
    } else if strings.HasPrefix(path, "~/") {
        home, err := os.UserHomeDir()
        if err != nil {
            log.Fatal(err)
        }
        path = filepath.Join(home, path[2:])
    }
    return path
}

//  convert glob pattern to regex pattern; supports `globstar` (`**`); does not
//  support `extglob`; character classes `[:class:]` within `[]` are not tested;
func glob2regex(globpat string) string {
    pat := []rune(globpat)
    var ans strings.Builder
    for i, n := 0, len(pat); i < n; i++ {
        c := pat[i]
        switch {
        case c == '?':
            ans.WriteString(`[^/]`)
        case c == '*':
            if i+1 < n && pat[i+1] == '*' {
                ans.WriteString(`.*`)
                i++
            } else {
                ans.WriteString(`[^/]*`)
            }
        case c == '[':
            j := i + 1
            for j < n && pat[j] != ']' {
                j++
            }
            if j >= n {
                ans.WriteString(`\[`)
            } else {
                if pat[i+1] == '!' {
                    pat[i+1] = '^'
                }
                s := strings.ReplaceAll(string(pat[i+1:j]), `\`, `\\`)
                ans.WriteString(`[` + s + `]`)
                i = j
            }
        default:
            if strings.ContainsRune(`/()[]{}?*+-|^$\.&~# `, c) {
                ans.WriteRune('\\')
            }
            ans.WriteRune(c)
        }
    }
    return ans.String()
}

//  return true iff input filename matches a pattern; the pattern is not given
//  as argument but defined in closure; use `makeMatcher` to create a `Matcher`;
type Matcher func(fname string) bool

//  make a matcher function for given pattern; the pattern is interpreted with
//  the config `matcher` option;
func makeMatcher(pattern string) Matcher {
    switch conf.Matcher {
    case "string":
        return func(fname string) bool {
            fname, err := filepath.Abs(fname)
            if err != nil {
                log.Fatal(err)
            }
            return fname == pattern
        }
    case "glob":
        re := regexp.MustCompile("^" + glob2regex(expanduser(pattern)) + "$")
        return func(fname string) bool {
            fname, err := filepath.Abs(fname)
            if err != nil {
                log.Fatal(err)
            }
            return re.MatchString(fname)
        }
    case "regex":
        re := regexp.MustCompile("^" + pattern + "$")
        return func(fname string) bool {
            fname, err := filepath.Abs(fname)
            if err != nil {
                log.Fatal(err)
            }
            return re.MatchString(fname)
        }
    default:
        log.Fatal(conf.Matcher)
    }
    return nil
}

//  make matchers for all patterns in given list file; the list file contains
//  one pattern on each line;
func makeMatchers(list string) []Matcher {
    fio, err := os.Open(expanduser(list))
    if err != nil {
        log.Fatal(err)
    }
    matchers := []Matcher{}
    scan := bufio.NewScanner(fio)
    for scan.Scan() {
        matchers = append(matchers, makeMatcher(scan.Text()))
    }
    fio.Close()
    return matchers
}

//  main function;
func main() {
    //  read config;
    readConfig()

    //  filenames to proc;
    fnames := []string{}
    //  proc a file;
    proc := func(fname string) {
        fnames = append(fnames, fname)
    }
    //  skip a file;
    skip := func(fname string) {
        fmt.Printf("skipping %s...\n", fname)
    }

    switch conf.Mode {
    case "blacklist":
        matchers := makeMatchers(conf.Blacklist)
        match := func(fname string) bool {
            for _, matcher := range matchers {
                if matcher(fname) {
                    return true
                }
            }
            return false
        }
        ddash := false
        for _, fname := range os.Args[1:] {
            switch {
            case fname == "-":
                if match(fname) {
                    skip(fname)
                } else {
                    proc(fname)
                }
            case fname == "--":
                ddash = true
                proc(fname)
            case strings.HasPrefix(fname, "-"):
                if ddash && match(fname) {
                    skip(fname)
                } else {
                    proc(fname)
                }
            default:
                if match(fname) {
                    skip(fname)
                } else {
                    proc(fname)
                }
            }
        }
    case "whitelist":
        matchers := makeMatchers(conf.Whitelist)
        match := func(fname string) bool {
            for _, matcher := range matchers {
                if matcher(fname) {
                    return true
                }
            }
            return false
        }
        ddash := false
        for _, fname := range os.Args[1:] {
            switch {
            case fname == "-":
                if !match(fname) {
                    skip(fname)
                } else {
                    proc(fname)
                }
            case fname == "--":
                ddash = true
                proc(fname)
            case strings.HasPrefix(fname, "-"):
                if ddash && !match(fname) {
                    skip(fname)
                } else {
                    proc(fname)
                }
            default:
                if !match(fname) {
                    skip(fname)
                } else {
                    proc(fname)
                }
            }
        }
    default:
        log.Fatal(conf.Mode)
    }

    //  run command and pipe its stdout/stderr;
    cmd := exec.Command(conf.Command, fnames...)
    cmd.Stdout = os.Stdout
    cmd.Stderr = os.Stderr
    err := cmd.Run()
    if err != nil {
        os.Exit(err.(*exec.ExitError).ExitCode())
    }
}
