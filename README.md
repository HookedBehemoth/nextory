# Nextory Client

A Nextory Client library and CLI.

## Warning
This was released because Nextory has overhauled it's API. All of this stopped working.

## Usage

```
Nextory Client CLI

Usage: nextory [OPTIONS]

Options:
  -o, --output <OUTPUT>          Output folder location
      --force-fetch              Number of times to greet
      --username <USERNAME>      Username
      --password <PASSWORD>      Password
      --mark-completed           Immedietly mark book as completed
      --download-active          Download all active books
      --download-new             Download new books
      --download-inactive        Download all inactive books
  -c, --categories <CATEGORIES>  Categories to download (e.g. "tttl_dynamic_2005$$ver_38")
      --views <VIEWS>            Views to download (e.g. "series")
  -h, --help                     Print help
  -V, --version                  Print version
```

## Examples

See [downloader.rs](src/downloader.rs)

## License

```
Nextory Client
Copyright (C) 2023 Luis

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
```
