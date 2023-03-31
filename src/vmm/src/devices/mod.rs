// SPDX-License-Identifier: Apache-2.0

use std::io::{Result, Write};
use std::os::unix::net::UnixStream;

pub(crate) mod net;
pub(crate) mod serial;

pub struct Writer {
    unix_stream: UnixStream,
}

impl Write for Writer {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let s = String::from_utf8_lossy(buf).to_string();
        let _ = &self.unix_stream.write(s.as_bytes()).unwrap();

        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Writer {
    pub fn new(unix_stream: UnixStream) -> Self {
        Writer { unix_stream }
    }
}
