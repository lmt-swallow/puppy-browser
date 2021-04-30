use std::error::Error;

#[derive(Debug, PartialEq)]
pub struct Source {
    pub from_url: String,
    pub data: Vec<u8>,
}

pub fn fetch(url: &String) -> Result<Source, Box<Error>> {
    // TODO
    Ok(Source {
        from_url: url.to_string(),
        data: "<p>Hello World</p><p>Hello World2</p>".as_bytes().to_vec(),
    })
}
