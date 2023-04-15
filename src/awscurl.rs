use std::collections::HashMap;

use crate::{
    signature::{mqtt_over_websockets_request, sign_headers, V4SigOptions},
    types::{AWSCurlError, AWSIotResult, AWSProfile, Body, Method},
};
use tungstenite::{handshake::client::Request, connect};

pub struct AWSCurl {
    profile: AWSProfile,
}

impl AWSCurl {
    pub fn new(profile: &AWSProfile) -> AWSCurl {
        AWSCurl {
            profile: profile.clone(),
        }
    }

    fn curl(&self, service: &str, method: &Method, url: &str) -> AWSIotResult<String> {
        let mut request = match method {
            Method::POST(_) => ureq::post(&url),
            Method::DELETE => ureq::delete(&url),
            Method::GET => ureq::get(&url),
        };

        let parsed_url = &url.parse().map_err(AWSCurlError::new)?;

        let mut headers = HashMap::from([
            ("accept".to_string(), "application/xml".to_string()),
            ("content-type".to_string(), "application/json".to_string()),
        ]);

        sign_headers(
            &mut headers,
            V4SigOptions {
                method,
                service,
                url: parsed_url,
                profile: &self.profile,
            },
        );

        for (k, v) in headers {
            request = request.set(&k, &v);
        }

        let response = match method {
            Method::POST(Body::String(x)) => request.send_string(&x),
            Method::POST(Body::Binary(x)) => request.send_bytes(&x),
            _ => request.call(),
        };

        match response {
            Ok(x) => x.into_string().map_err(AWSCurlError::new),
            Err(ureq::Error::Status(code, res)) => Err(AWSCurlError::new(format!(
                "{}: {}",
                code.to_string(),
                res.into_string().unwrap_or_default()
            ))),
            Err(err) => Err(AWSCurlError::new(err.to_string())),
        }
    }

    pub fn http_request(&self, method: &Method, url: &str) -> AWSIotResult<String> {
        self.curl("execute-api", method, &url)
    }

    pub fn publish_mqtt_over_https(
        &self,
        endpoint: String,
        topic: &str,
        value: Vec<u8>,
    ) -> AWSIotResult<()> {
        self.curl(
            "iotdevicegateway",
            &Method::POST(Body::Binary(value)),
            &format!("https://{}/topics/{}?qos=1", endpoint, topic),
        )?;
        Ok(())
    }

    pub fn mqtt_over_ws_request(&self, endpoint: &str) -> Request {
        mqtt_over_websockets_request(&self.profile, &endpoint)
    }

    pub fn publish_mqtt_over_wss(
        &self,
        endpoint: String,
        _topic: &str,
        _value: Vec<u8>,
    ) -> AWSIotResult<()> {
        let request = self.mqtt_over_ws_request(&endpoint);
        let (mut socket, response) = connect(request)
            .map_err(|x| AWSCurlError::new(format!("{:?}\n{}", x, x.to_string())))?;
        /*
        TODO: implement sending message
        */
        Ok(())
    }
}
