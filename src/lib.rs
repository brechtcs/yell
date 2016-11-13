pub use std::net::UdpSocket;
use std::{str, thread};
use std::net::SocketAddr;

pub fn socket (address: SocketAddr) -> UdpSocket {
  let attempt = UdpSocket::bind(address);
  let socket;

  match attempt {
    Err(e) => panic!("Could not bind to {}: {}", address, e),
    Ok(s) => socket = s
  }

  socket
}

pub fn send (socket: &UdpSocket, message: Vec<u8>, source: SocketAddr) {
  socket.set_broadcast(true).unwrap();

  let result = socket.send_to(&message, source);
  drop(socket);

  match result {
    Err(e) => panic!("Send error: {}", e),
    Ok(amount) => println!("Sent {} bytes to {}", amount, socket.local_addr().unwrap())
  }
}

pub fn listen (socket: UdpSocket) -> thread::JoinHandle<String> {
  let handle = thread::spawn(move || {
    receive(socket)
  });

  handle
}

fn receive (socket: UdpSocket) -> String {
  let mut buffer: [u8; 2048] = [0; 2048];
  let result = socket.recv_from(&mut buffer);
  drop(socket);

  let data: String;
  match result {
    Ok((amount, source)) => {
      println!("Received {} bytes from {}", amount, source);
      data = format(buffer, amount, source);
    },
    Err(e) => panic!("Receive error: {}", e)
  }

  data
}

fn format (buffer: [u8; 2048], amount: usize, source: SocketAddr) -> String {
  let body = str::from_utf8(&buffer[0..amount]).unwrap_or("{}");
  format!("{{\"src\":\"{}\",\"msg\":{}}}", source.ip(), body)
}

#[cfg(test)]
mod test {
  use std::{thread, time};
  use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
  use super::*;

  #[test]
  fn run_socket() {
    let local = Ipv4Addr::new(127, 0, 0, 1);
    let address = SocketAddrV4::new(local, 1905);
    let socket = socket(SocketAddr::V4(address));

    let message: Vec<u8> = "{\"dit\":\"dat\"}".to_string().into_bytes();
    let receiver = listen(socket.try_clone().unwrap());
    thread::sleep(time::Duration::from_millis(1500));

    send(&socket, message, SocketAddr::V4(address));
    let received = receiver.join().unwrap();

    assert_eq!(received, "{\"src\":\"127.0.0.1\",\"msg\":{\"dit\":\"dat\"}}")
  }
}
