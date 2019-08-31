use colored::{ColoredString, Colorize};

pub trait ColorsExt {
    fn bold_red(&self) -> ColoredString;
    fn bold_green(&self) -> ColoredString;
    fn print_in_green(&self);
    fn print_in_red(&self);
}
impl ColorsExt for str {
    fn bold_red(self: &str) -> ColoredString {
        self.bold().red()
    }
    fn bold_green(self: &str) -> ColoredString {
        self.bold().green()
    }
    fn print_in_green(self: &str) {
        println!("{}", self.bold_green());
    }
    fn print_in_red(self: &str) {
        println!("{}", self.bold_red());
    }
}
