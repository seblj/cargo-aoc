use reqwest::{
    header::{COOKIE, USER_AGENT},
    IntoUrl, Response,
};
use serde::Serialize;

use crate::error::AocError;

pub struct AocRequest
{
    client: reqwest::Client,
}

impl AocRequest
{
    const AOC_USER_AGENT: &str = "github.com/seblj/cargo-aoc by seblyng98@gmail.com";

    pub fn new() -> AocRequest
    {
        AocRequest {
            client: reqwest::Client::new()
        }
    }

    fn get_token(&self) -> Result<String, dotenv::Error>
    {
        dotenv::var("AOC_TOKEN")
    }

    async fn request(self, req: reqwest::RequestBuilder) -> Result<Response, AocError>
    {
        Ok(req
            .header(COOKIE, format!("session={}", self.get_token()?))
            .header(USER_AGENT, AocRequest::AOC_USER_AGENT)
            .send()
            .await?)
    }

    pub async fn get<U: IntoUrl>(self, url: U) -> Result<Response, AocError>
    {
        let req = self.client.get(url);
        self.request(req).await
    }

    pub async fn post<T, U>(self, url: U, form: &T) -> Result<Response, AocError>
    where
        U: IntoUrl,
        T: Serialize + ?Sized,
    {
        let req = self.client.post(url).form(form);
        self.request(req).await
    }
}
