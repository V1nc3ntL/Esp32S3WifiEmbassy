use embassy_net::tcp::Error;
use embassy_net::tcp::TcpSocket;
use embedded_io_async::Write;

/// The buf should contain the whole
pub async fn response_to_request(
    socket: &mut TcpSocket<'_>,
    _buf: &[u8],
    _n: usize,
) -> Result<(), Error> {
    match socket
        .write_all(b"HTTP/1.1 200 OK\r\n\r\n<html><body><h1>HOLA!</h1></body></html>\r\n")
        .await
    {
        Ok(()) => match socket.flush().await {
            Ok(()) => Ok(()),
            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    }
}
