# PO

`po` is a rust crate for reading and writing PO translation files.

## USAGE
To parse a `.po` or `.pot` file, just create a new `PoFile`
```Rust
let file = PoFile::new("<PATH>").unwrap();
```

To update the file `PoFile` was generated from, use `update()`
```Rust
let file = PoFile::new("<PATH>").unwrap();
file.update().unwrap();
```

To write the `PoFile` to another file, use `write(path)`
```Rust
let file = PoFile::new("<PATH>").unwrap();
file.write("<ANOTHER PATH>").unwrap();
```

To get the `PoFile` as the `String`, equal to the file, use `to_string()`
```Rust
let file = PoFile::new("<PATH>").unwrap();
let data = file.to_string();
// Content in `data` will be the same as the one written using `write` or `update`.
```
