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

#[derive(serde::Deserialize, Debug)]
pub struct EmptyResponse {}

pub type DateTime = chrono::DateTime<chrono::Utc>;

#[derive(Clone, Copy)]
pub enum Sort {
    Relevance,
    PublishedDate,
    Rating,
    Title,
    Authors,
    Volume,
    Nest,
}

impl From<Sort> for &str {
    fn from(value: Sort) -> Self {
        match value {
            Sort::Relevance => "relevance",
            Sort::PublishedDate => "published_date",
            Sort::Rating => "average_rating",
            Sort::Title => "title",
            Sort::Authors => "authors",
            Sort::Volume => "volume",
            Sort::Nest => "NEST",
        }
    }
}
