use std::fmt::Display;

use colored::{ColoredString, Colorize};

use crate::RequestType;
use reqwest::{StatusCode,r#async::Response};
pub trait ColorsExt {
    fn bold_red(&self) -> ColoredString;
    fn bold_green(&self) -> ColoredString;
}
impl ColorsExt for str {
    fn bold_red(self: &str) -> ColoredString {
        self.bold().red()
    }
    fn bold_green(self: &str) -> ColoredString {
        self.bold().green()
    }
}
pub fn is_valid_status_code(x: StatusCode) -> bool {
    x.is_success() | x.is_redirection()
}
pub fn print_response(x: Response, method: RequestType) {
    if is_valid_status_code(x.status()) {
        println!("{}", response_to_msg(x, method, "valid").bold_green());
    } else {
        println!("{}", response_to_msg(x, method, "invalid").bold_red());
    }
}
pub fn print_error<T: Display>(x: T) {
    println!("{}", format!("{}", x).bold_red());
}
fn response_to_msg(resp:Response, method: RequestType, state: &str) -> String {
    format!(
        "{} is {} ({:?},{})",
        resp.url().as_str(),
        state,
        method,
        resp.status().as_str()
    )
}
