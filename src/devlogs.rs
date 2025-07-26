use anyhow::Result;
use futures::{StreamExt, TryStreamExt};
use reqwest::header::{COOKIE, HeaderMap, HeaderValue};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Devlogs {
    devlogs: Vec<Devlog>,
    pagination: Pagination,
}

#[derive(Deserialize)]
pub struct Devlog {
    //id: u16,
    pub text: String,
    ////  attachment: Option<Attachment>,
    // project_id: u16,
    // slack_id: String,
    // created_at: String,
    // updated_at: String,
}

#[derive(Deserialize)]
pub struct Pagination {
    // page: u16,
    pages: u16,
    count: u16,
    // items: u16,
}

pub async fn get_all_devlogs(api_key: String) -> Result<Vec<String>> {
    let mut headers = HeaderMap::new();
    headers.insert(
        COOKIE,
        HeaderValue::from_str(format!("_journey_session={api_key}").as_str())?,
    );

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    let first_url = "https://summer.hackclub.com/api/v1/devlogs?page=1";
    let first: Devlogs = client.get(first_url).send().await?.json().await?;

    let total_pages = first.pagination.pages;
    let total_count = first.pagination.count as usize;

    let mut all_devlogs = Vec::with_capacity(total_count);
    all_devlogs.extend(first.devlogs);

    let pages_stream = futures::stream::iter(2..=total_pages).map(|page| {
        let client = client.clone();
        async move {
            let url = format!("https://summer.hackclub.com/api/v1/devlogs?page={page}");
            let resp: Devlogs = client.get(&url).send().await?.json().await?;
            println!("Fetched {page}/{total_pages}");
            Ok::<_, anyhow::Error>(resp.devlogs)
        }
    });

    let mut buffered = pages_stream.buffer_unordered(20);

    while let Some(page_res) = buffered.try_next().await? {
        all_devlogs.extend(page_res);
    }

    Ok(all_devlogs
        .into_iter()
        .map(|x| x.text)
        .collect::<Vec<String>>())
}
