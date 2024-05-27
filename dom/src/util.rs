pub fn is_alpha(ch: char) -> bool {
    // '_' is allowed in variable names, be sure to capture this
    ch.is_ascii_lowercase() || ch.is_ascii_uppercase() || ch == '_'
}

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
