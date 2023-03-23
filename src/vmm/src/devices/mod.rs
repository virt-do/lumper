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
        let s = String::from_utf8_lossy(buf);
        self.tx
            .send(s.to_string())
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Error sending data"))?;
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
