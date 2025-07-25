// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Modifications Copyright Kani Contributors
// See GitHub history for details.

// Original copyright tokio contributors.
// origin: tokio/tests/tokio/ at commit b2ada60e701d5c9e6644cf8fc42a100774f8e23f

#![warn(rust_2018_idioms)]
#![cfg(feature = "full")]

use std::io;
use tokio::io::AsyncReadExt;
use tokio_test::assert_ok;
use tokio_test::io::Builder;

#[cfg(disabled)] // CBMC takes more than 15 minutes
#[kani::proof]
#[kani::unwind(12)]
async fn read_to_string() {
    let mut buf = String::new();
    let mut rd: &[u8] = b"hello world";

    let n = assert_ok!(rd.read_to_string(&mut buf).await);
    assert_eq!(n, 11);
    assert_eq!(buf[..], "hello world"[..]);
}

#[cfg(disabled)] // CBMC consumes more than 10 GB
#[kani::proof]
#[kani::unwind(2)]
async fn to_string_does_not_truncate_on_utf8_error() {
    let data = vec![0xff, 0xff, 0xff];

    let mut s = "abc".to_string();

    match AsyncReadExt::read_to_string(&mut data.as_slice(), &mut s).await {
        Ok(len) => panic!("Should fail: {} bytes.", len),
        Err(err) if err.to_string() == "stream did not contain valid UTF-8" => {}
        Err(err) => panic!("Fail: {}.", err),
    }

    assert_eq!(s, "abc");
}

#[cfg(disabled)] // CBMC consumes more than 10 GB
#[kani::proof]
#[kani::unwind(2)]
async fn to_string_does_not_truncate_on_io_error() {
    let mut mock = Builder::new()
        .read(b"def")
        .read_error(io::Error::new(io::ErrorKind::Other, "whoops"))
        .build();
    let mut s = "abc".to_string();

    match AsyncReadExt::read_to_string(&mut mock, &mut s).await {
        Ok(len) => panic!("Should fail: {} bytes.", len),
        Err(err) if err.to_string() == "whoops" => {}
        Err(err) => panic!("Fail: {}.", err),
    }

    assert_eq!(s, "abc");
}

#[cfg(disabled)] // CBMC consumes more than 10 GB
#[kani::proof]
#[kani::unwind(2)]
async fn to_string_appends() {
    let data = b"def".to_vec();

    let mut s = "abc".to_string();

    let len = AsyncReadExt::read_to_string(&mut data.as_slice(), &mut s).await.unwrap();

    assert_eq!(len, 3);
    assert_eq!(s, "abcdef");
}
