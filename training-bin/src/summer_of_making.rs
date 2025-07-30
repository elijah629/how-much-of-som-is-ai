use std::time::Duration;

use anyhow::{Result, anyhow};
use futures::{StreamExt, TryStreamExt};
use reqwest::header::{COOKIE, HeaderMap, HeaderValue};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Devlogs {
    devlogs: Vec<Devlog>,
    pagination: Pagination,
}

#[derive(Deserialize)]
pub struct Projects {
    projects: Vec<Project>,
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
pub struct Project {
    //id: u16
    // title: string
    pub description: String,
    // category: null
    // readme_link
    // demo_link
    // repo_link
    // slack_id
    // created_at
    // updated_at
}

pub trait PagintatedResponse<T> {
    fn pagintation(&self) -> &Pagination;
    fn page(self) -> Vec<T>;
}

impl PagintatedResponse<Devlog> for Devlogs {
    fn pagintation(&self) -> &Pagination {
        &self.pagination
    }

    fn page(self) -> Vec<Devlog> {
        self.devlogs
    }
}

impl PagintatedResponse<Project> for Projects {
    fn pagintation(&self) -> &Pagination {
        &self.pagination
    }

    fn page(self) -> Vec<Project> {
        self.projects
    }
}

#[derive(Deserialize)]
pub struct Pagination {
    // page: u16,
    pages: u16,
    count: u16,
    // items: u16,
}

pub async fn pagintated_fetch<
    T: for<'a> Deserialize<'a> + PagintatedResponse<D>,
    D: for<'a> Deserialize<'a>,
>(
    api_url: &str,
    api_key: &str,
) -> Result<Vec<D>> {
    let mut headers = HeaderMap::new();
    headers.insert(
        COOKIE,
        HeaderValue::from_str(format!("_journey_session={api_key}").as_str())?,
    );

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    let first_url = format!("{api_url}?page=1");
    let first: T = client.get(first_url).send().await?.json().await?;

    let Pagination { count, pages } = first.pagintation();
    let pages = *pages;

    let mut all_data = Vec::with_capacity(*count as usize);
    all_data.extend(first.page());

    let pages_stream = futures::stream::iter(2..=pages).map(|page| {
        let client = client.clone();
        let url = format!("{api_url}?page={page}");

        async move {
            let resp = fetch_page_with_retries::<T>(&client, &url, 3).await?;
            let resp = resp.page();

            println!("{api_url} {page}/{pages}");
            Ok::<_, anyhow::Error>(resp)
        }
    });

    let mut buffered = pages_stream.buffer_unordered(20);

    while let Some(page_res) = buffered.try_next().await? {
        all_data.extend(page_res);
    }

    Ok(all_data)
}

async fn fetch_page_with_retries<T>(
    client: &reqwest::Client,
    url: &str,
    max_retries: usize,
) -> Result<T>
where
    T: for<'a> Deserialize<'a>,
{
    let mut attempt = 0;
    loop {
        match client.get(url).send().await {
            Ok(resp) => match resp.json::<T>().await {
                Ok(json) => return Ok(json),
                Err(err) => {
                    attempt += 1;
                    if attempt >= max_retries {
                        return Err(anyhow!(
                            "JSON parse error after {max_retries} attempts: {err}"
                        ));
                    }
                }
            },
            Err(err) => {
                attempt += 1;
                if attempt >= max_retries {
                    return Err(anyhow!(
                        "Request failed after {max_retries} attempts: {err}"
                    ));
                }
            }
        }

        let delay = Duration::from_millis(500 * 2_u64.pow(attempt as u32 - 1));
        println!("Retrying in {}ms", delay.as_millis());
        tokio::time::sleep(delay).await;
    }
}

pub async fn fetch_all(api_key: &str) -> Result<Vec<String>> {
    let devlogs =
        pagintated_fetch::<Devlogs, Devlog>("https://summer.hackclub.com/api/v1/devlogs", api_key)
            .await?
            .into_iter()
            .map(|x| x.text);

    let projects = pagintated_fetch::<Projects, Project>(
        "https://summer.hackclub.com/api/v1/projects",
        api_key,
    )
    .await?
    .into_iter()
    .map(|x| x.description);

    Ok(devlogs.chain(projects).collect())
}
