use std::ascii::AsciiExt;
use std::result::Result;

use std::convert::AsRef;

#[derive(Debug)]
#[derive(PartialEq)]
pub enum HttpMethod {
    Options,
    Get,
    Header,
    Post,
    Put,
    Delete,
    Trace,
    Extension(String)
}

#[derive(Debug)]
#[derive(PartialEq)]
pub enum HttpHeaderName {
    Accept,
    AcceptCharset,
    AcceptEncoding,
    Host,
    Referer,
    UserAgent,
    Custom(String)
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct HttpHeader {
    name:  HttpHeaderName,
    value: String
}

trait HttpHeaderSplitTrim {
    fn split_trim(&self, pattern: &str) -> Vec<&str>;
    fn splitn_trim(&self, n: usize, pattern: &str) -> Vec<&str>;
}

impl HttpHeaderSplitTrim for str {
    fn split_trim(&self, pattern: &str) -> Vec<&str> {
        self.split(pattern).map(|s| s.trim()).collect()
    }
    fn splitn_trim(&self, n: usize, pattern: &str) -> Vec<&str> {
        self.splitn(n, pattern).map(|s| s.trim()).collect()
    }
}

impl HttpHeader {
    pub fn new(line: &str) -> Result<HttpHeader, String> {
        let parts = line.splitn_trim(2, ":");
        if parts.len() != 2 {
            return Err(
                format!("Too many parts after line split: {}", parts.len()))
        }
        let name  = parts[0].to_ascii_lowercase();
        let value = parts[1];
        fn build_request(
            name:  HttpHeaderName,
            value: &str
        ) -> Result<HttpHeader, String> {
            Ok(HttpHeader {
                name:  name,
                value: value.to_string()
            })
        }
        if name == "accept" {
            build_request(HttpHeaderName::Accept, value)
        } else if name == "accept-charset" {
            build_request(HttpHeaderName::AcceptCharset, value)
        } else if name == "accept-encoding" {
            build_request(HttpHeaderName::AcceptEncoding, value)
        } else if name == "host" {
            build_request(HttpHeaderName::Host, value)
        } else if name == "user-agent" {
            build_request(HttpHeaderName::UserAgent, value)
        } else if name == "referer" {
            build_request(HttpHeaderName::Referer, value)
        } else {
            build_request(HttpHeaderName::Custom(parts[0].to_string()), value)
        }
    }
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct HttpRequest {
    method:  HttpMethod,
    uri:     String,
    headers: Vec<HttpHeader>
}

impl HttpRequest {
    pub fn new(header: &str) -> Result<HttpRequest, String> {
        let lines = header.split_trim("\r\n");
        if lines.len() == 0 {
            return Result::Err(format!("No CRLF in request: {}", header))
        }
        let request_line: Vec<&str> = lines[0].split_whitespace().collect();
        let method = match request_line[0].to_ascii_lowercase().as_ref() {
            "options" => HttpMethod::Options,
            "get"     => HttpMethod::Get,
            "header"  => HttpMethod::Header,
            "post"    => HttpMethod::Post,
            "put"     => HttpMethod::Put,
            "delete"  => HttpMethod::Delete,
            "trace"   => HttpMethod::Trace,
            x         => HttpMethod::Extension(x.to_string())
        };
        let uri = request_line[1].to_string();
        //TODO(efuquen): Ignoring any header that throws error.  Should
        //probably log this.
        let headers = (&lines[1 ..]).iter().filter_map(|l| {
            HttpHeader::new(l).ok()
        }).collect();

        Result::Ok(HttpRequest { method: method, uri: uri, headers: headers })
    }
}

#[cfg(test)]
mod tests {
    use super::HttpMethod;
    use super::HttpRequest;
    use super::HttpHeader;
    use super::HttpHeaderName;

    fn assert_header_eq(
        header_str: &str, name: HttpHeaderName, value: &str) {
        let header = HttpHeader::new(header_str).unwrap();
        assert_eq!(name,  header.name);
        assert_eq!(value, header.value);
    }

    #[test]
    fn http_request_headers_basic() {
        assert_header_eq("Accept: audio/*; q=0.2, audio/basic",
                         HttpHeaderName::Accept,
                         "audio/*; q=0.2, audio/basic");
        assert_header_eq("Accept-Charset: iso-8859-5, unicode-1-1;q=0.8",
                         HttpHeaderName::AcceptCharset,
                         "iso-8859-5, unicode-1-1;q=0.8");
        assert_header_eq("Referer: http://www.w3.org/hypertext/DataSources/Overview.html",
                         HttpHeaderName::Referer,
                         "http://www.w3.org/hypertext/DataSources/Overview.html");
        assert_header_eq("User-Agent: CERN-LineMode/2.15 libwww/2.17b3",
                         HttpHeaderName::UserAgent,
                         "CERN-LineMode/2.15 libwww/2.17b3");
        /*
        assert_header_eq("",
                         HttpHeaderName::,
                         "");
        */

    }

            #[test]
    fn http_get_request() {
        let get_request_str = "GET /some/path HTTP/1.1\r\n\
                               Host: http://rsproxy.com\r\n\
                               Accept: text/html\r\n";
        let get_request = HttpRequest::new(get_request_str).unwrap();
        assert_eq!(HttpMethod::Get, get_request.method);
        assert_eq!("/some/path", get_request.uri);
        let host_header: &HttpHeader = &(get_request.headers)[0];
        assert_eq!(HttpHeaderName::Host, host_header.name);
        assert_eq!("http://rsproxy.com", host_header.value);
    }
}

