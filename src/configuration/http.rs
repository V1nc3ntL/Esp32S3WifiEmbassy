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
    start_sequence: &'static [u8],
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
    pub fn get_bytes(&self, buffer: &mut [u8]) {
        let config = config::standard();
        bincode::encode_into_slice(self, buffer, config).unwrap();
    }
}
pub struct HttpResponseBuilder<'a> {
    http_header: &'a [u8],
    response_code: &'a [u8],
    code_comment: &'a [u8],
    start_sequence: &'static [u8],
    data: &'a [u8],
}

impl<'a> HttpResponseBuilder<'a> {
    pub fn new(in_data: &'a [u8]) -> Self {
        HttpResponseBuilder {
            http_header: &HTTP_HEADER,
            response_code: &CODE,
            code_comment: &OK_CODE,
            start_sequence: &START_SEQUENCE,
            data: in_data,
        }
    }
    pub fn header(mut self, header: &'a [u8]) -> HttpResponseBuilder<'a> {
        self.http_header = header;
        self
    }
    pub fn code(mut self, code: &'a [u8]) -> HttpResponseBuilder<'a> {
        self.response_code = code;
        self
    }
    pub fn code_comment(mut self, code_comment: &'a [u8]) -> HttpResponseBuilder<'a> {
        {
            self.code_comment = code_comment;
            self
        }
    }
    pub fn data(mut self, data: &'a [u8]) -> HttpResponseBuilder<'a> {
        {
            self.data = data;
            self
        }
    }
    // If we can get away with not consuming the Builder here, that is an
    // advantage. It means we can use the FooBuilder as a template for constructing
    // many Foos.
    pub fn build(self) -> HttpResponse<'a> {
        // Create a Foo from the FooBuilder, applying all settings in FooBuilder
        // to Foo.
        HttpResponse {
            http_header: self.http_header,
            response_code: self.response_code,
            code_comment: self.code_comment,
            start_sequence: self.start_sequence,
            data: self.data,
        }
    }
}
