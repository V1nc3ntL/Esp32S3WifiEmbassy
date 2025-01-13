use embassy_net::tcp::TcpSocket;

const NUMBER_OF_CLIENTS: usize = match usize::from_str_radix(env!("NUMBER_OF_CLIENTS"), 10) {
    Ok(v) => v,
    Err(_e) => panic!("Could not retrieve the maximum expected number of clients  "),
};

pub const RX_BUFFER_SIZE: usize = 1024;
pub const TX_BUFFER_SIZE: usize = 1024;
type RxBufferType = [u8; RX_BUFFER_SIZE];
type TxBufferType = [u8; TX_BUFFER_SIZE];
pub static HTTP_SOCKETS_CELL: [static_cell::StaticCell<TcpSocket<'static>>; NUMBER_OF_CLIENTS] =
    [const { static_cell::StaticCell::new() }; NUMBER_OF_CLIENTS];
pub static RX_BUFFERS_CELL: [static_cell::StaticCell<RxBufferType>; NUMBER_OF_CLIENTS] =
    [const { static_cell::StaticCell::new() }; NUMBER_OF_CLIENTS];
pub static TX_BUFFERS_CELL: [static_cell::StaticCell<TxBufferType>; NUMBER_OF_CLIENTS] =
    [const { static_cell::StaticCell::new() }; NUMBER_OF_CLIENTS];
