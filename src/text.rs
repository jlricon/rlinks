use console::style;
pub trait ColorsExt {
    fn bold_red(&self) -> String;
    fn bold_green(&self) -> String;
    fn print_in_green(&self);
    fn print_in_red(&self);
}
impl ColorsExt for str {
    fn bold_red(self: &str) -> String {
        format!("{}", style(self).red().bold())
    }
    fn bold_green(self: &str) -> String {
        format!("{}", style(self).green().bold())
    }
    fn print_in_green(self: &str) {
        println!("{}", self.bold_green());
    }
    fn print_in_red(self: &str) {
        println!("{}", self.bold_red());
    }
}
