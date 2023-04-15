mod timestamp;
mod uri;
mod utils;
use self::{
    timestamp::Timestamp,
    utils::{hash, merge, sign},
};
use super::types::Method;
use crate::{signature::uri::encode_uri, types::AWSProfile};
use std::collections::HashMap;
use tungstenite::{client::IntoClientRequest, handshake::client::Request, http::header::HOST};
use url::Url;

pub type Headers = HashMap<String, String>;

const AWS4_HMAC_SHA256: &str = "AWS4-HMAC-SHA256";
const X_AMZ_ALGORITHM: &str = "X-Amz-Algorithm";
const X_AMZ_DATE: &str = "X-Amz-Date";
const X_AMZ_CREDENTIAL: &str = "X-Amz-Credential";
const X_AMZ_SIGNED_HEADERS: &str = "X-Amz-SignedHeaders";
const X_AMZ_SIGNATURE: &str = "X-Amz-Signature";
const X_AMZ_SECURITY_TOKEN: &str = "X-Amz-Security-Token";
const AUTHORIZATION: &str = "authorization";
const X_AMZ_CONTENT_SHA256: &str = "x-amz-content-sha256";

// inspirations:
// - http://docs.aws.amazon.com/general/latest/gr/sigv4-create-canonical-request.html
// - https://github.com/psnszsn/aws-sign-v4/blob/a99f7f693cfc2d4da373ce0e1c28c53c8a8fe1df/src/lib.rs#L192
// - https://github.com/okigan/awscurl/tree/master/awscurl
// - https://github.com/awslabs/aws-iot-core-websockets/blob/master/src/main/java/com/awslabs/aws/iot/websockets/BasicMqttOverWebsocketsProvider.java

fn calc_signature(
    short_date: &str,
    secret_key: &str,
    region: &str,
    service: &str,
    msg: &str,
) -> String {
    let key = format!("AWS4{}", secret_key);
    let date = sign(key.as_bytes(), short_date);
    let region = sign(date.as_ref(), region);
    let service = sign(region.as_ref(), service);
    let signing_key = sign(service.as_ref(), "aws4_request");
    hex::encode(sign(signing_key.as_ref(), &msg).as_ref())
}

pub struct V4SigOptions<'a> {
    pub method: &'a Method,
    pub service: &'a str,
    pub url: &'a Url,
    pub profile: &'a AWSProfile,
}

struct V4SigBuilder<'a> {
    profile: &'a AWSProfile,
    uri: &'a str,
    query: Vec<(String, String)>,
    method: &'a Method,
    headers: &'a Headers,
    timestamp: &'a Timestamp,
    service: &'a str,
}

impl<'a> V4SigBuilder<'a> {
    fn scope(&self) -> String {
        format!(
            "{}/{}/{}/aws4_request",
            self.timestamp.date_stamp(),
            self.profile.region,
            self.service
        )
    }

    fn signed_headers(&self) -> String {
        merge(
            self.headers
                .iter()
                .map(|(key, _)| key.to_lowercase())
                .collect(),
            ";",
        )
    }

    fn canonical_headers(&self) -> String {
        merge(
            self.headers
                .iter()
                .map(|(key, value)| key.to_lowercase() + ":" + value.trim())
                .collect(),
            "\n",
        )
    }

    fn canonical_query(&self) -> String {
        merge(
            self.query
                .iter()
                .map(|(k, v)| {
                    format!(
                        "{}={}",
                        encode_uri(&k.to_string()),
                        encode_uri(&v.to_string())
                    )
                })
                .collect(),
            "&",
        )
    }

    fn signature(&self) -> String {
        let canonical_request = format!(
            "{method}\n{uri}\n{query}\n{headers}\n\n{signed}\n{sha256}",
            method = self.method,
            uri = self.uri,
            query = &self.canonical_query(),
            headers = &self.canonical_headers(),
            signed = self.signed_headers(),
            sha256 = &self.method.hash_body(),
        );

        let string_to_sign = format!(
            "{algorithm}\n{timestamp}\n{scope}\n{hash}",
            algorithm = AWS4_HMAC_SHA256,
            timestamp = self.timestamp.x_amz_date(),
            scope = self.scope(),
            hash = hash(&canonical_request)
        );

        calc_signature(
            &self.timestamp.date_stamp(),
            &self.profile.secret_key,
            &self.profile.region,
            self.service,
            &string_to_sign,
        )
    }

    fn authorization(&self) -> String {
        format!(
            "{alg} Credential={key}/{scope}, SignedHeaders={signed_headers},Signature={signature}",
            alg = AWS4_HMAC_SHA256,
            key = self.profile.access_key,
            scope = self.scope(),
            signed_headers = self.signed_headers(),
            signature = self.signature()
        )
    }

    fn credential(&self) -> String {
        format!("{}/{}", self.profile.access_key, self.scope())
    }

    fn add_query(&mut self, k: &str, v: String) {
        self.query.push((k.to_string(), v));
    }
}

pub fn sign_headers(
    headers: &mut Headers,
    V4SigOptions {
        method,
        service,
        url,
        profile,
    }: V4SigOptions,
) {
    let timestamp = &Timestamp::new();

    headers.insert("host".to_string(), url.host_str().unwrap().to_owned());

    let v4 = V4SigBuilder {
        uri: url.path().into(),
        query: url
            .query_pairs()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
        method,
        headers: &headers.clone(),
        timestamp,
        service,
        profile,
    };

    headers.insert(X_AMZ_DATE.to_string(), timestamp.x_amz_date());
    headers.insert(AUTHORIZATION.to_string(), v4.authorization());
    headers.insert(
        X_AMZ_SECURITY_TOKEN.to_string(),
        profile.session_token.to_string(),
    );
    headers.insert(X_AMZ_CONTENT_SHA256.to_string(), method.hash_body());
}

pub fn mqtt_over_websockets_request(profile: &AWSProfile, endpoint: &str) -> Request {
    let host = format!("{}:443", endpoint);

    let mut v4 = V4SigBuilder {
        uri: "/mqtt",
        query: Vec::new(),
        method: &Method::GET,
        headers: &HashMap::from([("host".to_string(), host.to_string())]),
        timestamp: &Timestamp::new(),
        service: "iotdata",
        profile,
    };

    v4.add_query(X_AMZ_ALGORITHM, AWS4_HMAC_SHA256.to_string());
    v4.add_query(X_AMZ_DATE, v4.timestamp.x_amz_date());
    v4.add_query(X_AMZ_CREDENTIAL, v4.credential());
    v4.add_query(X_AMZ_SIGNED_HEADERS, v4.signed_headers());
    v4.add_query(X_AMZ_SIGNATURE, v4.signature());
    v4.add_query(X_AMZ_SECURITY_TOKEN, profile.session_token.to_string());

    let mut request = format!("wss://{}{}?{}", endpoint, v4.uri, v4.canonical_query())
        .into_client_request()
        .unwrap();

    let headers = request.headers_mut();

    headers.insert(HOST, host.parse().unwrap());

    request
}
