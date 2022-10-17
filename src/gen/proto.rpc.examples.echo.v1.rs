// @generated
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EchoRequest {
    #[prost(string, tag="1")]
    pub message: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EchoResponse {
    #[prost(string, tag="1")]
    pub message: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EchoMultipleRequest {
    #[prost(string, tag="1")]
    pub message: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EchoMultipleResponse {
    #[prost(string, tag="1")]
    pub message: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EchoBiDiRequest {
    #[prost(string, tag="1")]
    pub message: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EchoBiDiResponse {
    #[prost(string, tag="1")]
    pub message: ::prost::alloc::string::String,
}
/// Encoded file descriptor set for the `proto.rpc.examples.echo.v1` package
pub const FILE_DESCRIPTOR_SET: &[u8] = &[
    0x0a, 0xc6, 0x0c, 0x0a, 0x25, 0x70, 0x72, 0x6f, 0x74, 0x6f, 0x2f, 0x72, 0x70, 0x63, 0x2f, 0x65,
    0x78, 0x61, 0x6d, 0x70, 0x6c, 0x65, 0x73, 0x2f, 0x65, 0x63, 0x68, 0x6f, 0x2f, 0x76, 0x31, 0x2f,
    0x65, 0x63, 0x68, 0x6f, 0x2e, 0x70, 0x72, 0x6f, 0x74, 0x6f, 0x12, 0x1a, 0x70, 0x72, 0x6f, 0x74,
    0x6f, 0x2e, 0x72, 0x70, 0x63, 0x2e, 0x65, 0x78, 0x61, 0x6d, 0x70, 0x6c, 0x65, 0x73, 0x2e, 0x65,
    0x63, 0x68, 0x6f, 0x2e, 0x76, 0x31, 0x1a, 0x1c, 0x67, 0x6f, 0x6f, 0x67, 0x6c, 0x65, 0x2f, 0x61,
    0x70, 0x69, 0x2f, 0x61, 0x6e, 0x6e, 0x6f, 0x74, 0x61, 0x74, 0x69, 0x6f, 0x6e, 0x73, 0x2e, 0x70,
    0x72, 0x6f, 0x74, 0x6f, 0x22, 0x27, 0x0a, 0x0b, 0x45, 0x63, 0x68, 0x6f, 0x52, 0x65, 0x71, 0x75,
    0x65, 0x73, 0x74, 0x12, 0x18, 0x0a, 0x07, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65, 0x18, 0x01,
    0x20, 0x01, 0x28, 0x09, 0x52, 0x07, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65, 0x22, 0x28, 0x0a,
    0x0c, 0x45, 0x63, 0x68, 0x6f, 0x52, 0x65, 0x73, 0x70, 0x6f, 0x6e, 0x73, 0x65, 0x12, 0x18, 0x0a,
    0x07, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65, 0x18, 0x01, 0x20, 0x01, 0x28, 0x09, 0x52, 0x07,
    0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65, 0x22, 0x2f, 0x0a, 0x13, 0x45, 0x63, 0x68, 0x6f, 0x4d,
    0x75, 0x6c, 0x74, 0x69, 0x70, 0x6c, 0x65, 0x52, 0x65, 0x71, 0x75, 0x65, 0x73, 0x74, 0x12, 0x18,
    0x0a, 0x07, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65, 0x18, 0x01, 0x20, 0x01, 0x28, 0x09, 0x52,
    0x07, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65, 0x22, 0x30, 0x0a, 0x14, 0x45, 0x63, 0x68, 0x6f,
    0x4d, 0x75, 0x6c, 0x74, 0x69, 0x70, 0x6c, 0x65, 0x52, 0x65, 0x73, 0x70, 0x6f, 0x6e, 0x73, 0x65,
    0x12, 0x18, 0x0a, 0x07, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65, 0x18, 0x01, 0x20, 0x01, 0x28,
    0x09, 0x52, 0x07, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65, 0x22, 0x2b, 0x0a, 0x0f, 0x45, 0x63,
    0x68, 0x6f, 0x42, 0x69, 0x44, 0x69, 0x52, 0x65, 0x71, 0x75, 0x65, 0x73, 0x74, 0x12, 0x18, 0x0a,
    0x07, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65, 0x18, 0x01, 0x20, 0x01, 0x28, 0x09, 0x52, 0x07,
    0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65, 0x22, 0x2c, 0x0a, 0x10, 0x45, 0x63, 0x68, 0x6f, 0x42,
    0x69, 0x44, 0x69, 0x52, 0x65, 0x73, 0x70, 0x6f, 0x6e, 0x73, 0x65, 0x12, 0x18, 0x0a, 0x07, 0x6d,
    0x65, 0x73, 0x73, 0x61, 0x67, 0x65, 0x18, 0x01, 0x20, 0x01, 0x28, 0x09, 0x52, 0x07, 0x6d, 0x65,
    0x73, 0x73, 0x61, 0x67, 0x65, 0x32, 0xf4, 0x02, 0x0a, 0x0b, 0x45, 0x63, 0x68, 0x6f, 0x53, 0x65,
    0x72, 0x76, 0x69, 0x63, 0x65, 0x12, 0x80, 0x01, 0x0a, 0x04, 0x45, 0x63, 0x68, 0x6f, 0x12, 0x27,
    0x2e, 0x70, 0x72, 0x6f, 0x74, 0x6f, 0x2e, 0x72, 0x70, 0x63, 0x2e, 0x65, 0x78, 0x61, 0x6d, 0x70,
    0x6c, 0x65, 0x73, 0x2e, 0x65, 0x63, 0x68, 0x6f, 0x2e, 0x76, 0x31, 0x2e, 0x45, 0x63, 0x68, 0x6f,
    0x52, 0x65, 0x71, 0x75, 0x65, 0x73, 0x74, 0x1a, 0x28, 0x2e, 0x70, 0x72, 0x6f, 0x74, 0x6f, 0x2e,
    0x72, 0x70, 0x63, 0x2e, 0x65, 0x78, 0x61, 0x6d, 0x70, 0x6c, 0x65, 0x73, 0x2e, 0x65, 0x63, 0x68,
    0x6f, 0x2e, 0x76, 0x31, 0x2e, 0x45, 0x63, 0x68, 0x6f, 0x52, 0x65, 0x73, 0x70, 0x6f, 0x6e, 0x73,
    0x65, 0x22, 0x25, 0x82, 0xd3, 0xe4, 0x93, 0x02, 0x1f, 0x3a, 0x01, 0x2a, 0x22, 0x1a, 0x2f, 0x72,
    0x70, 0x63, 0x2f, 0x65, 0x78, 0x61, 0x6d, 0x70, 0x6c, 0x65, 0x73, 0x2f, 0x65, 0x63, 0x68, 0x6f,
    0x2f, 0x76, 0x31, 0x2f, 0x65, 0x63, 0x68, 0x6f, 0x12, 0x75, 0x0a, 0x0c, 0x45, 0x63, 0x68, 0x6f,
    0x4d, 0x75, 0x6c, 0x74, 0x69, 0x70, 0x6c, 0x65, 0x12, 0x2f, 0x2e, 0x70, 0x72, 0x6f, 0x74, 0x6f,
    0x2e, 0x72, 0x70, 0x63, 0x2e, 0x65, 0x78, 0x61, 0x6d, 0x70, 0x6c, 0x65, 0x73, 0x2e, 0x65, 0x63,
    0x68, 0x6f, 0x2e, 0x76, 0x31, 0x2e, 0x45, 0x63, 0x68, 0x6f, 0x4d, 0x75, 0x6c, 0x74, 0x69, 0x70,
    0x6c, 0x65, 0x52, 0x65, 0x71, 0x75, 0x65, 0x73, 0x74, 0x1a, 0x30, 0x2e, 0x70, 0x72, 0x6f, 0x74,
    0x6f, 0x2e, 0x72, 0x70, 0x63, 0x2e, 0x65, 0x78, 0x61, 0x6d, 0x70, 0x6c, 0x65, 0x73, 0x2e, 0x65,
    0x63, 0x68, 0x6f, 0x2e, 0x76, 0x31, 0x2e, 0x45, 0x63, 0x68, 0x6f, 0x4d, 0x75, 0x6c, 0x74, 0x69,
    0x70, 0x6c, 0x65, 0x52, 0x65, 0x73, 0x70, 0x6f, 0x6e, 0x73, 0x65, 0x22, 0x00, 0x30, 0x01, 0x12,
    0x6b, 0x0a, 0x08, 0x45, 0x63, 0x68, 0x6f, 0x42, 0x69, 0x44, 0x69, 0x12, 0x2b, 0x2e, 0x70, 0x72,
    0x6f, 0x74, 0x6f, 0x2e, 0x72, 0x70, 0x63, 0x2e, 0x65, 0x78, 0x61, 0x6d, 0x70, 0x6c, 0x65, 0x73,
    0x2e, 0x65, 0x63, 0x68, 0x6f, 0x2e, 0x76, 0x31, 0x2e, 0x45, 0x63, 0x68, 0x6f, 0x42, 0x69, 0x44,
    0x69, 0x52, 0x65, 0x71, 0x75, 0x65, 0x73, 0x74, 0x1a, 0x2c, 0x2e, 0x70, 0x72, 0x6f, 0x74, 0x6f,
    0x2e, 0x72, 0x70, 0x63, 0x2e, 0x65, 0x78, 0x61, 0x6d, 0x70, 0x6c, 0x65, 0x73, 0x2e, 0x65, 0x63,
    0x68, 0x6f, 0x2e, 0x76, 0x31, 0x2e, 0x45, 0x63, 0x68, 0x6f, 0x42, 0x69, 0x44, 0x69, 0x52, 0x65,
    0x73, 0x70, 0x6f, 0x6e, 0x73, 0x65, 0x22, 0x00, 0x28, 0x01, 0x30, 0x01, 0x42, 0x2e, 0x5a, 0x2c,
    0x67, 0x6f, 0x2e, 0x76, 0x69, 0x61, 0x6d, 0x2e, 0x63, 0x6f, 0x6d, 0x2f, 0x75, 0x74, 0x69, 0x6c,
    0x73, 0x2f, 0x70, 0x72, 0x6f, 0x74, 0x6f, 0x2f, 0x72, 0x70, 0x63, 0x2f, 0x65, 0x78, 0x61, 0x6d,
    0x70, 0x6c, 0x65, 0x73, 0x2f, 0x65, 0x63, 0x68, 0x6f, 0x2f, 0x76, 0x31, 0x4a, 0xa2, 0x06, 0x0a,
    0x06, 0x12, 0x04, 0x00, 0x00, 0x2c, 0x01, 0x0a, 0x08, 0x0a, 0x01, 0x0c, 0x12, 0x03, 0x00, 0x00,
    0x12, 0x0a, 0x08, 0x0a, 0x01, 0x08, 0x12, 0x03, 0x01, 0x00, 0x43, 0x0a, 0x09, 0x0a, 0x02, 0x08,
    0x0b, 0x12, 0x03, 0x01, 0x00, 0x43, 0x0a, 0x08, 0x0a, 0x01, 0x02, 0x12, 0x03, 0x03, 0x00, 0x23,
    0x0a, 0x09, 0x0a, 0x02, 0x03, 0x00, 0x12, 0x03, 0x05, 0x00, 0x26, 0x0a, 0x0a, 0x0a, 0x02, 0x04,
    0x00, 0x12, 0x04, 0x07, 0x00, 0x09, 0x01, 0x0a, 0x0a, 0x0a, 0x03, 0x04, 0x00, 0x01, 0x12, 0x03,
    0x07, 0x08, 0x13, 0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x00, 0x02, 0x00, 0x12, 0x03, 0x08, 0x02, 0x15,
    0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x00, 0x05, 0x12, 0x03, 0x08, 0x02, 0x08, 0x0a, 0x0c,
    0x0a, 0x05, 0x04, 0x00, 0x02, 0x00, 0x01, 0x12, 0x03, 0x08, 0x09, 0x10, 0x0a, 0x0c, 0x0a, 0x05,
    0x04, 0x00, 0x02, 0x00, 0x03, 0x12, 0x03, 0x08, 0x13, 0x14, 0x0a, 0x0a, 0x0a, 0x02, 0x04, 0x01,
    0x12, 0x04, 0x0b, 0x00, 0x0d, 0x01, 0x0a, 0x0a, 0x0a, 0x03, 0x04, 0x01, 0x01, 0x12, 0x03, 0x0b,
    0x08, 0x14, 0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x01, 0x02, 0x00, 0x12, 0x03, 0x0c, 0x02, 0x15, 0x0a,
    0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02, 0x00, 0x05, 0x12, 0x03, 0x0c, 0x02, 0x08, 0x0a, 0x0c, 0x0a,
    0x05, 0x04, 0x01, 0x02, 0x00, 0x01, 0x12, 0x03, 0x0c, 0x09, 0x10, 0x0a, 0x0c, 0x0a, 0x05, 0x04,
    0x01, 0x02, 0x00, 0x03, 0x12, 0x03, 0x0c, 0x13, 0x14, 0x0a, 0x0a, 0x0a, 0x02, 0x04, 0x02, 0x12,
    0x04, 0x0f, 0x00, 0x11, 0x01, 0x0a, 0x0a, 0x0a, 0x03, 0x04, 0x02, 0x01, 0x12, 0x03, 0x0f, 0x08,
    0x1b, 0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x02, 0x02, 0x00, 0x12, 0x03, 0x10, 0x02, 0x15, 0x0a, 0x0c,
    0x0a, 0x05, 0x04, 0x02, 0x02, 0x00, 0x05, 0x12, 0x03, 0x10, 0x02, 0x08, 0x0a, 0x0c, 0x0a, 0x05,
    0x04, 0x02, 0x02, 0x00, 0x01, 0x12, 0x03, 0x10, 0x09, 0x10, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x02,
    0x02, 0x00, 0x03, 0x12, 0x03, 0x10, 0x13, 0x14, 0x0a, 0x0a, 0x0a, 0x02, 0x04, 0x03, 0x12, 0x04,
    0x13, 0x00, 0x15, 0x01, 0x0a, 0x0a, 0x0a, 0x03, 0x04, 0x03, 0x01, 0x12, 0x03, 0x13, 0x08, 0x1c,
    0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x03, 0x02, 0x00, 0x12, 0x03, 0x14, 0x02, 0x15, 0x0a, 0x0c, 0x0a,
    0x05, 0x04, 0x03, 0x02, 0x00, 0x05, 0x12, 0x03, 0x14, 0x02, 0x08, 0x0a, 0x0c, 0x0a, 0x05, 0x04,
    0x03, 0x02, 0x00, 0x01, 0x12, 0x03, 0x14, 0x09, 0x10, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x03, 0x02,
    0x00, 0x03, 0x12, 0x03, 0x14, 0x13, 0x14, 0x0a, 0x0a, 0x0a, 0x02, 0x04, 0x04, 0x12, 0x04, 0x17,
    0x00, 0x19, 0x01, 0x0a, 0x0a, 0x0a, 0x03, 0x04, 0x04, 0x01, 0x12, 0x03, 0x17, 0x08, 0x17, 0x0a,
    0x0b, 0x0a, 0x04, 0x04, 0x04, 0x02, 0x00, 0x12, 0x03, 0x18, 0x02, 0x15, 0x0a, 0x0c, 0x0a, 0x05,
    0x04, 0x04, 0x02, 0x00, 0x05, 0x12, 0x03, 0x18, 0x02, 0x08, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x04,
    0x02, 0x00, 0x01, 0x12, 0x03, 0x18, 0x09, 0x10, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x04, 0x02, 0x00,
    0x03, 0x12, 0x03, 0x18, 0x13, 0x14, 0x0a, 0x0a, 0x0a, 0x02, 0x04, 0x05, 0x12, 0x04, 0x1b, 0x00,
    0x1d, 0x01, 0x0a, 0x0a, 0x0a, 0x03, 0x04, 0x05, 0x01, 0x12, 0x03, 0x1b, 0x08, 0x18, 0x0a, 0x0b,
    0x0a, 0x04, 0x04, 0x05, 0x02, 0x00, 0x12, 0x03, 0x1c, 0x02, 0x15, 0x0a, 0x0c, 0x0a, 0x05, 0x04,
    0x05, 0x02, 0x00, 0x05, 0x12, 0x03, 0x1c, 0x02, 0x08, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x05, 0x02,
    0x00, 0x01, 0x12, 0x03, 0x1c, 0x09, 0x10, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x05, 0x02, 0x00, 0x03,
    0x12, 0x03, 0x1c, 0x13, 0x14, 0x0a, 0x0a, 0x0a, 0x02, 0x06, 0x00, 0x12, 0x04, 0x1f, 0x00, 0x2c,
    0x01, 0x0a, 0x0a, 0x0a, 0x03, 0x06, 0x00, 0x01, 0x12, 0x03, 0x1f, 0x08, 0x13, 0x0a, 0x0c, 0x0a,
    0x04, 0x06, 0x00, 0x02, 0x00, 0x12, 0x04, 0x20, 0x02, 0x25, 0x03, 0x0a, 0x0c, 0x0a, 0x05, 0x06,
    0x00, 0x02, 0x00, 0x01, 0x12, 0x03, 0x20, 0x06, 0x0a, 0x0a, 0x0c, 0x0a, 0x05, 0x06, 0x00, 0x02,
    0x00, 0x02, 0x12, 0x03, 0x20, 0x0b, 0x16, 0x0a, 0x0c, 0x0a, 0x05, 0x06, 0x00, 0x02, 0x00, 0x03,
    0x12, 0x03, 0x20, 0x21, 0x2d, 0x0a, 0x0d, 0x0a, 0x05, 0x06, 0x00, 0x02, 0x00, 0x04, 0x12, 0x04,
    0x21, 0x04, 0x24, 0x0a, 0x0a, 0x11, 0x0a, 0x09, 0x06, 0x00, 0x02, 0x00, 0x04, 0xb0, 0xca, 0xbc,
    0x22, 0x12, 0x04, 0x21, 0x04, 0x24, 0x0a, 0x0a, 0x0c, 0x0a, 0x04, 0x06, 0x00, 0x02, 0x01, 0x12,
    0x04, 0x27, 0x02, 0x28, 0x03, 0x0a, 0x0c, 0x0a, 0x05, 0x06, 0x00, 0x02, 0x01, 0x01, 0x12, 0x03,
    0x27, 0x06, 0x12, 0x0a, 0x0c, 0x0a, 0x05, 0x06, 0x00, 0x02, 0x01, 0x02, 0x12, 0x03, 0x27, 0x13,
    0x26, 0x0a, 0x0c, 0x0a, 0x05, 0x06, 0x00, 0x02, 0x01, 0x06, 0x12, 0x03, 0x27, 0x31, 0x37, 0x0a,
    0x0c, 0x0a, 0x05, 0x06, 0x00, 0x02, 0x01, 0x03, 0x12, 0x03, 0x27, 0x38, 0x4c, 0x0a, 0x0c, 0x0a,
    0x04, 0x06, 0x00, 0x02, 0x02, 0x12, 0x04, 0x2a, 0x02, 0x2b, 0x03, 0x0a, 0x0c, 0x0a, 0x05, 0x06,
    0x00, 0x02, 0x02, 0x01, 0x12, 0x03, 0x2a, 0x06, 0x0e, 0x0a, 0x0c, 0x0a, 0x05, 0x06, 0x00, 0x02,
    0x02, 0x05, 0x12, 0x03, 0x2a, 0x0f, 0x15, 0x0a, 0x0c, 0x0a, 0x05, 0x06, 0x00, 0x02, 0x02, 0x02,
    0x12, 0x03, 0x2a, 0x16, 0x25, 0x0a, 0x0c, 0x0a, 0x05, 0x06, 0x00, 0x02, 0x02, 0x06, 0x12, 0x03,
    0x2a, 0x30, 0x36, 0x0a, 0x0c, 0x0a, 0x05, 0x06, 0x00, 0x02, 0x02, 0x03, 0x12, 0x03, 0x2a, 0x37,
    0x47, 0x62, 0x06, 0x70, 0x72, 0x6f, 0x74, 0x6f, 0x33,
];
include!("proto.rpc.examples.echo.v1.tonic.rs");
// @@protoc_insertion_point(module)