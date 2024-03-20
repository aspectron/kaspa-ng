use crate::imports::*;

#[derive(Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct TestRequest {
    // pub request: String,
}

#[derive(Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct TestResponse {
    // pub response: String,
}

#[derive(Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ConnectRequest {
    // pub request: String,
}

#[derive(Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ConnectResponse {
    // pub request: String,
}

#[derive(Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct SignMessageRequest {
    // pub message: String,
}

#[derive(Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct SignMessageResponse {
    // pub request: String,
}
