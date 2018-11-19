use clap::{App as ClapApp, Arg};

use errors::*;
use grafana::GrafanaClient;
use prometheus::PrometheusClient;

pub fn add_prometheus_args<'a, 'b>(cmd: ClapApp<'a, 'b>) -> ClapApp<'a, 'b> {
    cmd.arg(
        Arg::with_name("prometheus-url")
            .long("prometheus-url")
            .value_name("URL")
            .takes_value(true),
    )
}

pub fn add_grafana_args<'a, 'b>(cmd: ClapApp<'a, 'b>) -> ClapApp<'a, 'b> {
    cmd.arg(
        Arg::with_name("grafana-url")
            .long("grafana-url")
            .value_name("URL")
            .required(true)
            .takes_value(true),
    ).arg(
        Arg::with_name("grafana-api-key")
            .long("grafana-api-key")
            .value_name("KEY")
            .required(true)
            .takes_value(true),
    )
}

pub fn get_arg(m: &clap::ArgMatches, name: &str) -> Result<String, CliError> {
    m.value_of(name)
        .map(|arg| arg.to_owned())
        .ok_or(CliError::ArgumentRequired(name.to_string()))
}

pub fn get_grafana_client(m: &clap::ArgMatches) -> Result<GrafanaClient, CliError> {
    get_arg(m, "grafana-url").and_then(|url| {
        GrafanaClient::new(
            url.to_string(),
            m.value_of("grafana-api-key")
                .map(|api_key| api_key.to_string()),
        ).map_err(CliError::ClientError)
    })
}
pub fn get_prometheus_client(m: &clap::ArgMatches) -> Result<PrometheusClient, CliError> {
    get_arg(m, "prometheus-url")
        .and_then(|url| PrometheusClient::new(url.to_string()).map_err(CliError::ClientError))
}

pub type CommandResult = Result<serde_json::Value, CliError>;

pub trait Command {
    fn get_name<'a>(&self) -> &'a str;
    fn get_cmd<'a>(&self) -> clap::App<'a, 'a>;
    fn run_cmd(&self, matches: &clap::ArgMatches) -> CommandResult;
}

pub struct App {
    pub name: String,
    pub version: String,
    pub about: String,
    pub commands: Vec<Box<Command>>,
}

impl App {
    pub fn run(&mut self) -> CommandResult {
        let mut app = ClapApp::new(self.name.clone())
            .version(self.version.as_ref())
            .about(self.about.as_ref());

        for cmd in self.commands.iter() {
            app = app.subcommand(cmd.get_cmd());
        }

        let matches = app.get_matches();
        let (cmd_name, may_cmd_matches) = matches.subcommand();

        if cmd_name == "" {
            return Err(CliError::CommandRequired);
        }

        let cmd_matches = may_cmd_matches.ok_or(CliError::CommandRequired)?;

        //let cmd_not_found_err = CliError::CommandRequired(cmd_name);

        let cmd = self
            .commands
            .iter()
            .find(|cmd| cmd.get_name() == cmd_name)
            .ok_or_else(|| CliError::InvalidCommand(cmd_name.to_string()))?;
        // .ok_or_else(|| );
        // .map(|b| (b.as_mut()).run_cmd(cmd_matches))?;

        cmd.run_cmd(cmd_matches)

        //self.run_a(cmd_matches, cmd_name)
    }
    /*
    fn run_a(mut self, matches: &clap::ArgMatches, name: &str) -> CommandResult {
        let results: Vec<CommandResult> = self
            .commands
            .iter_mut()
            .filter(|cmd| cmd.get_name() == name)
            .map(|cmd| cmd.run_cmd(matches))
            .collect();

        if results.len() == 0 {
            return Err(CliError::CommandRequired);
        }

        let result: CommandResult = match results[0] {
            Ok(ref result) => Ok(result.clone()),
            Err(ref err) => {
                let r = err.clone();

                Err(*r)
            }
        }; //first().ok_or()?;

        Ok(serde_json::Value::Null)
    }
    */
}
