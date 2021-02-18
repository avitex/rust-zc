[![Build Status](https://github.com/avitex/rust-zc/workflows/build/badge.svg)](https://github.com/avitex/rust-zc/actions?query=workflow:build)
[![Coverage Status](https://codecov.io/gh/avitex/rust-zc/branch/master/graph/badge.svg?token=X2LXHI8VYL)](https://codecov.io/gh/avitex/rust-zc)
[![Crate](https://img.shields.io/crates/v/zc.svg)](https://crates.io/crates/zc)
[![Docs](https://docs.rs/zc/badge.svg)](https://docs.rs/zc)

# rust-zc

**Rust library providing `Zc` for self-referential zero-copy structures.**  
Documentation hosted on [docs.rs](https://docs.rs/zc).

```toml
zc = "0.4"
```

## Usage

```rust
use zc::Dependant;

#[derive(PartialEq, Debug, Dependant)]
pub struct StructWithBytes<'a>(&'a [u8]);

impl<'a> From<&'a [u8]> for StructWithBytes<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        Self(&bytes[1..])
    }
}

fn main() {
    let owner = vec![1, 2, 3];
    let data = zc::from!(owner, StructWithBytes, [u8]);

    assert_eq!(
        data.get::<StructWithBytes>(),
        &StructWithBytes(&[2, 3])
    )
}
```

## Testing

Run standard tests:

```sh
cargo test
```

Run miri tests:

```sh
cargo miri test --test test_zc
```
