//use bitbit::{BitReader, BitWriter, MSB};
use slice_deque::SliceDeque;
use std::cmp;
use std::io::{Error, ErrorKind, Read, Result};
use log::debug;

pub struct HistoryReader<R: Read> {
    reader: R,
    buffer: SliceDeque<u8>,
    history_size: usize,
    window_size: usize,
    current_history_size: usize,
}

impl<R: Read> HistoryReader<R> {
    pub fn new(
        reader: R,
        history_size: usize,
        current_window_size: usize,
    ) -> Result<HistoryReader<R>> {
        let mut r = HistoryReader {
            reader,
            // current_window_size * 2 - because we have to read into this queue before popping
            buffer: SliceDeque::with_capacity(history_size + current_window_size * 2),
            history_size,
            window_size: current_window_size,
            current_history_size: 0,
        };

        let buff_size = current_window_size * 2;
        r.buffer.resize(buff_size, 0);
        let buff = &mut r.buffer[0..buff_size];
        let bytes_read = r.reader.read(buff)?;
        assert!(bytes_read <= buff_size);
        if bytes_read < buff_size {
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                "File has to be at least twice the size of current_window_size",
            ));
        } else {
            r.current_history_size = r.window_size;
        }

        Ok(r)
    }

    // Returns less than end - start only when end of file is reached
    fn read(&mut self, start: usize, end: usize) -> Result<usize> {
        let mut st = start;
        while st < end {
            let buffer = &mut self.buffer[st..end];
            match self.reader.read(buffer) {
                Ok(n) => { 
                    if n > 0 { st += n; } else { break; }
                }
                Err(error) => {
                    match error.kind() {
                        ErrorKind::Interrupted => { continue; },
                        ErrorKind::UnexpectedEof => { break; }
                        _ => { return Err(error); }
                    }
                }
            }
        }

        Ok(st - start)
    }

    // Slides windows a specified amount of bytes and returns slices to history and current windows
    // If file has ended, current window (slice 2) starts getting smaller until it's size becomes 0.
    pub fn next(&mut self, move_bytes: usize) -> Result<(&[u8], &[u8])> {
        let buff_len = self.buffer.len();
        assert!(self.current_history_size >= self.window_size);
        assert!(buff_len == self.current_history_size + self.window_size);
        assert!(
            move_bytes <= self.window_size,
            "Unexpected case. If you're going to move more than current window size
                some bytes will skip current window."
        );
        assert!(self.window_size > 0);
        assert!(self.current_history_size <= self.history_size);

        // debug!(
        //     "current_history_size: {}, current_window_size: {}",
        //     self.current_history_size, self.window_size
        // );

        let new_size = buff_len + move_bytes;
        // Checking if we don't exceed initial capacity
        assert!(new_size <= self.history_size + self.window_size * 2);
        self.buffer.resize(new_size, 0);
        let bytes_read = self.read(buff_len, new_size)?;

        debug!(
            "window_size: {}, move_bytes: {}, bytes_read: {}",
            self.window_size, move_bytes, bytes_read
        );

        let (to_pop, history_size_change) = if bytes_read < move_bytes {
            debug!("Read less than asked");
            // Current window should get smaller by this amount
            let size_change = move_bytes - bytes_read;
            self.window_size -= size_change;
            // We resized to get exactly move_bytes. Have resize back to the actual.
            for _ in 0..size_change {
                self.buffer.pop_back();
            }
            let history_diff = self.history_size - self.current_history_size;
            let history_change = cmp::min(move_bytes, history_diff);
            (move_bytes - history_change, history_change)
        } else {
            // bytes_read == move_bytes
            let history_diff = self.history_size - self.current_history_size;
            if history_diff == 0 {
                (bytes_read, 0)
            } else if history_diff < bytes_read {
                (bytes_read - history_diff, history_diff)
            } else {
                (0, bytes_read)
            }
        };

        self.current_history_size += history_size_change;
        for _ in 0..to_pop {
            self.buffer.pop_front();
        }

        // debug!(
        //     "current_history_size: {}, current_window_size: {}",
        //     self.current_history_size, self.window_size
        // );
        let buff_len = self.buffer.len();
        Ok((
            &self.buffer[0..self.current_history_size],
            &self.buffer[self.current_history_size..buff_len],
        ))
    }

    // Returns slices containing current history and current window
    pub fn current(&self) -> (&[u8], &[u8]) {
        assert!(self.current_history_size >= self.window_size);
        assert!(self.buffer.len() == self.current_history_size + self.window_size);

        (
            &self.buffer[0..self.current_history_size],
            &self.buffer[self.current_history_size..self.buffer.len()],
        )
    }
}
