[![Build Status](https://travis-ci.com/avitex/rust-zc.svg?branch=master)](https://travis-ci.com/avitex/rust-zc)
[![Coverage Status](https://codecov.io/gh/avitex/rust-zc/branch/master/graph/badge.svg?token=X2LXHI8VYL)](https://codecov.io/gh/avitex/rust-zc)
[![Crate](https://img.shields.io/crates/v/zc.svg)](https://crates.io/crates/zc)
[![Docs](https://docs.rs/zc/badge.svg)](https://docs.rs/zc)

# rust-zc

**Rust library for zero-copy data**  
Documentation hosted on [docs.rs](https://docs.rs/zc).

```toml
zc = "0.1"
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
        data.dependant::<StructWithBytes>(),
        &StructWithBytes(&[2, 3])
    )
}
```
