/**
 * Nextory Client
 * Copyright (C) 2023 Luis
 * 
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 * 
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 * 
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::mem::MaybeUninit;

use rand::*;

const ALPHANUM: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

pub struct AsciiString<const LENGTH: usize> {
    inner: [u8; LENGTH],
}

impl<const LENGTH: usize> AsciiString<LENGTH> {
    fn new(inner: [u8; LENGTH]) -> AsciiString<LENGTH> {
        debug_assert!(std::str::from_utf8(&inner).is_ok_and(|s| s.is_ascii()));
        Self { inner }
    }
    pub fn as_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.inner) }
    }
}

pub struct RandomString {
    symbols: &'static [u8],
}

impl RandomString {
    pub fn new(symbols: Option<&'static [u8]>) -> RandomString {
        RandomString {
            symbols: symbols.unwrap_or(ALPHANUM),
        }
    }

    pub fn next_string<const LENGTH: usize>(&self) -> AsciiString<LENGTH> {
		let mut rng = thread_rng();
		
		let mut buf: MaybeUninit<[u8; LENGTH]> = MaybeUninit::uninit();
		let buf = unsafe {
			for i in 0..LENGTH {
				let idx = rng.gen_range(0..self.symbols.len());
				(*buf.as_mut_ptr())[i] = self.symbols[idx] as u8;
			}
			buf.assume_init()
		};

		AsciiString::new(buf)
	}
}
