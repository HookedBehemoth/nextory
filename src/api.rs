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

 #[derive(Debug)]
pub enum Error {
    Api(u16, String),
    Cdn(u16, String),
    Status(String),
    Reqwest(reqwest::Error),
    Unknown,
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self::Reqwest(value)
    }
}

impl Error {
    pub async fn ensure_ok(response: reqwest::Response) -> Result<reqwest::Response> {
        let status = response.status();
        if !status.is_success() {
            let text = response.text().await;
            let message = text.unwrap_or_default();
            return Err(Error::Api(status.as_u16(), message));
        }

        Ok(response)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
