use std::time::Duration;

use futures::future::Either;
use futures::{Future, Stream};
use hyper::header::*;
use hyper::{Client, Request};
use hyper_tls::HttpsConnector;
use restson::Error;
use tokio_core::reactor::Timeout;
use url::Url;

pub use hyper::Method;
pub use restson::Query;

static VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub struct HttpClient {
    core: tokio_core::reactor::Core,
    client: Client<HttpsConnector<hyper::client::HttpConnector>>,
    timeout: Duration,
    baseurl: Url,
    headers: Option<HeaderMap>,
}

impl HttpClient {
    /// Construct new client to make HTTP requests.
    pub fn new<U: Into<String>>(
        url: U,
        headers: Option<HeaderMap>,
        timeout: Option<Duration>,
    ) -> Result<HttpClient, Error> {
        let core = tokio_core::reactor::Core::new().map_err(|_| Error::HttpClientError)?;

        let https = HttpsConnector::new(4).map_err(|_| Error::HttpClientError)?;
        let client = Client::builder().build(https);

        let baseurl = Url::parse(&url.into()).map_err(|_| Error::UrlError)?;

        Ok(HttpClient {
            core,
            client,
            // For some reason using u32::MAX or u64::MAX causes request error inside
            // tokio/hyper. Use 1 year as default which is sufficiently large for "no timeout"
            timeout: timeout.unwrap_or(Duration::from_secs(31_556_926)),
            headers,
            baseurl,
        })
    }

    pub fn get<S: Into<String>>(
        &mut self,
        path: S,
        params: Option<&Query>,
        headers: Option<HeaderMap>,
    ) -> Result<hyper::Response<String>, Error> {
        self.request(Method::GET, path, params, None, headers)
    }

    pub fn request<S: Into<String>>(
        &mut self,
        method: Method,
        path: S,
        params: Option<&Query>,
        body: Option<S>,
        headers: Option<HeaderMap>,
    ) -> Result<hyper::Response<String>, Error> {
        let mut request = Request::new(hyper::Body::empty());
        let mut url = self.baseurl.clone();

        url.set_path(&path.into());

        if let Some(params) = params {
            for &(key, item) in params.iter() {
                url.query_pairs_mut().append_pair(key, item);
            }
        }

        *request.method_mut() = method;
        *request.uri_mut() = url
            .as_str()
            .parse::<hyper::Uri>()
            .map_err(|_| Error::UrlError)?;

        *request.headers_mut() = self.combine_headers(headers)?;

        if let Some(body) = body {
            let b = body.into();

            if b != "null" {
                let len =
                    HeaderValue::from_str(&b.len().to_string()).map_err(|_| Error::RequestError)?;
                request.headers_mut().insert(CONTENT_LENGTH, len);
                request.headers_mut().insert(
                    CONTENT_TYPE,
                    HeaderValue::from_str("application/json").unwrap(),
                );
                //trace!("set request body: {}", body);
                *request.body_mut() = hyper::Body::from(b);
            }
        }

        //debug!("{} {}", req.method(), req.uri());
        //trace!("{:?}", req);

        let response = self.client.request(request).and_then(|res| {
            //trace!("response headers: {:?}", res.headers());
            let (parts, body) = res.into_parts();

            body.concat2()
                .map(|chunk| (parts, String::from_utf8_lossy(&chunk).to_string()))
        });

        let timeout =
            Timeout::new(self.timeout, &self.core.handle()).map_err(|_| Error::RequestError)?;
        let work = response.select2(timeout).then(|res| match res {
            Ok(Either::A((got, _))) => Ok(got),
            Ok(Either::B((_, _))) => Err(Error::TimeoutError),
            Err(_) => Err(Error::RequestError),
        });

        match self.core.run(work) {
            Ok((parts, body)) => Ok(hyper::Response::from_parts(parts, body)),
            Err(e) => Err(e),
        }
    }

    pub fn combine_headers(&self, headers: Option<HeaderMap>) -> Result<HeaderMap, Error> {
        let mut result = HeaderMap::new();

        let add_headers = |r: &mut HeaderMap, ref headers: Option<HeaderMap>| {
            if let Some(headers) = headers {
                for (key, ref value) in headers.iter() {
                    r.insert(key, (**value).clone());
                }
            }
        };

        add_headers(&mut result, self.headers.clone());
        add_headers(&mut result, headers);

        if !result.contains_key(USER_AGENT) {
            result.insert(
                USER_AGENT,
                HeaderValue::from_str(&("restson/".to_owned() + VERSION))
                    .map_err(|_| Error::RequestError)?,
            );
        }

        Ok(result)
    }
}
