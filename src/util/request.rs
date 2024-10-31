use reqwest::{
    header::{COOKIE, USER_AGENT},
    IntoUrl, Response,
};

use crate::error::AocError;

pub struct AocRequest {
    client: reqwest::Client,
}

impl AocRequest {
    const AOC_USER_AGENT: &'static str =
        "github.com/seblj/cargo-aoc by sebastian@lyngjohansen.com and sivert-joh@hotmail.com";

    pub fn new() -> AocRequest {
        AocRequest {
            client: reqwest::Client::new(),
        }
    }

    fn get_token(&self) -> Result<String, dotenv::Error> {
        dotenv::var("AOC_TOKEN")
    }

    async fn request(self, req: reqwest::RequestBuilder) -> Result<Response, AocError> {
        let token = self.get_token()?.replace("session=", "");
        Ok(req
            .header(COOKIE, format!("session={}", token))
            .header(USER_AGENT, AocRequest::AOC_USER_AGENT)
            .send()
            .await?)
    }

    pub async fn get<U: IntoUrl>(self, url: U) -> Result<Response, AocError> {
        let req = self.client.get(url);
        self.request(req).await
    }

    #[cfg(feature = "submit")]
    pub async fn post<T, U>(self, url: U, form: &T) -> Result<Response, AocError>
    where
        U: IntoUrl,
        T: serde::Serialize + ?Sized,
    {
        let req = self.client.post(url).form(form);
        self.request(req).await
    }
}
