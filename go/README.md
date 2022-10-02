# skip-rm (go)

skip important files and dirs during rm;

in the past, users have frequently reported success in deleting their home dir,
root dir, and much more; skip-rm helps prevent such accidental deletions using
filename checks; you specify which files to and not to delete and skip-rm does
the rest for you;

## install

skip-rm has a single executable file and a few config files;

to build the executable file (named `skip-rm`):

    go build -o skip-rm .

the executable file shall be installed into a dir listed in envar `PATH`:

    cp skip-rm /usr/bin/

the config files shall be installed into either system or user config dir;
system config dir is `/etc/skip-rm`; user config dir is `~/.config/skip-rm`;
user config has higher priority;

to install into system config dir:

    cp -r config/skip-rm /etc/

to install into user config dir:

    cp -r config/skip-rm ~/.config/

## usage

to test with the default config (do not test with important files):

    skip-rm {files-to-remove}

if everything is good, alias `rm` to `skip-rm`:

    alias rm='skip-rm'

`skip-rm` is designed as a wrapper of and a drop-in replacement for `rm`; it is
recommened to alias `rm` to `skip-rm` in bashrc; then simply run `rm` as usual,
knowing important files and dirs will be protected from deletion;

## config

the main config file is `skip-rm.conf`, in json format:

    {
        "command": "/bin/rm",
        "matcher": "glob",
        "mode": "blacklist",
        "blacklist": "~/.config/skip-rm/black.list",
        "whitelist": "~/.config/skip-rm/white.list"
    }

available config options are:

-   *command*: the builtin rm command;

-   *matcher*: filename matcher:

    -   *string*: match string verbatim;

    -   *glob*: match with glob patterns:

        -   `?` matches any single char (excluding slash `/`);

        -   `*` matches any string (excluding slash `/`);

        -   `**` matches any string (including slash `/`);

        -   `[...]`: matches any one of the enclosed characters;

    -   *regex*: match with regular expressions;

-   *mode*: how to compare against a pre-defined list:

    -   *blacklist*: input filenames matching any pattern on the list are
        skipped; others are deleted;

    -   *whitelist*: input filenames matching any pattern on the list are
        deleted; others are skipped;

-   *blacklist*: blacklist file path; a leading tilde `~` is translated to home
    dir of current user;

-   *whitelist*: whitelist file path; a leading tilde `~` is translated to home
    dir of current user;

### blacklist and whitelist

blacklist and whitelist have the same format: each line specifies a pattern; a
leading tilde `~` in the pattern is translated to home dir of current user;

each input filename is first converted to realpath (without symlink expansion),
then compared with these patterns one by one, until a match is found or no match
is found; therefore, when you are writing patterns, expect them to be matched
against realpaths;

## depend

this implementation depends on [go][] and its standard library (1.18 or newer);

## license

Copyright (C) 2018-2022 Cyker Way

This program is free software: you can redistribute it and/or modify it under
the terms of the GNU General Public License as published by the Free Software
Foundation, either version 3 of the License, or (at your option) any later
version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY
WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with
this program. If not, see <https://www.gnu.org/licenses/>.

[go]: https://go.dev/
