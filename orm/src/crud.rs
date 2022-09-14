#[allow(dead_code)]
/// Data insert builder
pub struct CRUD<P> {
    prepare: P, // database for insert
}

impl<P> CRUD<P> {
    pub fn new(prepare: P) -> Self {
        Self { prepare }
    }
}

pub mod insert;
