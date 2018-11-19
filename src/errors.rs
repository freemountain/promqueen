use hyper::http::Error as HttpError;
use restson;

quick_error! {
    #[derive(Debug)]
    pub enum ClientError {
          RestError { client: &'static str, err: restson::Error } {
            cause(err)
            description("client error at")
        }
        HttpError(err: HttpError) {
            from()
            description("http error")
            cause(err)
        }
    }
}

quick_error! {
    #[derive(Debug)]
    pub enum CliError {
        ArgumentRequired(name: String) {
            description("The argument is required for this command")
            display(r#"The argument "{}" is required for this command"#, name)
            display(me) -> ("{}", me.description())
        }

        CommandRequired {
            description("A command is required")
        }
        InvalidCommand(name: String) {
            from()
            description("Could not find command")
            display(r#"Could not find command "{}""#, name)
            display(me) -> ("{}", me.description())
        }

        ArgumentParseError {
            description("A command is required")
        }

        ClientError(err: ClientError) {
            from()
            description("client error")
            cause(err)
        }


        SerdeError(err: serde_json::Error) {
            from()
            description("serde error")
            display("I/O error: {}", err)
            cause(err)
        }

        PromqlError(expression: String, message: Option<String>) {
            description("promql error")
            display("promql error {} in expression: {}",message.clone().unwrap_or("".to_string()), expression)
        }

    }
}
