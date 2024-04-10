use crate::buffered_response::BufferedResponse;
use crate::digi4school::book::Book;
use crate::digi4school::lti_form::LTIForm;
use crate::scraper::get_scraper_constructor;
use crate::scraper::scraper_trait::Scraper;
use getset::Getters;
use reqwest::{Client, Url};
use std::sync::{Arc, OnceLock};

#[derive(Getters)]
pub struct Volume {
    url: Url,
    resp: OnceLock<BufferedResponse>,

    #[getset(get = "pub")]
    name: String,
    #[getset(get = "pub")]
    thumbnail: Url,

    client: Arc<Client>,
}

impl Volume {
    pub(crate) fn new(url: Url, name: &str, thumbnail: Url, client: Arc<Client>) -> Self {
        Self {
            url,
            resp: OnceLock::default(),

            name: name.to_string(),
            thumbnail,

            client,
        }
    }

    pub(crate) fn from_single_volume_book(book: &Book, resp: BufferedResponse) -> Self {
        Volume {
            url: resp.url().clone(),
            resp: OnceLock::from(resp),

            name: book.title().to_string(),
            thumbnail: book.thumbnail().clone(),

            client: book.client(),
        }
    }

    pub async fn get_scraper(&self) -> Result<Box<dyn Scraper + '_>, reqwest::Error> {
        let resp = self.get_response().await?;

        Ok(get_scraper_constructor(resp.url())(
            resp,
            self.client.clone(),
        ))
    }

    async fn get_response(&self) -> Result<&BufferedResponse, reqwest::Error> {
        match self.resp.get() {
            Some(resp) => Ok(resp),
            None => {
                self.gen_response().await?;
                Ok(self.resp.get().unwrap())
            }
        }
    }

    async fn gen_response(&self) -> Result<(), reqwest::Error> {
        if self.resp.get().is_none() {
            self.resp
                .set(
                    LTIForm::follow(
                        BufferedResponse::new(self.client.get(self.url.clone()).send().await?)
                            .await?,
                        &self.client,
                    )
                    .await?,
                )
                .unwrap();
        }

        Ok(())
    }
}
