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

use crate::{api::Result, client::Client, common::Sort};

use const_format::concatcp;

const BASE_URL: &str = "https://api.nextory.se/api/app/catalogue/7.5/";
const GROUPS_URL: &str = concatcp!(BASE_URL, "groups");
const BOOKSFORBOOKGROUP_URL: &str = concatcp!(BASE_URL, "booksforbookgroup");

use chrono::Datelike;

#[derive(serde::Deserialize, Debug)]
pub struct Search {
    pub books: Box<[Book]>,
    pub bookcount: usize,
    pub pagetoken: Option<String>,
}

#[derive(serde::Deserialize, Debug)]
pub struct Book {
    pub id: u32,
    pub title: String,
    pub imageurl: String,
    pub authors: Box<[String]>,
    pub pubdate: crate::common::DateTime,
    pub esalesticket: String,
    pub isupcoming: Option<u32>,
    pub avgrate: f32,
    pub libstatus: String,
}

impl std::fmt::Display for Book {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{} {}({}) by {}",
            self.id,
            self.title,
            self.pubdate.year(),
            self.authors.join(", ")
        ))
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct Groups {
    pub bookgroups: Box<[Group]>,
    pub bookgroupcount: u32,
}

#[derive(serde::Deserialize, Debug)]
pub struct Group {
    pub id: String,
}

pub async fn groups(
    client: &Client,
    pagenumber: u32,
    view: Option<&str>,
) -> Result<Groups> {
    let mut request = client
        .client
        .get(GROUPS_URL)
        .query(&[
            ("languages", "de,en"),
            ("formattype", "0"),
            ("pagesize", "12"),
        ])
        .query(&[("pagenumber", pagenumber)]);

    if let Some(view) = view {
        request = request.query(&[("view", view)]);
    }

    client.request_with_auth(request).await
}

pub async fn booksforbookgroup(
    client: &Client,
    bookgroupid: &str,
    sort: Sort,
    pagetoken: Option<&str>,
    pagenumber: Option<u32>,
) -> Result<Search> {
    let mut request = client.client.get(BOOKSFORBOOKGROUP_URL).query(&[
        ("bookgroupid", bookgroupid),
        ("sort", sort.into()),
        ("type", "0"),
        ("languages", "de,en"),
        ("pagetoken", pagetoken.unwrap_or_default()),
        ("segment", "5"),
        ("rows", "12"),
        ("includenotallowedbooks", "true"),
    ]);

    if let Some(pagenumber) = pagenumber {
        request = request.query(&[("pagenumber", pagenumber)]);
    }

    client.request_with_auth(request).await
}

pub async fn new(client: &Client, pagenumber: u32) -> Result<Search> {
    let request = client
        .client
        .get(BOOKSFORBOOKGROUP_URL)
        .query(&[
            ("bookgroupid", "tttl_dynamic_1544$$ver_179"),
            ("sort", "NEST"),
            ("includenotallowedbooks", "true"),
        ])
        .query(&[("pagenumber", pagenumber)])
        .query(&[
            ("type", "0"),
            ("languages", "de,en"),
            ("rows", "12"),
            ("segment", "1"),
            ("pagetoken", ""),
        ]);

    client.request_with_auth(request).await
}
