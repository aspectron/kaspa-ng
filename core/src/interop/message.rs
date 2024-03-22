use crate::imports::*;

#[derive(Debug, Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
pub enum Request {
    Test { data: String },
    Connect {},
    SignMessage { message: String },
    CloseWindow,
}

// #[repr(u64)]
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[serde(tag = "type", content = "data")]
pub enum Response {
    Test { response: String },
    Connect { address: String },
    SignMessage { signature: String },
    Canceled { error: String },
}
// pub enum Response {
//     Test(TestResponse),
//     Connect(ConnectResponse),
//     SignMessage(SignMessageResponse),
// }
// impl Response{
//     pub fn data(&self)->String{
//         match self {
//             Response::Test(r)=>serde_json::to_string(r).unwrap(),
//             Response::Connect(r)=>serde_json::to_string(r).unwrap(),
//             Response::SignMessage(r)=>serde_json::to_string(r).unwrap(),
//         }
//     }
// }
// impl From<TestResponse> for Response {
//     fn from(value: TestResponse) -> Self {
//         Self::Test(value)
//     }
// }
// impl From<ConnectResponse> for Response {
//     fn from(value: ConnectResponse) -> Self {
//         Self::Connect(value)
//     }
// }
// impl From<SignMessageResponse> for Response {
//     fn from(value: SignMessageResponse) -> Self {
//         Self::SignMessage(value)
//     }
// }

// #[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
// pub struct TestRequest {
//     pub data: String,
// }

// #[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
// pub struct TestResponse {
//     pub response: String,
// }

// #[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
// pub struct ConnectRequest {
//     // pub request: String,
// }

// #[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
// pub struct ConnectResponse {
//     pub address: String,
// }

// #[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
// pub struct SignMessageRequest {
//     // pub message: String,
// }

// #[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
// pub struct SignMessageResponse {
//     // pub request: String,
// }
