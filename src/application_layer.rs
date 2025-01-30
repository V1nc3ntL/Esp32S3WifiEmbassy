use embassy_net::tcp::TcpSocket;
use embedded_io_async::Write;
use httparse::Status::Complete;

#[derive(Debug, Clone)]
pub enum ApplicationError {
    MethodUnknown,
    RequestParsing,
    RequestUnkown,
    RequestHandling,
    RequestNotImplemented,
    SocketError,
}

pub async fn handle_request(
    socket: &mut TcpSocket<'_>,
    buf: &[u8],
) -> core::result::Result<(), ApplicationError> {
    const HEADER_BUFFER_SIZE: usize = 128;
    let mut header = [httparse::EMPTY_HEADER; HEADER_BUFFER_SIZE];
    let mut request = httparse::Request::new(&mut header);

    let body_idx = match request.parse(buf) {
        Ok(v) => match v {
            Complete(i) => i,
            httparse::Status::Partial => 0,
        },
        Err(_e) => return Err(ApplicationError::RequestParsing),
    };

    if request.method.is_some() {
        handle_method(socket, request.method.unwrap(), &buf[body_idx..]).await
    } else {
        write(
            socket,
            concat_bytes!(
                b"HTTP/1.1",
                b" 400 Bad Request\r\n\r\n<html><body><h1>BAD REQUEST!</h1></body></html>\r\n"
            ),
        )
        .await
    }
}

pub async fn handle_method(
    socket: &mut TcpSocket<'_>,
    method: &str,
    body_buf: &[u8],
) -> core::result::Result<(), ApplicationError> {
    match method {
        "GET" => handle_get(socket, body_buf).await,
        "POST" => handle_post(socket, body_buf).await,
        _ => Err(ApplicationError::RequestNotImplemented),
    }
}

pub async fn write(
    socket: &mut TcpSocket<'_>,
    buf: &[u8],
) -> core::result::Result<(), ApplicationError> {
    match socket.write_all(buf).await {
        Ok(()) => match socket.flush().await {
            Ok(()) => {
                socket.close();
                Ok(())
            }
            Err(_e) => Err(ApplicationError::SocketError),
        },
        Err(_e) => Err(ApplicationError::SocketError),
    }
}
static HEADER_SIZE: usize = 8;
static HTTP_HEADER: [u8; HEADER_SIZE] = *b"HTTP/1.1";

static OK_CODE: [u8; 7] = *b" 200 OK";
static START_SEQUENCE: [u8; 4] = *b"\r\n\r\n";

async fn handle_get(
    socket: &mut TcpSocket<'_>,
    _body_buf: &[u8],
) -> core::result::Result<(), ApplicationError> {
    let mut answer: [u8; 512] = [0; 512];
    let data: [u8; 41] = *b"<html><body><h1>GET!</h1></body></html>\r\n";

    let (header, after_header) = answer.split_at_mut(HEADER_SIZE);
    header.copy_from_slice(&HTTP_HEADER);
    let (answer_code, after_answer_code) = after_header.split_at_mut(7);
    answer_code.copy_from_slice(&OK_CODE);
    let (start_sequence, body) = after_answer_code.split_at_mut(4);
    start_sequence.copy_from_slice(&START_SEQUENCE);
    body[..41].copy_from_slice(&data);

    write(socket, &answer).await
}

async fn handle_post(
    socket: &mut TcpSocket<'_>,
    _body_buf: &[u8],
) -> core::result::Result<(), ApplicationError> {
    let mut answer: [u8; 512] = [0; 512];
    let data: [u8; 42] = *b"<html><body><h1>POST!</h1></body></html>\r\n";

    let (header, after_header) = answer.split_at_mut(HEADER_SIZE);
    header.copy_from_slice(&HTTP_HEADER);
    let (answer_code, after_answer_code) = after_header.split_at_mut(7);
    answer_code.copy_from_slice(&OK_CODE);
    let (start_sequence, body) = after_answer_code.split_at_mut(4);
    start_sequence.copy_from_slice(&START_SEQUENCE);
    body[..42].copy_from_slice(&data);

    write(socket, &answer).await
}
