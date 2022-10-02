package main

import (
    "strings"
    "testing"
)

func TestExpandUser(t *testing.T) {
    i := "~/.local"
    o := expanduser(i)
    if !strings.HasPrefix(o, "/home") || !strings.HasSuffix(o, "/.local") {
        t.Fatalf("expanduser(%s):%s", i, o)
    }
}

func TestGlob2Regex(t *testing.T) {
    tcases := [][2]string{
        {`*`, `[^/]*`},             //  input, output
        {`**`, `.*`},
        {`*.[ch]`, `[^/]*\.[ch]`},
        {`?`, `[^/]`},
        {`[!a-z]`, `[^a-z]`},
        {`[^a-z]`, `[^a-z]`},
        {`[`, `\[`},
    }
    for _, tcase := range tcases {
        tinp := tcase[0]
        tout := glob2regex(tinp)
        texp := tcase[1]
        if tout != texp {
            t.Fatalf("glob2regex(%s):%s:%s", tinp, tout, texp)
        }
    }
}
