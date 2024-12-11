use std::io::SeekFrom;

use async_spooled_tempfile::{SpooledData, SpooledTempFile};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

#[tokio::test]
async fn test_implicit_rollover() {
    let mut sf = SpooledTempFile::new(5);
    let mut read_buffer = Vec::new();

    assert!(!sf.is_rolled());
    assert!(!sf.is_poisoned());
    assert!(std::matches!(sf.stream_position().await, Ok(0)));
    sf.read_to_end(&mut read_buffer).await.unwrap();
    assert_eq!(read_buffer.len(), 0);

    let first_data_written = b"1234";
    sf.write_all(first_data_written).await.unwrap();

    assert!(!sf.is_rolled());
    assert!(!sf.is_poisoned());
    let res_cur_pos = sf.stream_position().await;
    assert!(std::matches!(res_cur_pos, Ok(4)));
    let cur_pos = res_cur_pos.unwrap();
    sf.seek(SeekFrom::Start(0)).await.unwrap();
    sf.read_to_end(&mut read_buffer).await.unwrap();
    assert_eq!(read_buffer.as_slice(), first_data_written);
    read_buffer.clear();

    sf.seek(SeekFrom::Start(cur_pos)).await.unwrap();

    let second_data_written = b"5678";
    sf.write_all(second_data_written).await.unwrap();

    assert!(sf.is_rolled());
    assert!(!sf.is_poisoned());
    assert!(std::matches!(sf.stream_position().await, Ok(8)));

    sf.seek(SeekFrom::Start(0)).await.unwrap();
    sf.read_to_end(&mut read_buffer).await.unwrap();

    assert_eq!(
        read_buffer,
        [
            first_data_written.as_slice(),
            second_data_written.as_slice()
        ]
        .concat()
    );

    assert!(std::matches!(
        sf.into_inner().await,
        Ok(SpooledData::OnDisk(..))
    ));
}

#[tokio::test]
async fn test_explicit_rollover() {
    let mut sf = SpooledTempFile::new(5);
    let mut read_buffer = Vec::new();

    let first_data_written = b"123";
    sf.write_all(first_data_written).await.unwrap();

    assert!(!sf.is_rolled());
    assert!(!sf.is_poisoned());
    assert!(std::matches!(sf.stream_position().await, Ok(3)));

    sf.seek(SeekFrom::Start(0)).await.unwrap();

    sf.roll().await.unwrap();

    assert!(sf.is_rolled());
    assert!(!sf.is_poisoned());
    assert!(std::matches!(sf.stream_position().await, Ok(0)));

    sf.read_to_end(&mut read_buffer).await.unwrap();
    assert_eq!(read_buffer.as_slice(), first_data_written);
    assert!(std::matches!(
        sf.into_inner().await,
        Ok(SpooledData::OnDisk(..))
    ));
}

#[tokio::test]
async fn test_into_inner() {
    let mut sf1 = SpooledTempFile::new(10);

    let first_data_written = b"1234567";
    sf1.write_all(first_data_written).await.unwrap();

    assert!(!sf1.is_rolled());
    assert!(!sf1.is_poisoned());
    assert!(std::matches!(sf1.stream_position().await, Ok(7)));
    assert!(std::matches!(
        sf1.into_inner().await,
        Ok(SpooledData::InMemory(..))
    ));

    let mut sf2 = SpooledTempFile::with_max_size_and_capacity(10, 10);

    let second_data_written = b"123456789abcdef";
    sf2.write_all(second_data_written).await.unwrap();

    assert!(sf2.is_rolled());
    assert!(!sf2.is_poisoned());
    assert!(std::matches!(sf2.stream_position().await, Ok(15)));
    assert!(std::matches!(
        sf2.into_inner().await,
        Ok(SpooledData::OnDisk(..))
    ));
}

#[tokio::test]
async fn test_set_len() {
    let mut sf1 = SpooledTempFile::new(10);
    sf1.set_len(4).await.unwrap();

    let f1_buffer = match sf1.into_inner().await {
        Ok(SpooledData::InMemory(cur)) => cur.into_inner(),
        _ => panic!("the data of the first spooled file should be in memory"),
    };
    assert_eq!(f1_buffer.as_slice(), b"\x00\x00\x00\x00");

    let mut sf2 = SpooledTempFile::new(5);
    sf2.write_all(b"abc").await.unwrap();
    assert!(!sf2.is_rolled());
    assert!(!sf2.is_poisoned());
    sf2.set_len(10).await.unwrap();

    let mut sf2_file = match sf2.into_inner().await {
        Ok(SpooledData::OnDisk(file)) => file,
        _ => panic!("the data of the second spooled file should be located in a file"),
    };
    sf2_file.seek(SeekFrom::Start(0)).await.unwrap();
    let mut sf2_content = Vec::new();
    sf2_file.read_to_end(&mut sf2_content).await.unwrap();

    assert_eq!(sf2_content.as_slice(), b"abc\x00\x00\x00\x00\x00\x00\x00");
}
