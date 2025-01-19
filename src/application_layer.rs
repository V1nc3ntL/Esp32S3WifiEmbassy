use embassy_net::tcp::TcpSocket;
use embedded_io_async::Write;
use httparse::Status::Complete;

pub const HEADER_BUFFER_SIZE: usize = 128;

pub async fn handle_request(
    socket: &mut TcpSocket<'_>,
    buf: &[u8],
) -> core::result::Result<(), embassy_net::tcp::Error> {
    let mut header = [httparse::EMPTY_HEADER; HEADER_BUFFER_SIZE];
    let mut request = httparse::Request::new(&mut header);

    let body_idx = match request.parse(buf) {
        Ok(v) => match v {
            Complete(i) => i,
            httparse::Status::Partial => 0,
        },
        Err(e) => 0,
    };

    match request.method.is_some() {
        true => handle_method(socket, request.method.unwrap(), &buf[body_idx..]).await,
        false => write_bad_request(socket).await,
    }
}
pub async fn handle_method(
    socket: &mut TcpSocket<'_>,
    method: &str,
    body_buf: &[u8],
) -> core::result::Result<(), embassy_net::tcp::Error> {
    match method {
        "GET" => {
            write(
                socket,
                b"HTTP/1.1 200 OK\r\n\r\n<html><body><h1>HOLA!</h1></body></html>\r\n",
            )
            .await
        }
        _ => Err(embassy_net::tcp::Error::ConnectionReset),
    }
}

pub async fn write(
    socket: &mut TcpSocket<'_>,
    buf: &[u8],
) -> core::result::Result<(), embassy_net::tcp::Error> {
    match socket.write_all(buf).await {
        Ok(()) => match socket.flush().await {
            Ok(()) => {
                socket.close();
                Ok(())
            }
            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    }
}

pub async fn write_bad_request(
    socket: &mut TcpSocket<'_>,
) -> core::result::Result<(), embassy_net::tcp::Error> {
    socket
        .write_all(
            b"HTTP/1.1 400 Bad Request\r\n\r\n<html><body><h1>BAD REQUEST!</h1></body></html>\r\n",
        )
        .await
}
