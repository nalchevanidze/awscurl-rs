use awscurl::{AWSCurl, AWSProfile, Method};

fn main() {
    let profile = AWSProfile::from_env().expect("can't read aws credentials");
    let awscurl = AWSCurl::new(&profile);
    awscurl.http_request(&Method::GET, "https://blog.com/users")
        .expect("can't fetch users");
}
