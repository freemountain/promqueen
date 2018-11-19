use clap::{Arg, SubCommand};

use super::super::cli::*;
use super::super::errors::*;

pub struct GetValuesCmd {}

impl Command for GetValuesCmd {
    fn get_name<'a>(&self) -> &'a str {
        "get-values"
    }
    fn get_cmd<'a>(&self) -> clap::App<'a, 'a> {
        let mut cmd = SubCommand::with_name(self.get_name()).about("get prometheus label values");

        cmd = add_prometheus_args(cmd).arg(
            Arg::with_name("label")
                .help("the label name for which all values should be exported")
                .required(true)
                .index(1),
        );

        cmd
    }

    fn run_cmd(&self, matches: &clap::ArgMatches) -> CommandResult {
        let prometheus_client = get_prometheus_client(matches)?;
        let label = get_arg(matches, "label")?;

        let values = prometheus_client
            .get_label_values(label)
            .map_err(CliError::ClientError)
            .map(|result| serde_json::to_value(result).unwrap());

        values
    }
}
