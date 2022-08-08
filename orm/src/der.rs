pub trait Deserializer {
    fn next(&mut self, placeholder: rdbc::Placeholder) -> anyhow::Result<()>;
}
