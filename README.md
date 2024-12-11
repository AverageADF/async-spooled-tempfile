# async-spooled-tempfile

---

Crate exposing an asynchronous version of the [`tempfile::SpooledTempFile`](https://docs.rs/tempfile/latest/tempfile/struct.SpooledTempFile.html)
structure provided by the [tempfile](https://docs.rs/tempfile/latest/tempfile/index.html) crate.

## Dependency

Add the following line to your `Cargo.toml` file:

```toml
[dependencies]
async-spooled-tempfile = "0.1.0"
```

## Example

```rust
use async_spooled_tempfile::{SpooledData, SpooledTempFile};
use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() {
    let mut sf = SpooledTempFile::new(10);

    sf.write_all(b"12345").await.unwrap();
    assert!(!sf.is_rolled());

    sf.write_all(b"6789abc").await.unwrap();
    assert!(sf.is_rolled());

    assert!(std::matches!(
        sf.into_inner().await,
        Ok(SpooledData::OnDisk(_file))
    ));
}
```
