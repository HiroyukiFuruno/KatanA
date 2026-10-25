use std::io::Read;

const READ_CHUNK_BYTES: usize = 64 * 1024;

pub(super) struct ReadCancellation<F> {
    cancelled: F,
}

impl<F: Fn() -> bool> ReadCancellation<F> {
    pub(super) fn new(cancelled: F) -> Self {
        Self { cancelled }
    }

    pub(super) fn check(&self) -> std::io::Result<()> {
        if (self.cancelled)() {
            /* WHY: Interruptedはread_to_endが再試行するため、取消では再試行させない。 */
            Err(std::io::Error::other("local preview request was cancelled"))
        } else {
            Ok(())
        }
    }
}

pub(super) struct CancellableReader<'a, R, F> {
    reader: R,
    cancellation: &'a ReadCancellation<F>,
}

impl<'a, R, F> CancellableReader<'a, R, F> {
    pub(super) fn new(reader: R, cancellation: &'a ReadCancellation<F>) -> Self {
        Self {
            reader,
            cancellation,
        }
    }
}

impl<R: Read, F: Fn() -> bool> Read for CancellableReader<'_, R, F> {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        self.cancellation.check()?;
        let length = buffer.len().min(READ_CHUNK_BYTES);
        let read = self.reader.read(&mut buffer[..length])?;
        /* WHY: 停止中のOS読込は中断せず、復帰後に次の読込と処理への進行を止める。 */
        self.cancellation.check()?;
        Ok(read)
    }
}

#[cfg(test)]
mod tests {
    use super::{CancellableReader, READ_CHUNK_BYTES, ReadCancellation};
    use std::cell::Cell;
    use std::io::Read;

    #[test]
    fn real_file_reads_are_bounded_and_preserve_bytes() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let path = directory.path().join("input.bin");
        let bytes = vec![b'x'; READ_CHUNK_BYTES * 2];
        std::fs::write(&path, &bytes).expect("fixture write");
        let cancellation = ReadCancellation::new(|| false);
        let mut reader = CancellableReader::new(
            std::fs::File::open(&path).expect("fixture open"),
            &cancellation,
        );
        let mut buffer = vec![0; bytes.len()];
        assert_eq!(
            reader.read(&mut buffer).expect("bounded read"),
            READ_CHUNK_BYTES
        );
        let mut actual = buffer[..READ_CHUNK_BYTES].to_vec();
        reader.read_to_end(&mut actual).expect("complete read");
        assert_eq!(actual, bytes);
    }

    #[test]
    fn cancellation_before_and_after_real_read_does_not_retry() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let path = directory.path().join("input.bin");
        std::fs::write(&path, b"content").expect("fixture write");
        for initially_cancelled in [true, false] {
            let checks = Cell::new(0);
            let cancellation = ReadCancellation::new(|| {
                let checked = checks.get();
                checks.set(checked + 1);
                initially_cancelled || checked > 0
            });
            let mut reader = CancellableReader::new(
                std::fs::File::open(&path).expect("fixture open"),
                &cancellation,
            );
            let error = reader
                .read_to_end(&mut Vec::new())
                .expect_err("cancelled read");
            assert_eq!(error.kind(), std::io::ErrorKind::Other);
            assert_eq!(checks.get(), if initially_cancelled { 1 } else { 2 });
        }
    }

    #[test]
    fn cancellation_and_real_io_errors_remain_errors() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let path = directory.path().join("input.bin");
        let cancellation = ReadCancellation::new(|| false);
        let mut reader = CancellableReader::new(
            std::fs::File::create(&path).expect("write-only fixture"),
            &cancellation,
        );
        assert!(reader.read(&mut [0]).is_err());
    }
}
