# skip-rm

skip important files and dirs during rm;

in the past, users have frequently reported success in deleting their home dir,
root dir, and much more; skip-rm helps prevent such accidental deletions using
filename checks; you specify which files to and not to delete and skip-rm does
the rest for you;

## install

this tool has several implementations:

-   `bash/README.md`;

-   `go/README.md`;

-   `rust/README.md`;

these implementations are independent, and you can choose any one of them; but
beware there may be minor differences among these implementations, which means
they may not give exactly the same behavior; double check the source code, and
make sure you understand what is happening before use; open an issue if needed;

the bash implementation has longer history and tends to be more stable; but it
is very slow when there are a lot of filenames; the go implementation is newer
and faster, but has a higher chance of malfunction; the rust implementation is
even newer and less mature; you are strongly advised to do some dry-run (using
fake `rm`) before deploying this tool; it is better safe than sorry;

these implementations share the same config files; a sample config is provided
in `config` dir; the entire config has a json file plus some lists of patterns;

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

