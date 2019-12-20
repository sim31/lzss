//use bitbit::{BitReader, BitWriter, MSB};
use slice_deque::SliceDeque;
use std::io::{Error, ErrorKind, Read, Result};

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

        let new_size = buff_len + move_bytes;
        self.buffer.resize(new_size, 0);
        let buff = &mut self.buffer[buff_len..new_size];

        let bytes_read = self.reader.read(buff)?;
        println!(
            "window_size: {}, move_bytes: {}, bytes_read: {}",
            self.window_size, move_bytes, bytes_read
        );

        let (to_pop, history_size_change) = if bytes_read < move_bytes {
            // Current window should get smaller by this amount
            let size_change = move_bytes - bytes_read;
            self.window_size -= size_change;
            // We resized to get exactly move_bytes. Have resize back to the actual.
            for _ in 0..size_change {
                self.buffer.pop_back();
            }
            (bytes_read, size_change)
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
        // // We resized to get exactly move_bytes. Have resize to the actual.
        // let rem = std::cmp::min(self.window_size, move_bytes - bytes_read);
        // let to_add_history = if rem > 0 {
        //     for _ in 0..rem {
        //         self.buffer.pop_back();
        //     }
        //     // Means current window got smaller
        //     self.window_size -= rem;
        //     rem     // We want history to expand by this amount at this point
        // } else {
        //     0
        // };

        // // Adjust history window
        // let history_diff = self.history_size - self.current_history_size;
        // let to_pop = if history_diff < bytes_read {
        //     let to_add = std::cmp::max(history_diff, to_add_history);
        //     self.current_history_size += to_add;
        //     if bytes_read > to_add {
        //         bytes_read - to_add
        //     } else {
        //         move_bytes - bytes_read - history_diff
        //     }
        // } else {
        //     self.current_history_size += bytes_read + to_add_history;
        //     0
        // };

        let buff_len = self.buffer.len();
        println!(
            "current_history_size: {}, current_window_size: {}",
            self.current_history_size, self.window_size
        );
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
