use chrono::Local;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;

pub struct API {
    username: String,
    token: String,
    client: Client,
}

#[derive(Debug, Deserialize)]
struct GetPixelResponse {
    quantity: i64,
}

impl API {
    pub fn new(username: String, token: String) -> API {
        API {
            username,
            token,
            client: Client::new(),
        }
    }

    pub async fn get_pixel(&self, graph_id: &str) -> i64 {
        let time = &Local::now().format("%Y%m%d").to_string();
        let url = format!(
            "https://pixe.la/v1/users/{}/graphs/{}/{}",
            self.username, graph_id, time,
        );
        let response = self
            .client
            .get(&url)
            .header("X-USER-TOKEN", &self.token)
            .send()
            .await;
        match response {
            Ok(res) => {
                if let Ok(body) = res.json::<GetPixelResponse>().await {
                    return body.quantity;
                }
                0
            }
            Err(_) => 0,
        }
    }

    pub async fn post_pixel(&self, graph_id: &str, count: i64) -> anyhow::Result<()> {
        let cnt = &format!("{}", count);
        let time = &Local::now().format("%Y%m%d").to_string();

        let mut map = HashMap::<&str, &str>::new();
        map.insert("quantity", cnt);

        let client = Client::new();
        let url = format!(
            "https://pixe.la/v1/users/{}/graphs/{}/{}",
            self.username, graph_id, time,
        );
        client
            .put(&url)
            .header("X-USER-TOKEN", &self.token)
            .json(&map)
            .send()
            .await?;

        Ok(())
    }
}
