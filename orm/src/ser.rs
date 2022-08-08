pub trait Serializer {
    fn next(&mut self, placeholder: rdbc::Placeholder) -> anyhow::Result<()>;
}
