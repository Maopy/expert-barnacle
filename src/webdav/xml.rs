use chrono::{DateTime, Utc};
use quick_xml::events::Event;
use quick_xml::Reader;

use super::types::{DavError, DavResource};

/// The XML body sent with PROPFIND requests to retrieve all properties.
pub const PROPFIND_ALL_PROPS: &str = r#"<?xml version="1.0" encoding="utf-8" ?>
<D:propfind xmlns:D="DAV:">
  <D:allprop/>
</D:propfind>"#;

/// Parse a WebDAV multistatus XML response into a list of DavResource.
pub fn parse_multistatus(xml: &str) -> Result<Vec<DavResource>, DavError> {
    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);

    let mut resources = Vec::new();
    let mut buf = Vec::new();

    // Current parsing state
    let mut in_response = false;
    let mut in_propstat = false;
    let mut in_prop = false;
    let mut current_tag = String::new();

    // Current resource being built
    let mut href = String::new();
    let mut display_name: Option<String> = None;
    let mut is_collection = false;
    let mut content_length: Option<u64> = None;
    let mut content_type: Option<String> = None;
    let mut last_modified: Option<DateTime<Utc>> = None;
    let mut etag: Option<String> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let local_name = local_name(e.name().as_ref());
                match local_name {
                    "response" => {
                        in_response = true;
                        href.clear();
                        display_name = None;
                        is_collection = false;
                        content_length = None;
                        content_type = None;
                        last_modified = None;
                        etag = None;
                    }
                    "propstat" => in_propstat = true,
                    "prop" if in_propstat => in_prop = true,
                    "collection" if in_prop => is_collection = true,
                    other => {
                        current_tag = other.to_string();
                    }
                }
            }
            Ok(Event::Text(ref e)) => {
                if !in_response {
                    continue;
                }
                let text = e.unescape().unwrap_or_default().to_string();
                if text.is_empty() {
                    continue;
                }

                if current_tag == "href" && !in_prop {
                    href = text;
                } else if in_prop {
                    match current_tag.as_str() {
                        "displayname" => display_name = Some(text),
                        "getcontentlength" => content_length = text.parse().ok(),
                        "getcontenttype" => content_type = Some(text),
                        "getlastmodified" => {
                            last_modified = parse_http_date(&text);
                        }
                        "getetag" => {
                            etag = Some(text.trim_matches('"').to_string());
                        }
                        _ => {}
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let local_name = local_name(e.name().as_ref());
                match local_name {
                    "response" => {
                        if !href.is_empty() {
                            resources.push(DavResource {
                                href: href.clone(),
                                display_name: display_name.clone(),
                                is_collection,
                                content_length,
                                content_type: content_type.clone(),
                                last_modified,
                                etag: etag.clone(),
                            });
                        }
                        in_response = false;
                    }
                    "propstat" => in_propstat = false,
                    "prop" => in_prop = false,
                    _ => {}
                }
                current_tag.clear();
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(DavError::XmlParse(format!("XML error at {}: {}", reader.buffer_position(), e))),
            _ => {}
        }
        buf.clear();
    }

    Ok(resources)
}

/// Extract local name from a possibly-namespaced XML tag.
/// e.g. "DAV:response" -> "response", "D:href" -> "href"
fn local_name(name: &[u8]) -> &str {
    let s = std::str::from_utf8(name).unwrap_or("");
    if let Some(pos) = s.rfind(':') {
        &s[pos + 1..]
    } else {
        s
    }
}

/// Parse an HTTP-date string (RFC 2616 / RFC 7231) into a DateTime<Utc>.
fn parse_http_date(s: &str) -> Option<DateTime<Utc>> {
    // Try RFC 2822 format (most common in WebDAV)
    if let Ok(dt) = DateTime::parse_from_rfc2822(s) {
        return Some(dt.with_timezone(&Utc));
    }
    // Try common HTTP date format: "Mon, 01 Jan 2024 00:00:00 GMT"
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%a, %d %b %Y %H:%M:%S GMT") {
        return Some(dt.and_utc());
    }
    // Try ISO 8601
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt.with_timezone(&Utc));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_multistatus() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<D:multistatus xmlns:D="DAV:">
  <D:response>
    <D:href>/dav/</D:href>
    <D:propstat>
      <D:prop>
        <D:displayname>Root</D:displayname>
        <D:resourcetype><D:collection/></D:resourcetype>
      </D:prop>
      <D:status>HTTP/1.1 200 OK</D:status>
    </D:propstat>
  </D:response>
  <D:response>
    <D:href>/dav/test.txt</D:href>
    <D:propstat>
      <D:prop>
        <D:displayname>test.txt</D:displayname>
        <D:getcontentlength>1234</D:getcontentlength>
        <D:getcontenttype>text/plain</D:getcontenttype>
        <D:getlastmodified>Mon, 01 Jan 2024 00:00:00 GMT</D:getlastmodified>
        <D:getetag>"abc123"</D:getetag>
        <D:resourcetype/>
      </D:prop>
      <D:status>HTTP/1.1 200 OK</D:status>
    </D:propstat>
  </D:response>
</D:multistatus>"#;

        let resources = parse_multistatus(xml).unwrap();
        assert_eq!(resources.len(), 2);

        assert_eq!(resources[0].href, "/dav/");
        assert!(resources[0].is_collection);
        assert_eq!(resources[0].name(), "Root");

        assert_eq!(resources[1].href, "/dav/test.txt");
        assert!(!resources[1].is_collection);
        assert_eq!(resources[1].content_length, Some(1234));
        assert_eq!(resources[1].etag.as_deref(), Some("abc123"));
    }

    #[test]
    fn test_local_name() {
        assert_eq!(local_name(b"DAV:response"), "response");
        assert_eq!(local_name(b"D:href"), "href");
        assert_eq!(local_name(b"href"), "href");
    }
}
