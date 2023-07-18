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

use crate::{
    api::{Error, Result},
    randomstring::RandomString,
};

use reqwest::header::{HeaderMap, HeaderValue};

const USER_AGENT: &str = "okhttp/4.9.3";
const USER_AGENT_DOWNLOAD: &str =
    "Dalvik/2.1.0 (Linux; U; Android 10; ONEPLUS A5000 Build/QKQ1.191014.012)";
const API_VERSION: &str = "7.5";

const SALT_URL: &str = "https://api.nextory.se/api/app/catalogue/7.5/salt";
const USER_LOGIN_URL: &str = "https://api.nextory.se/api/app/user/7.5/login";
const USER_ACCOUNTS_LIST_URL: &str = "https://api.nextory.se/api/app/user/7.5/accounts/list";

#[derive(serde::Deserialize, Debug)]
struct Response<T> {
    data: Option<T>,
    error: Option<NextoryError>,
}

#[derive(serde::Deserialize, Debug)]
struct NextoryError {
    msg: String,
    code: u16,
}

impl From<NextoryError> for Error {
    fn from(value: NextoryError) -> Self {
        Self::Api(value.code, value.msg)
    }
}

#[derive(serde::Deserialize, Debug)]
struct SaltData {
    salt: String,
}

#[derive(serde::Deserialize, Debug)]
struct LoginData {
    token: String,
    accounttype: u8,
}

pub enum AccountType {
    Member = 1,
    Canceled = 2,
    NonMember = 3,
    Visitor = 4,
}

#[derive(serde::Deserialize, Debug)]
pub struct AccountList {
    accounts: Box<[SubAccount]>,
}

#[derive(serde::Deserialize, Debug)]
struct SubAccount {
    loginkey: String,
    status: String,
}

pub struct Client {
    pub client: reqwest::Client,
    pub token: String,
    pub random: RandomString,
}

impl Client {
    fn inner() -> Result<reqwest::Client> {
        let headers = [
            ("canary", ""),
            ("appid", "200"),
            ("model", "OnePlus+ONEPLUS+A5000"),
            ("locale", "en_GB"),
            ("version", "4.34.6"),
            ("deviceid", "eSsnwXyvS4qK4vMzu79tGh"),
            ("osinfo", "Android 10"),
        ];

        let headers = headers.iter().fold(
            reqwest::header::HeaderMap::new(),
            |mut map, &(k, v)| -> HeaderMap {
                map.append(k, HeaderValue::from_static(v));
                map
            },
        );

        /* 15 Minute keepalive */
        let keepalive = std::time::Duration::from_secs(15 * 60);

        let mut builder = reqwest::ClientBuilder::new()
            .user_agent(USER_AGENT)
            .tcp_keepalive(keepalive)
            .default_headers(headers);

        if cfg!(feature = "mitm") {
            println!("Installing ssl proxy with certificate");
            let proxy = reqwest::Proxy::all("http://127.0.0.1:8888").unwrap();
            let pem = std::fs::read("cert.pem").unwrap();
            let cert = reqwest::Certificate::from_pem(&pem).unwrap();
            builder = builder.proxy(proxy).add_root_certificate(cert);
        }

        let inner = builder.build()?;

        Ok(inner)
    }

    pub fn from_token(token: String) -> Result<Self> {
        let client = Self::inner()?;

        let random = RandomString::new(None);

        Ok(Self {
            client,
            token,
            random,
        })
    }

    pub async fn from_credentials(username: &str, password: &str) -> Result<Self> {
        let client = Self::inner()?;

        let salt = Self::salt(&client).await?;

        /* Initial login step to get the main account */
        let token = Self::user_login(&client, username, password, &salt).await?;
        let _self = Self {
            client,
            token,
            random: RandomString::new(None),
        };

        /* Authenticate with a subaccount */
        let accounts = _self.user_accounts_list().await?;
        let Some(key) = accounts.accounts.iter().find(|sub| sub.status == "active").map(|sub| &sub.loginkey) else {
			return Err(Error::Status("Couldn't find loginkey for active subacccount".into()))
		};
        let token = _self.user_login_subaccount(&key, &salt).await?;

        Ok(_self.update_token(token))
    }

    fn update_token(self, token: String) -> Self {
        Self {
            client: self.client,
            token,
            random: RandomString::new(None),
        }
    }

    async fn parse<T: serde::de::DeserializeOwned>(response: reqwest::Response) -> Result<T> {
        let response: Response<T> = response.json().await?;

        if let Some(error) = response.error {
            Err(error.into())
        } else if let Some(data) = response.data {
            Ok(data)
        } else {
            Err(Error::Unknown)
        }
    }

    async fn salt(client: &reqwest::Client) -> Result<String> {
        let request = client.get(SALT_URL);

        let response = request.send().await?;

        let response = Error::ensure_ok(response).await?;

        let response: SaltData = Self::parse(response).await?;

        Ok(response.salt)
    }

    async fn user_login(
        client: &reqwest::Client,
        username: &str,
        password: &str,
        salt: &str,
    ) -> Result<String> {
        use md5::{Digest, Md5};

        let hashable = format!("{username}{salt}{password}");

        let mut hasher = Md5::new();
        hasher.update(hashable.as_bytes());
        let result = hasher.finalize();

        let checksum = format!("{:02X}", result);

        let form = reqwest::multipart::Form::new()
            .text("username", username.to_string())
            .text("password", password.to_string())
            .text("checksum", checksum);

        let request = client.post(USER_LOGIN_URL).multipart(form);

        let response = request.send().await?;

        let response = Error::ensure_ok(response).await?;

        let response: LoginData = Self::parse(response).await?;

        if response.accounttype != 1 {
            eprintln!(
                "Unrecognized account type: {}\nThere will be dragons",
                response.accounttype
            );
        }

        Ok(response.token)
    }

    pub(crate) async fn request_with_auth<T: serde::de::DeserializeOwned>(
        &self,
        request: reqwest::RequestBuilder,
    ) -> Result<T> {
        let request = request.header("token", &self.token);

        let response = request.send().await?;

        let response = Error::ensure_ok(response).await?;

        Self::parse(response).await
    }

    pub(crate) async fn get_with_auth<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
    ) -> Result<T> {
        self.request_with_auth(self.client.get(url)).await
    }

    pub async fn user_accounts_list(&self) -> Result<AccountList> {
        self.get_with_auth(USER_ACCOUNTS_LIST_URL).await
    }

    async fn user_login_subaccount(&self, loginkey: &str, salt: &str) -> Result<String> {
        use md5::{Digest, Md5};

        let hashable = format!("{loginkey}{salt}");

        let mut hasher = Md5::new();
        hasher.update(hashable.as_bytes());
        let result = hasher.finalize();

        let login_url = format!("{USER_LOGIN_URL}?loginkey={loginkey}&checksum={result:032X}");
        let login = self.get_with_auth::<LoginData>(&login_url).await?;

        Ok(login.token)
    }

    pub async fn start_download(&self, file: &crate::library::File) -> Result<reqwest::Response> {
        let url: &str = &file.url;
        let response = self
            .client
            .get(url)
            .header("token", &self.token)
            .header("User-Agent", USER_AGENT_DOWNLOAD)
            .header("apiver", API_VERSION)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error = response
                .text()
                .await
                .unwrap_or_else(|_| "(Unknown)".to_owned());
            return Err(Error::Cdn(status.as_u16(), error));
        }

        Ok(response)
    }
}
