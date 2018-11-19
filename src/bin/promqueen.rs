extern crate env_logger;
extern crate promqueen;
extern crate serde_json;

use promqueen::cli::*;
use promqueen::commands::*;

use promqueen::CliError;

fn main() {
    env_logger::init();

    let mut app = App {
        name: "promqueen".to_string(),
        version: "0.1".to_string(),
        about: "Does awesome things".to_string(),
        commands: vec![
            Box::new(GetValuesCmd {}),
            Box::new(ValidateDashboardCmd {}),
            Box::new(GetAlertsCmd {}),
        ],
    };

    let result = app
        .run()
        .and_then(|value| serde_json::to_string_pretty(&value).map_err(CliError::SerdeError));

    let exit_code = match result {
        Ok(result) => {
            println!("{}", result);
            0
        }
        Err(err) => {
            eprintln!("{:?}", err);
            1
        }
    };

    std::process::exit(exit_code);
}
