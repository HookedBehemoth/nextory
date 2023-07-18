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
use std::{fs::create_dir_all, path::PathBuf};

use futures_util::stream::StreamExt;
use id3::TagLike;
use tokio::io::AsyncWriteExt;

use crate::{
    api::Result,
    catalogue,
    catalogue::Search,
    client::Client,
    common::Sort,
    library,
    library::{Book, File, FileFormat},
};

const PROGRESS_TEMPLATE: &str = "{wide_bar} [{bytes:10}/{total_bytes:10}] {eta:4}";

pub struct Downloader {
    path: PathBuf,
    pub style: indicatif::ProgressStyle,
    mark_completed: bool,
}

impl Downloader {
    pub fn new(path: PathBuf, mark_completed: bool) -> Self {
        let style = indicatif::ProgressStyle::default_bar()
            .template(PROGRESS_TEMPLATE)
            .unwrap();

        Self {
            path,
            style,
            mark_completed,
        }
    }

    async fn download_file(
        &self,
        client: &Client,
        folder: &str,
        api_file: &File,
        file_name: &str,
    ) -> Result<PathBuf> {
        let mut path = self.path.clone();

        path.push(folder.replace('/', "_"));
        if !path.exists() {
            create_dir_all(&path).unwrap();
        }

        path.push(file_name.replace('/', "_"));

        if path.exists() {
            println!("{file_name} exists. Skipping!");
            return Ok(path);
        }

        println!("downloading {path:?}");

        let mut file = tokio::fs::File::create(&path).await.unwrap();

        let response = client.start_download(api_file).await?;

        let content_size = if let Some(length) = response
            .headers()
            .get("Content-Length")
            .and_then(|e| e.to_str().ok())
            .and_then(|s| s.parse().ok())
        {
            length
        } else {
            api_file.sizeinbytes
        } as u64;

        let bar = indicatif::ProgressBar::new(content_size).with_style(self.style.clone());

        let mut stream = response.bytes_stream();
        while let Some(Ok(mut chunk)) = stream.next().await {
            bar.inc(chunk.len() as u64);
            let _ = file.write_all_buf(&mut chunk).await;
        }

        bar.finish_and_clear();

        Ok(path)
    }

    pub async fn download_book(&self, client: &Client, book: &Book) -> Result<PathBuf> {
        let file_format = FileFormat::from(book.file.formatid);
        let file_name = match book.title.char_indices().nth(200) {
            None => &book.title,
            Some((idx, _)) => &book.title[..idx],
        };

        let file_name = format!("{}.{}", file_name, file_format.get_extension());
        let authors = if book.authors.len() > 5 {
            &book.authors[..5]
        } else {
            &book.authors
        };
        let folder = authors.join(" & ");
        let path = self
            .download_file(client, &folder, &book.file, &file_name)
            .await?;

        if file_format == FileFormat::Mp3 {
            use id3::{
                frame::{Picture, PictureType},
                Tag, Version,
            };

            let mut tag = Tag::read_from_path(&path).unwrap();

            tag.set_title(&book.title);
            if let Some(author) = book.authors.get(0) {
                tag.set_artist(author);
            }

            let response = reqwest::get(&book.imageurl).await?;
            let mime = response.headers().get("content-type").unwrap();

            tag.add_frame(Picture {
                mime_type: mime.to_str().unwrap().to_owned(),
                picture_type: PictureType::CoverFront,
                description: String::new(),
                data: response.bytes().await.unwrap().to_vec(),
            });

            tag.write_to_path(&path, Version::Id3v23).unwrap();
        }

        Ok(path)
    }

    pub async fn download_active(&self, client: &Client) -> Result<()> {
        println!("Downloading \"active\" books");
        let active = library::list_active(&client).await?;
        for book in active.books.iter() {
            self.download_book(&client, &book).await?;

            if self.mark_completed {
                library::add_completed(&client, book.id).await?;
            }
        }

        Ok(())
    }

    pub async fn download_inactive(&self, client: &Client) -> Result<()> {
        println!("Downloading \"inactive\"/saved books");
        loop {
            let inactive = library::list_inactive(&client, 0).await?;

            for book in inactive.books.iter() {
                /* Upcoming books can't be activated */
                if book.isupcoming == 1 {
                    continue;
                }

                match library::directctbookactivation(&client, book.id, "", "").await {
                    Ok(activation) => {
                        if let Err(err) = self.download_book(&client, &activation.books).await {
                            eprintln!("{} failed with {:?}", book.id, err);
                        }

                        if self.mark_completed {
                            library::add_completed(&client, book.id).await?;
                        }
                    }
                    Err(err) => {
                        eprintln!("Failed to download with error {:?}", err);
                    }
                }
            }
            if inactive.books.len() < 12 {
                break;
            }
        }

        Ok(())
    }

    pub async fn download_groups(
        &self,
        view: Option<&str>,
        sort: Sort,
        client: &Client,
    ) -> Result<()> {
        let mut i: u32 = 0;
        loop {
            println!("Categories page {i}");
            let groups = catalogue::groups(&client, i, view).await?;

            if groups.bookgroups.is_empty() {
                break;
            }

            for group in groups.bookgroups.iter() {
                self.download_category(&group.id, sort, &client).await?;
            }
            i += 1;
        }

        Ok(())
    }

    pub async fn download_search(&self, client: &Client, search: &Search) -> Result<()> {
        let traceid = client.random.next_string::<21>();

        for book in search.books.iter() {
            println!("{book}");
            if book.libstatus != "NOTINLIB" && book.isupcoming.is_some_and(|v| v == 1) {
                continue;
            }

            match library::directctbookactivation(
                &client,
                book.id,
                &book.esalesticket,
                traceid.as_str(),
            )
            .await
            {
                Ok(activation) => {
                    if let Err(err) = self.download_book(&client, &activation.books).await {
                        println!("{} failed with {:?}", book.id, err);
                    }

                    if self.mark_completed {
                        library::add_completed(client, book.id).await?;
                    }
                }
                Err(err) => {
                    println!("Failed to download with error {:?}", err);
                }
            }
        }

        Ok(())
    }

    pub async fn download_new(&self, client: &Client) -> Result<()> {
        println!("Downloading new books");
        for i in 0.. {
            let search = catalogue::new(client, i).await?;

            println!("page {i}; count: {}", search.bookcount);
            if search.books.len() == 0 {
                break;
            }

            self.download_search(client, &search).await?;
        }

        Ok(())
    }

    pub async fn download_category(
        &self,
        category: &str,
        sort: Sort,
        client: &Client,
    ) -> Result<()> {
        println!("Downloading category {category}");
        let mut pagetoken: Option<String> = None;
        let mut i = 0;
        loop {
            let search = if let Some(p) = pagetoken {
                catalogue::booksforbookgroup(&client, category, sort, Some(&p), Some(i)).await?
            } else {
                catalogue::booksforbookgroup(&client, category, sort, None, Some(i)).await?
            };

            println!("page {i}; count: {}", search.bookcount);
            if search.books.len() == 0 {
                break;
            }

            self.download_search(client, &search).await?;

            pagetoken = search.pagetoken;
            i += 1;
        }

        Ok(())
    }
}
