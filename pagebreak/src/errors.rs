use std::fmt;

#[derive(Debug, PartialEq)]
pub enum PageErrorCode {
    ParentDir,
}

pub struct PageError {
    pub relative_path: String,
    pub message: String,
    pub code: PageErrorCode,
}

impl fmt::Display for PageError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error on page {}: {}", self.relative_path, self.message)
    }
}

impl fmt::Debug for PageError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "PageError {{ relative_path: {}, message: {} }}",
            self.relative_path, self.message
        )
    }
}
