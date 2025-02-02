use bincode::{config, Encode};

static HEADER_SIZE: usize = 9;
static HTTP_HEADER: [u8; HEADER_SIZE] = *b"HTTP/1.1 ";
static CODE_LENGTH: usize = 3;
static CODE: [u8; CODE_LENGTH] = *b"200";
static OK_CODE: [u8; 3] = *b" OK";
static START_SEQUENCE: [u8; 4] = *b"\r\n\r\n";

#[derive(Encode)]
pub struct HttpResponse<'a> {
    http_header: &'a [u8],
    response_code: &'a [u8],
    code_comment: &'a [u8],
    start_sequence: &'a [u8],
    data: &'a [u8],
}

impl<'a> HttpResponse<'a> {
    pub fn new(in_data: &'a [u8]) -> Self {
        Self { 
            http_header: &HTTP_HEADER,
            response_code: &CODE,
            code_comment: &OK_CODE,
            start_sequence: &START_SEQUENCE,
            data: in_data,
        }
    }
    pub fn get_bytes(self : &Self,buffer : &mut [u8] ){
        let config = config::standard();
        bincode::encode_into_slice(self, buffer, config).unwrap();
        return ;
    }

}
