use std::collections::HashSet;
use std::iter::FromIterator;

use clap::{Arg, SubCommand};

use super::super::cli::*;
use super::super::errors::*;
use super::super::usage::*;

pub struct ValidateDashboardCmd {}

impl Command for ValidateDashboardCmd {
    fn get_name<'a>(&self) -> &'a str {
        "validate-dashboard"
    }

    fn get_cmd<'a>(&self) -> clap::App<'a, 'a> {
        let mut cmd = SubCommand::with_name(self.get_name()).about("get dashboard");

        cmd = add_prometheus_args(add_grafana_args(cmd)).arg(
            Arg::with_name("uri")
                .help("the dashboard uri to validate")
                .required(true)
                .index(1),
        );

        cmd
    }

    fn run_cmd(&self, matches: &clap::ArgMatches) -> CommandResult {
        let prometheus_client = get_prometheus_client(matches)?;
        let grafana_client = get_grafana_client(matches)?;
        let uri = get_arg(matches, "uri")?;

        let metrics = prometheus_client
            .get_label_values("__name__".to_string())
            .map_err(CliError::ClientError)?;

        let dashboard = grafana_client
            .get_dashboard_by_uri(uri)
            .map_err(CliError::ClientError)?;

        let valid_metrics: HashSet<String> = HashSet::from_iter(metrics.into_iter());

        let usages = get_used_metrics_from_dashboard(dashboard);

        let inv: Vec<MetricsUsage> = usages
            .iter()
            .map(|usage| {
                let invalid_metrics: HashSet<_> = usage
                    .metrics
                    .difference(&valid_metrics)
                    .map(|metric| metric.to_owned())
                    .collect();

                MetricsUsage {
                    pointer: usage.pointer.clone(),
                    expression: usage.expression.clone(),
                    metrics: invalid_metrics,
                }
            }).filter(|usage| usage.metrics.len() > 0)
            .collect();

        serde_json::to_value(inv).map_err(CliError::SerdeError)
    }
}
