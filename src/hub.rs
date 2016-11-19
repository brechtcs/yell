use std::net::SocketAddr;
use std::{str, thread, time};
use mio::udp::UdpSocket;

pub struct Hub {
  pub socket: UdpSocket
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
  json!({
    "src": (source.ip().to_string()),
    "msg": (body)
  }).to_string()
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
            Some(msg) => {
              message = msg;
              break
            },
            None => {
              thread::sleep(time::Duration::from_millis(1000));
              continue
            }
          }
        }
      }
    }
    Some(message)
  }
}
