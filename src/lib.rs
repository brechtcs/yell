extern crate mio;

pub use mio::udp::UdpSocket;
use std::str::{FromStr, from_utf8};
use std::net::SocketAddr;

pub fn open (address: &str) -> UdpSocket {
  let target = SocketAddr::from_str(address).unwrap();
  let attempt = UdpSocket::bind(&target);

  match attempt {
    Err(why) => panic!("Could not bind to {}: {}", address, why),
    Ok(socket) => socket
  }
}

pub fn send (socket: &UdpSocket, message: &str, source: &str) {
  socket.set_broadcast(true).unwrap();

  let bytes = message.to_string().into_bytes();
  let source = SocketAddr::from_str(source).unwrap();
  let result = socket.send_to(&bytes, &source);
  drop(socket);

  match result {
    Err(e) => panic!("Send error: {}", e),
    Ok(amount) => println!("Sent {} bytes to {}", amount.unwrap(), socket.local_addr().unwrap())
  }
}

pub fn listen (socket: UdpSocket) -> Hub {
  Hub {
    socket: socket
  }
}

pub struct Hub {
  socket: UdpSocket
}

impl Iterator for Hub {
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
  let body = from_utf8(&buffer[0..amount]).unwrap_or("{}");
  format!("{{\"src\":\"{}\",\"msg\":{}}}", source.ip(), body)
}

#[cfg(test)]
mod test {
  use std::{thread, time};
  use super::*;

  #[test]
  fn short_message() {
    let socket = open("127.0.0.1:1905");
    let receiver = listen(socket.try_clone().unwrap());
    thread::sleep(time::Duration::from_millis(1500));

    send(&socket, "\"Hello localhost!\"", "127.0.0.1:1905");

    for received in receiver {
      assert_eq!(received, "{\"src\":\"127.0.0.1\",\"msg\":\"Hello localhost!\"}");
      break
    }
  }

  #[test]
  fn local_network() {
    let address = "192.168.0.255:1905";
    let socket = open(address);
    let receiver = listen(socket.try_clone().unwrap());
    thread::sleep(time::Duration::from_millis(1500));

    send(&socket, "\"Hello network!\"", address);

    for received in receiver {
      assert_eq!(received, "{\"src\":\"127.0.0.1\",\"msg\":\"Hello network!\"}");
      break
    }
  }
}
