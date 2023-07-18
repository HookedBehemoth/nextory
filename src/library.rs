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

use crate::{api::Result, client::Client, common::EmptyResponse};

use const_format::concatcp;

const BASE_URL: &str = "https://api.nextory.se/api/app/library/7.5/";
const ACTIVE_URL: &str = concatcp!(BASE_URL, "active");
const INACTIVE_URL: &str = concatcp!(BASE_URL, "inactive");
const ACTIVATION_URL: &str = concatcp!(BASE_URL, "directctbookactivation");
const DELETION_URL: &str = concatcp!(BASE_URL, "directctbookdeletion");
const COMPLETED_ADD_URL: &str = concatcp!(BASE_URL, "completed/add");

pub async fn list_active(client: &Client) -> Result<Active> {
    client.get_with_auth(ACTIVE_URL).await
}

pub async fn list_inactive(client: &Client, pagenumber: u32) -> Result<Inactive> {
    let request = client
        .client
        .get(INACTIVE_URL)
        .query(&[("type", "0"), ("sort", "dateModified"), ("rows", "12")])
        .query(&[("pagenumber", pagenumber)]);

    client.request_with_auth(request).await
}

/**
 * traceid usually is a 21 Character alphanumeric string which is generated once per page. See [`crate::randomstring::RandomString`]
 */
pub async fn directctbookactivation(
    client: &Client,
    bookid: u32,
    esalesticket: &str,
    traceid: &str,
) -> Result<Activation> {
    let request = client
        .client
        .post(ACTIVATION_URL)
        .query(&[("bookid", bookid)])
        .query(&[("esalesticket", esalesticket), ("traceid", traceid)]);

    client.request_with_auth(request).await
}

pub async fn directctbookdeletion(client: &Client, bookid: u32) -> Result<EmptyResponse> {
    let request = client
        .client
        .post(DELETION_URL)
        .query(&[("bookid", bookid)])
        .query(&[("esalesticket", "")]);

    client.request_with_auth(request).await
}

pub async fn add_completed(client: &Client, bookid: u32) -> Result<EmptyResponse> {
    const DATETIME_FORMAT_STRING: &str = "%Y-%m-%d %H:%M:%S %z";

    let completion_date = chrono::offset::Utc::now();
    let formatted_date = completion_date.format(DATETIME_FORMAT_STRING).to_string();
    let request = client
        .client
        .post(COMPLETED_ADD_URL)
        .query(&[("bookid", bookid)])
        .query(&[("visibility", "PUBLIC"), ("completeddate", &formatted_date)]);

    client.request_with_auth(request).await
}

#[derive(serde::Deserialize, Debug)]
pub struct Active {
    pub books: Box<[Book]>,
    pub bookcount: usize,
    pub maxactivecount: usize,
}

#[derive(serde::Deserialize, Debug)]
pub struct Inactive {
    pub books: Box<[InactiveBook]>,
}

#[derive(serde::Deserialize, Debug)]
pub struct InactiveBook {
    pub id: u32,
    pub isupcoming: u8,
}

#[derive(serde::Deserialize, Debug)]
pub struct Activation {
    pub books: Book,
}

#[derive(serde::Deserialize, Debug)]
pub struct Book {
    pub id: u32,
    pub isbn: String,
    pub isupcoming: u8,
    #[serde(rename = "type")]
    _type: u8,
    pub title: String,
    pub imageurl: String,
    pub authors: Box<[String]>,
    pub file: File,
    pub pubdate: crate::common::DateTime,
}

#[derive(serde::Deserialize, Debug)]
pub struct File {
    pub url: String,
    pub formatid: u32,
    pub duration: String,
    pub sizeinbytes: usize,
}

#[derive(PartialEq)]
pub enum FileFormat {
    Mp3,
    EPub,
    PdfDrm,
    PdfWatermark,
    HLS,
    Unknown,
}

impl From<u32> for FileFormat {
    fn from(value: u32) -> Self {
        match value {
            0x016 => Self::Mp3,
            0x009 => Self::EPub,
            0x00A => Self::PdfDrm,
            0x00B => Self::PdfWatermark,
            0x130 => Self::HLS,
            _ => Self::Unknown,
        }
    }
}

impl FileFormat {
    pub fn get_extension(&self) -> &'static str {
        match self {
            FileFormat::Mp3 => "mp3",
            FileFormat::EPub => "epub",
            FileFormat::PdfDrm | FileFormat::PdfWatermark => "pdf",
            _ => panic!("unsupported file type for single file store"),
        }
    }
}
