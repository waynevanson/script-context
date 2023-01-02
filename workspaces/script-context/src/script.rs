/// Name of a script broken down into its components.
///
/// example:
/// let script = Script {
///     lifecycle: "postinstall".to_string(),
///     delimiter: ":".to_string(),
///     suffix: "project".to_string()
/// }
///
/// assert_eq!(script.to_string(), "postinstall:project".to_string())
#[derive(Debug, PartialEq)]
pub struct Script {
    pub lifecycle: String,
    pub delimiter: char,
    pub suffix: String,
}

impl ToString for Script {
    fn to_string(&self) -> String {
        self.lifecycle.to_string() + &self.delimiter.to_string() + &self.suffix
    }
}
