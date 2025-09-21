use std::io::{self, Write};

/// Trait for input operations to allow for testing
pub trait Input: Send + Sync {
    fn read_line(&self) -> io::Result<String>;
}

/// Default implementation using standard input
pub struct StdInput;

impl Input for StdInput {
    fn read_line(&self) -> io::Result<String> {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Ok(input)
    }
}

/// Trait for output operations to allow for testing
pub trait Output: Send + Sync {
    fn print(&self, text: &str) -> io::Result<()>;
    fn println(&self, text: &str) -> io::Result<()>;
    fn flush(&self) -> io::Result<()>;
}

/// Default implementation using standard output
pub struct StdOutput;

impl Output for StdOutput {
    fn print(&self, text: &str) -> io::Result<()> {
        print!("{}", text);
        Ok(())
    }

    fn println(&self, text: &str) -> io::Result<()> {
        println!("{}", text);
        Ok(())
    }

    fn flush(&self) -> io::Result<()> {
        io::stdout().flush()
    }
}

/// Mock implementation for testing
#[cfg(test)]
pub mod test_helpers {
    use super::*;
    use std::sync::{Arc, Mutex};

    /// Mock input that returns predefined responses
    #[derive(Clone)]
    #[allow(dead_code)]
    pub struct MockInput {
        responses: Arc<Mutex<Vec<String>>>,
    }

    impl MockInput {
        #[allow(dead_code)]
        pub fn new(responses: Vec<&str>) -> Self {
            Self {
                responses: Arc::new(Mutex::new(
                    responses.into_iter().map(String::from).collect(),
                )),
            }
        }
    }

    impl Input for MockInput {
        fn read_line(&self) -> io::Result<String> {
            let mut responses = self.responses.lock().unwrap();
            if responses.is_empty() {
                Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "No more test responses",
                ))
            } else {
                Ok(responses.remove(0))
            }
        }
    }

    /// Mock output that captures all output
    #[derive(Clone)]
    #[allow(dead_code)]
    pub struct MockOutput {
        output: Arc<Mutex<Vec<String>>>,
    }

    impl Default for MockOutput {
        fn default() -> Self {
            Self {
                output: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    impl MockOutput {
        #[allow(dead_code)]
        pub fn new() -> Self {
            Self::default()
        }

        #[allow(dead_code)]
        pub fn get_output(&self) -> String {
            self.output.lock().unwrap().join("")
        }
    }

    impl Output for MockOutput {
        fn print(&self, text: &str) -> io::Result<()> {
            self.output.lock().unwrap().push(text.to_string());
            Ok(())
        }

        fn println(&self, text: &str) -> io::Result<()> {
            self.output.lock().unwrap().push(format!("{}\n", text));
            Ok(())
        }

        fn flush(&self) -> io::Result<()> {
            Ok(())
        }
    }
}
