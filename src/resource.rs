use std::error::Error;

#[derive(Debug, PartialEq)]
pub struct Resource {
    pub from_url: String,
    pub data: Vec<u8>,
}

pub fn fetch(url: &String) -> Result<Resource, Box<Error>> {
    // TODO
    Ok(Resource {
        from_url: url.to_string(),
        data: "<p>Hello World</p><p>Hello World2</p>".as_bytes().to_vec(),
    })
}
