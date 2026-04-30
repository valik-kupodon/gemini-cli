pub trait Feature {
    fn execute(&self, full_response: &str) -> Result<(), Box<dyn std::error::Error>>;
}
