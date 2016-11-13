extern crate mio;

pub use mio::udp::UdpSocket;
use std::{str, thread};
use std::net::SocketAddr;

pub fn open (address: &SocketAddr) -> UdpSocket {
  let attempt = UdpSocket::bind(address);
  let socket;

  match attempt {
    Err(e) => panic!("Could not bind to {}: {}", address, e),
    Ok(s) => socket = s
  }

  socket
}

pub fn send (socket: &UdpSocket, message: &str, source: &SocketAddr) {
  socket.set_broadcast(true).unwrap();

  let bytes = message.to_string().into_bytes();
  let result = socket.send_to(&bytes, &source);
  drop(socket);

  match result {
    Err(e) => panic!("Send error: {}", e),
    Ok(amount) => println!("Sent {} bytes to {}", amount.unwrap(), socket.local_addr().unwrap())
  }
}

pub fn listen (socket: UdpSocket) -> Client {
  Client {
    socket: socket
  }
}

pub struct Client {
  socket: UdpSocket
}

impl Iterator for Client {
  type Item = String;

  fn next(&mut self) -> Option<String> {
    let message: String;

    loop {
      match self.socket.try_clone() {
        Err(why) => panic!("Socket error: {}", why),
        Ok(socket) => {
          match receive(socket) {
            None => continue,
            Some(msg) => {
              message = msg;
              break
            }
          }
        }
      }
    }
    Some(message)
  }
}

fn receive (socket: UdpSocket) -> Option<String> {
  let mut buffer: [u8; 2048] = [0; 2048];
  let result = socket.recv_from(&mut buffer);
  drop(socket);

  match result {
    Err(e) => panic!("Receive error: {}", e),
    Ok(opt) => match opt {
      None => None,
      Some((amount, source)) => {
        println!("Received {} bytes from {}", amount, source);
        Some(format(buffer, amount, source))
      }
    }
  }
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
  fn single_message() {
    let local = Ipv4Addr::new(127, 0, 0, 1);
    let address = SocketAddrV4::new(local, 1905);
    let socket = open(&SocketAddr::V4(address));

    let receiver = listen(socket.try_clone().unwrap());
    thread::sleep(time::Duration::from_millis(1500));

    send(&socket, "{\"dit\":\"dat\"}", &SocketAddr::V4(address));

    for received in receiver {
      assert_eq!(received, "{\"src\":\"127.0.0.1\",\"msg\":{\"dit\":\"dat\"}}");
      break
    }
  }
}
