use clap::SubCommand;

use super::super::cli::*;
use super::super::errors::*;

pub struct GetAlertsCmd {}

impl Command for GetAlertsCmd {
    fn get_name<'a>(&self) -> &'a str {
        "get-alerts"
    }
    fn get_cmd<'a>(&self) -> clap::App<'a, 'a> {
        let cmd = SubCommand::with_name(self.get_name()).about("get prometheus alerts");

        add_prometheus_args(cmd)
    }

    fn run_cmd(&self, matches: &clap::ArgMatches) -> CommandResult {
        let prometheus_client = get_prometheus_client(matches)?;

        let values = prometheus_client
            .get_alerts()
            .map_err(CliError::ClientError)
            .map(|result| serde_json::to_value(result).unwrap());

        values
    }
}
