use restson::{Error, RestClient, RestPath};

use errors::ClientError;

#[derive(Serialize, Deserialize, Debug)]
pub struct Dashboard {
    pub id: u32,
    pub title: String,

    #[serde(default = "Vec::new")]
    pub rows: Vec<Row>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Row {
    pub title: String,

    #[serde(default = "Vec::new")]
    pub panels: Vec<Panel>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Panel {
    pub title: String,
    pub datasource: Option<String>,

    #[serde(default = "Vec::new")]
    pub targets: Vec<Target>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Target {
    pub title: Option<String>,
    pub expr: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DashboardSearchResult {
    id: u32,
    #[serde(rename = "isStarred")]
    is_starred: bool,
    tags: Vec<String>,
    title: String,
    #[serde(rename = "type")]
    dashboard_type: String,
    uri: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum SearchDashboards {
    Array(Vec<DashboardSearchResult>),
}

#[derive(Serialize, Deserialize, Debug)]
struct GetDashboard {
    dashboard: Dashboard,
}

impl RestPath<()> for SearchDashboards {
    fn get_path(_: ()) -> Result<String, Error> {
        Ok(String::from("api/search"))
    }
}

impl RestPath<String> for GetDashboard {
    fn get_path(uri: String) -> Result<String, Error> {
        Ok(format!("api/dashboards/{}", uri))
    }
}

pub struct GrafanaClient {
    client: RestClient,
}

fn to_grafana_error(err: Error) -> ClientError {
    ClientError::RestError {
        client: "grafana",
        err: err,
    }
}

impl GrafanaClient {
    pub fn new(url: String, api_key: Option<String>) -> Result<GrafanaClient, ClientError> {
        RestClient::new(&url)
            .and_then(|mut client| {
                if api_key.is_some() {
                    let mut auth_value = "Bearer ".to_owned();
                    auth_value.push_str(&api_key.unwrap());
                    client.set_header("Authorization", &auth_value)?;
                }

                client.set_header("Accept", "application/json")?;
                client.set_header("Content-Type", "application/json")?;

                Ok(GrafanaClient { client })
            }).map_err(to_grafana_error)
    }
    pub fn find_all_dashboards(mut self) -> Result<Vec<DashboardSearchResult>, ClientError> {
        let result: Result<SearchDashboards, Error> = self.client.get(());
        result
            .map(|result| match result {
                SearchDashboards::Array(result) => result,
            }).map_err(to_grafana_error)
    }

    pub fn get_dashboard(
        mut self,
        search_result: DashboardSearchResult,
    ) -> Result<Dashboard, ClientError> {
        let result: Result<GetDashboard, Error> = self.client.get(search_result.uri);

        result
            .map(|result| result.dashboard)
            .map_err(to_grafana_error)
    }

    pub fn get_dashboard_by_uri(mut self, uri: String) -> Result<Dashboard, ClientError> {
        let result: Result<GetDashboard, Error> = self.client.get(uri);

        result
            .map(|result| result.dashboard)
            .map_err(to_grafana_error)
    }
}
