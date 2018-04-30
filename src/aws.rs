use std::str::FromStr;
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use sha2::Sha256 as sha2_256;
use hmac::{Hmac, Mac};
use rustc_serialize::hex::ToHex;
use base64::encode;
use url::form_urlencoded;

fn aws_v4_canonical_request(http_method: &str, uri:&str, query_string:&str, headers:&Vec<String>, signed_headers:&str, payload_hash:&str) -> String {
    let mut input = String::new();
    input.push_str(http_method);
    input.push_str("\n");
    input.push_str(uri);
    input.push_str("\n");
    input.push_str(query_string);
    input.push_str("\n");
    for h in headers {
        input.push_str(h.as_str());
        input.push_str("\n");
    }
    input.push_str("\n");
    input.push_str(signed_headers);
    input.push_str("\n");
    input.push_str(payload_hash);

    let mut sha = Sha256::new();
    sha.input_str(input.as_str());
    sha.result_str()
}

pub fn aws_v4_get_string_to_signed(http_method: &str, uri:&str, query_string:&str, headers:&Vec<String>, signed_headers:&str, payload_hash:&str) -> String {
    let mut string_to_signed = String::from_str("AWS4-HMAC-SHA256\n").unwrap();
    string_to_signed.push_str("20150830T123600Z");
    string_to_signed.push_str("\n");
    string_to_signed.push_str("20150830/us-east-1/iam/aws4_request");
    string_to_signed.push_str("\n");
    string_to_signed.push_str(aws_v4_canonical_request(http_method, uri, query_string, headers, signed_headers, payload_hash).as_str());
    return  string_to_signed
}


// HMAC(HMAC(HMAC(HMAC("AWS4" + kSecret,"20150830"),"us-east-1"),"iam"),"aws4_request")
pub fn aws_v4_sign(secret_key: &str, data: &str) -> String {
    let mut key = String::from("AWS4");
    key.push_str(secret_key);

    let mut mac = Hmac::<sha2_256>::new(key.as_str().as_bytes());
    mac.input(b"20150830");
    let result = mac.result();
    let code_bytes = result.code();

    let mut mac1 = Hmac::<sha2_256>::new(code_bytes);
    mac1.input(b"us-east-1");
    let result1 = mac1.result();
    let code_bytes1 = result1.code();

    let mut mac2 = Hmac::<sha2_256>::new(code_bytes1);
    mac2.input(b"iam");
    let result2 = mac2.result();
    let code_bytes2 = result2.code();

    let mut mac3 = Hmac::<sha2_256>::new(code_bytes2);
    mac3.input(b"aws4_request");
    let result3 = mac3.result();
    let code_bytes3 = result3.code();

    let mut mac4 = Hmac::<sha2_256>::new(code_bytes3);
    mac4.input(data.as_bytes());
    let result4 = mac4.result();
    let code_bytes4 = result4.code();

    code_bytes4.to_hex()
}

pub fn aws_v2_get_string_to_signed(http_method: &str, host:&str, uri:&str, query_strings:&mut Vec<(&str, &str)>) -> String {
    let mut string_to_signed = String::from_str(http_method).unwrap();
    string_to_signed.push_str("\n");
    string_to_signed.push_str(host);
    string_to_signed.push_str("\n");
    string_to_signed.push_str(uri);
    string_to_signed.push_str("\n");
    query_strings.sort_by_key(|a| a.0);
    let mut encoded = form_urlencoded::Serializer::new(String::new());
    for q in query_strings{
        encoded.append_pair(q.0, q.1);
    }
    string_to_signed.push_str(encoded.finish().as_str());
    return  string_to_signed
}

pub fn aws_v2_sign(secret_key: &str, data: &str) -> String {
    let mut mac = Hmac::<sha2_256>::new(secret_key.as_bytes());
    mac.input(data.as_bytes());

    let result = mac.result();
    let code_bytes = result.code();

    encode(code_bytes)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aws_v2_get_string_to_signed() {
        let mut query_strings = vec![
            ("Timestamp", "2011-10-03T15:19:30"),
            ("AWSAccessKeyId", "AKIAIOSFODNN7EXAMPLE"),
            ("Action", "DescribeJobFlows"),
            ("SignatureMethod", "HmacSHA256"),
            ("SignatureVersion", "2"),
            ("Version", "2009-03-31")
        ];

        let string_need_signed = aws_v2_get_string_to_signed(
            "GET",
            "elasticmapreduce.amazonaws.com",
            "/", 
            &mut query_strings);

        assert_eq!(
            "GET\n\
            elasticmapreduce.amazonaws.com\n\
            /\n\
            AWSAccessKeyId=AKIAIOSFODNN7EXAMPLE&\
            Action=DescribeJobFlows&\
            SignatureMethod=HmacSHA256&\
            SignatureVersion=2&\
            Timestamp=2011-10-03T15%3A19%3A30\
            &Version=2009-03-31", string_need_signed.as_str());
    }

    #[test]
    fn test_aws_v2_sign() {
        let sig = aws_v2_sign("wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY", 
                              "GET\n\
                              elasticmapreduce.amazonaws.com\n\
                              /\n\
                              AWSAccessKeyId=AKIAIOSFODNN7EXAMPLE&\
                              Action=DescribeJobFlows&\
                              SignatureMethod=HmacSHA256&\
                              SignatureVersion=2&\
                              Timestamp=2011-10-03T15%3A19%3A30&\
                              Version=2009-03-31");
        assert_eq!("i91nKc4PWAt0JJIdXwz9HxZCJDdiy6cf/Mj6vPxyYIs=", sig.as_str());
    }

    #[test]
    fn test_aws_v4_get_string_to_signed() {
        let headers = vec![
            String::from_str("content-type:application/x-www-form-urlencoded; charset=utf-8").unwrap(),
            String::from_str("host:iam.amazonaws.com").unwrap(),
            String::from_str("x-amz-date:20150830T123600Z").unwrap()];

        let string_need_signed = aws_v4_get_string_to_signed(
                "GET",
                "/", 
                "Action=ListUsers&Version=2010-05-08", 
                &headers,
                "content-type;host;x-amz-date", 
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");

        assert_eq!(
            "AWS4-HMAC-SHA256\n\
            20150830T123600Z\n\
            20150830/us-east-1/iam/aws4_request\n\
            f536975d06c0309214f805bb90ccff089219ecd68b2577efef23edd43b7e1a59",
            string_need_signed.as_str());
    }


    #[test]
    fn test_aws_v4_sign() {
        let sig = aws_v4_sign("wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY", 
                              "AWS4-HMAC-SHA256\n\
                              20150830T123600Z\n\
                              20150830/us-east-1/iam/aws4_request\n\
                              f536975d06c0309214f805bb90ccff089219ecd68b2577efef23edd43b7e1a59");

        assert_eq!("5d672d79c15b13162d9279b0855cfba6789a8edb4c82c400e06b5924a6f2b5d7", sig.as_str());
    }



}
