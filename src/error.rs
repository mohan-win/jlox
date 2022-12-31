pub fn error(line: usize, message: &str) {
    report(line, "", message)
}

pub fn report(line: usize, where_in: &str, message: &str) {
    eprintln!("[Line {}] Error {}: {}", line, where_in, message)
}
