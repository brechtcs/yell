#![feature(plugin)]
#![plugin(json_macros)]

extern crate get_if_addrs;
extern crate mio;
extern crate rustc_serialize;

pub use mio::udp::UdpSocket;
use std::net::{IpAddr, SocketAddr};
use std::str;

pub fn open (address: &IpAddr, port: u16) -> UdpSocket {
  let target = SocketAddr::new(*address, port);
  let attempt = UdpSocket::bind(&target);

  match attempt {
    Err(why) => panic!("Could not bind to {}: {}", address, why),
    Ok(socket) => socket
  }
}

pub fn send (socket: &UdpSocket, message: &str) {
  socket.set_broadcast(true).unwrap();

  let bytes = message.to_string().into_bytes();
  let source = socket.local_addr().unwrap();
  let result = socket.send_to(&bytes, &source);
  drop(socket);

  match result {
    Err(e) => panic!("Send error: {}", e),
    Ok(amount) => println!("Sent {} bytes to {}", amount.unwrap(), socket.local_addr().unwrap())
  }
}

pub fn listen (socket: UdpSocket) -> Option<String> {
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

#[cfg(test)]
mod test {
  use super::*;
  use std::net::{IpAddr, Ipv4Addr};
  use get_if_addrs::{get_if_addrs, IfAddr};
  use rustc_serialize::json::Json;

  #[test]
  fn localhost() {
    let socket = open(&IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 1905);
    send(&socket, "Hello localhost!");

    loop {
      match listen(socket.try_clone().unwrap()) {
        None => continue,
        Some(received) => {
          assert_eq!(received, json!({"src":"127.0.0.1","msg":"Hello localhost!"}).to_string());
          break
        }
      }
    }
  }

  #[test]
  fn network() {
    for iface in get_if_addrs().unwrap() {
      match iface.addr {
        IfAddr::V4(addr) => {
          let target = match addr.broadcast {
            Some(broadcast) => broadcast,
            None => addr.ip
          };

          let socket = open(&IpAddr::V4(target), 2304);
          send(&socket, "Hello network!");

          loop {
            match listen(socket.try_clone().unwrap()) {
              None => continue,
              Some(received) => {
                let data = Json::from_str(&received).unwrap();
                let msg = data.as_object().unwrap().get("msg").unwrap();
                let src = data.as_object().unwrap().get("src").unwrap();

                assert_eq!(msg.to_string(), json!(("Hello network!")).to_string());
                assert_eq!(src.to_string(), json!((addr.ip.to_string())).to_string());
                break
              }
            }
          }
        },
        IfAddr::V6(_) => continue
      }
    }
  }
}
