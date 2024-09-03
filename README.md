# Kyu

an alternative git client for the command line written in rust

this project is just for fun and nothing serious, use it at your own risk

## Features

this is basicaly a reimplementation of git in rust, so the commands mostly work the same with a few exceptions

currently implemented commands:

-   init
-   ls-tree <hash>
-   cat-file <type> <hash>
    cat file can also be used to print trees (like ls-tree)
-   hash-object <file> (-w)
-   log
-   checkout <hash> (<file/folder>)
    if checkout is used without <file/folder> it will act like file / folder is workdir instead of switching branches

## Credits

This whole project is heavily inspired by the wyag tutorial (https://wyag.thb.lt) which is a tutorial about implementing a git client in python
The idea itself is not mine, i just wanted to try to implement it in rust.
Also the concepts are from the tutorial and the complete know how on how git works (at least i learned it from there first)

So many thanks to the author of the great tutorial for the super interesting learning experience :)

---

also many thanks to the contributors of the crates i used for this project:

chrono
clap
colored
flate2
hex-literal
rust-ini
sha1

and of course thanks to the rust team for the great language

all rights of the tutorial, crates, rust belong to the respective owners / contributors

many thanks :)
