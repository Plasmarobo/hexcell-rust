use embedded_error_chain::ErrorCategory;

#[derive(Clone, Copy, ErrorCategory)]
#[repr(u8)]
pub enum CoreError {
    CommandError,
    PatternError,
}

#[derive(Clone, Copy, ErrorCategory)]
#[error_category(links(CoreError))]
#[repr(u8)]
pub enum PatternError {
    InvalidPatternError,
    PatternSizeError,
    PatternCountError,
    InvalidCursorError,
}
