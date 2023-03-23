// SPDX-License-Identifier: Apache-2.0

use std::io::{Result, Write};
use std::sync::mpsc;

pub(crate) mod net;
pub(crate) mod serial;

pub struct Writer {
    tx: mpsc::Sender<String>,
}

impl Write for Writer {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        if buf.len() > 0 && (buf[0] != 10 && buf[0] != 13) {
            let s = String::from_utf8_lossy(buf).to_string();
            self.tx.send(s).map_err(|_| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Error while sending data to channel",
                )
            })?;
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Writer {
    pub fn new(tx: mpsc::Sender<String>) -> Self {
        Writer { tx }
    }
}
