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

mod api;
mod catalogue;
mod client;
mod common;
mod downloader;
mod library;
mod randomstring;

use std::{fs, path::PathBuf, str::FromStr};

use crate::{client::Client, common::Sort, downloader::Downloader};

const TOKEN_PATH: &str = "token.txt";

use clap::Parser;

/// Nextory Client CLI
#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Output folder location
    #[arg(short, long)]
    output: Option<String>,

    /// Number of times to greet
    #[arg(long, default_value_t = false)]
    force_fetch: bool,

    /// Username
    #[arg(long)]
    username: Option<String>,

    /// Password
    #[arg(long)]
    password: Option<String>,

    /// Immedietly mark book as completed
    #[arg(long, default_value_t = true)]
    mark_completed: bool,

    /// Download all active books
    #[arg(long, default_value_t = true)]
    download_active: bool,

    /// Download new books
    #[arg(long)]
    download_new: bool,

    /// Download all inactive books
    #[arg(long, default_value_t = true)]
    download_inactive: bool,

    /// Categories to download (e.g. "tttl_dynamic_2005$$ver_38")
    #[arg(short, long)]
    categories: Vec<String>,

    /// Views to download (e.g. "series")
    #[arg(long)]
    views: Vec<String>,
}

#[tokio::main]
async fn main() -> api::Result<()> {
    let args = Args::parse();

    println!("{args:?}");

    /* Load ouput directory, fallback to cwd */
    let dest = if let Some(path) = args.output {
        let dest = PathBuf::from_str(&path).unwrap();

        if !dest.exists() {
            fs::create_dir_all(&dest).unwrap();
        }

        dest
    } else {
        eprintln!("Storing in current working directory");
        std::env::current_dir().unwrap()
    };

    /* Allow overriding token */
    if args.force_fetch {
        let _ = fs::remove_file(TOKEN_PATH);
    }

    let client = if let Ok(token) = fs::read_to_string(TOKEN_PATH) {
        let client = Client::from_token(token)?;

        client
    } else {
        let username: String = args.username.expect("Username not specified");
        let password: String = args.password.expect("Password not specified");

        let client = Client::from_credentials(&username, &password).await?;

        let _ = fs::write(TOKEN_PATH, &client.token);

        client
    };

    let downloader = Downloader::new(dest, args.mark_completed);

    if args.download_inactive {
        downloader.download_inactive(&client).await?;
    }

    if args.download_active {
        downloader.download_active(&client).await?;
    }

    if args.download_new {
        downloader.download_new(&client).await?;
    }

    for category in args.categories {
        downloader.download_category(&category, Sort::Relevance, &client).await?;
    }

    for view in args.views {
        downloader.download_groups(Some(&view), Sort::Relevance, &client).await?;
    }

    Ok(())
}
