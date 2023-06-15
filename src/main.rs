use lazy_static::lazy_static;
use regex::Regex;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{lookup_host, TcpSocket},
};

lazy_static! {
    static ref RE_URL:       Regex = Regex::new(r".*\.[A-z]*").unwrap();
    static ref RE_URL_PATH:  Regex = Regex::new(r"https?://[^/]+(?P<path>/[^?#]*)").unwrap();
    static ref RE_URL_QUERY: Regex = Regex::new(r"\?.*").unwrap();
}

type Error = Box<dyn std::error::Error>;

#[derive(Debug)]
struct URI<'a> {
    url: String,
    path: &'a str,
    query: &'a str,
}

impl<'a> URI<'a> {
    fn new(uri: &'a str) -> Result<URI<'a>, Error> {
        let mut url = RE_URL
            .find(uri)
            .ok_or("Problem finding url!")?
            .as_str()
            .replace("http://", "")
            .replace("https://", "");
        if url.ends_with('/') {
            url.pop();
        }
        url += ":80";

        let path = RE_URL_PATH
            .captures(uri)
            .map_or(Some(""), |path| 
                path.name("path").map_or(None, |x| Some(x.as_str()))
            )
            .ok_or("Problem finding path!")?;

        let query = RE_URL_QUERY.find(uri).map_or("", |query| query.as_str());

        let ret = Self { url, path, query };
        Ok(ret)
    }

    /// response string or error
    async fn get(self) -> Result<String, Error> {
        let ip = lookup_host(&self.url)
            .await?
            .next()
            .ok_or("Problem with url")?;
        let mut stream = TcpSocket::new_v4()?.connect(ip).await?;

        let req_msg = format!("GET {}{} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
            self.path, self.query, self.url);

        stream.write_all(req_msg.as_bytes()).await?;
        let mut buffer = Vec::new();
        stream.read_to_end(&mut buffer).await?;
        let response = String::from_utf8_lossy(&buffer);

        Ok(response.to_string())
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let url = "https://google.com/";
    let uri = URI::new(url)?;
    let resp = uri.get().await?;
    dbg!(resp);
    Ok(())
}
