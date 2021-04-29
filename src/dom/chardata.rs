// interface `CharacterData`
// definition: https://dom.spec.whatwg.org/#interface-characterdata

pub trait CharacterData {}

#[derive(Debug)]
pub struct Text {}
impl CharacterData for Text {}
