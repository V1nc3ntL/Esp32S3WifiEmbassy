use embassy_net::tcp::TcpSocket;
use embedded_io::WriteReady;
use embedded_io_async::Write;
use esp_println::println;
use httparse::*;
pub async fn handle_request(
    socket: &mut TcpSocket<'_>,
    buf: &[u8],
    _n: usize,
) -> core::result::Result<(), embassy_net::tcp::Error> {
    // let body_idx = match parse_request(buf).await {
    //     Ok(count) => count,
    //     // This should return a proper error
    //     Err(e) => return write_bad_request(socket).await,
    // };
    write(
        socket,
        b"HTTP/1.1 200 OK\r\n\r\n<html><body><h1>HOLA!</h1></body></html>\r\n",
    )
    .await
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
        Err(e) => {
            println!("Error while writing");
            Err(e)
        }
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

pub async fn parse_request(buf: &[u8]) -> Result<usize> {
    let mut headers = [httparse::EMPTY_HEADER; 1];
    let mut req = httparse::Request::new(&mut headers);

    let result = req.parse(buf);

    match result {
        Ok(count) => Ok(count),
        Err(e) => Err(e),
    }
}
