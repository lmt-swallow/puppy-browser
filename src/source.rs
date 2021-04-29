#[derive(Debug, PartialEq)]
pub struct Source {
    pub from_url: String,
    pub data: Vec<u8>,
}

pub fn fetch(url: String) -> Source {
    // TODO
    Source {
        from_url: url.to_string(),
        data: "<p>Hello World</p>".as_bytes().to_vec(),
    }
}
