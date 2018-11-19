use std::collections::HashMap;

use restson::{Error, RestClient, RestPath};
use scraper::{Html, Selector};

use errors::ClientError;

use http_client::HttpClient;

#[derive(Serialize, Deserialize, Debug)]
struct GetLabelValues {
    data: Vec<String>,
    status: String,
}

impl RestPath<String> for GetLabelValues {
    fn get_path(label: String) -> Result<String, Error> {
        Ok(format!("api/v1/label/{}/values", label))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Alert {
    #[serde(rename = "alert")]
    pub name: String,
    #[serde(rename = "expr")]
    pub expression: String,
    pub labels: HashMap<String, String>,
    pub annotations: HashMap<String, String>,
}

pub struct PrometheusClient {
    rest_client: RestClient,
    http_client: HttpClient,
}

fn to_prometheus_error(err: Error) -> ClientError {
    ClientError::RestError {
        client: "prometheus",
        err: err,
    }
}

impl PrometheusClient {
    pub fn new(url: String) -> Result<PrometheusClient, ClientError> {
        RestClient::new(&url)
            .and_then(|mut rest_client| {
                rest_client.set_header("Accept", "application/json")?;
                rest_client.set_header("Content-Type", "application/json")?;

                HttpClient::new(url, None, None).map(|http_client| PrometheusClient {
                    http_client,
                    rest_client,
                })
            }).map_err(to_prometheus_error)
    }

    pub fn get_label_values(mut self, label: String) -> Result<Vec<String>, ClientError> {
        let result: Result<GetLabelValues, Error> = self.rest_client.get(label);
        result
            .map(|result| result.data)
            .map_err(to_prometheus_error)
    }

    pub fn get_alerts(mut self) -> Result<Vec<Alert>, ClientError> {
        let alerts_page = self
            .http_client
            .get("alerts", None, None)
            .map(|response| response.body().to_owned())
            .map_err(to_prometheus_error)?;

        let document = Html::parse_document(&alerts_page);
        let selector = Selector::parse("code").unwrap();
        let code_elements = document
            .select(&selector)
            .map(|element| {
                let text = element.text().collect::<Vec<&str>>().join("\n");
                serde_yaml::from_str::<Alert>(&text)
            }).filter(|alert| alert.is_ok())
            .map(|may_alert| may_alert.unwrap())
            .collect::<Vec<Alert>>();

        Ok(code_elements)
    }
}
