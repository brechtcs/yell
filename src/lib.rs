#![feature(plugin)]
#![plugin(json_macros)]

extern crate get_if_addrs;
extern crate mio;
extern crate rustc_serialize;

mod hub;

pub use mio::udp::UdpSocket;
use std::net::{IpAddr, SocketAddr};
use hub::Hub;

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

pub fn listen (socket: UdpSocket) -> Hub {
  Hub {
    socket: socket
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use std::net::{IpAddr, Ipv4Addr};
  use std::{thread, time};
  use get_if_addrs::{get_if_addrs, IfAddr};
  use rustc_serialize::json::Json;

  #[test]
  fn localhost() {
    let socket = open(&IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 1905);
    let receiver = listen(socket.try_clone().unwrap());
    thread::sleep(time::Duration::from_millis(1500));

    send(&socket, "Hello localhost!");

    for received in receiver {
      assert_eq!(received, json!({"src":"127.0.0.1","msg":"Hello localhost!"}).to_string());
      break
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
          let receiver = listen(socket.try_clone().unwrap());
          thread::sleep(time::Duration::from_millis(1500));

          send(&socket, "Hello network!");

          for received in receiver {
            let data = Json::from_str(&received).unwrap();
            let msg = data.as_object().unwrap().get("msg").unwrap();
            let src = data.as_object().unwrap().get("src").unwrap();

            assert_eq!(msg.to_string(), json!(("Hello network!")).to_string());
            assert_eq!(src.to_string(), json!((addr.ip.to_string())).to_string());

            break
          }
        },
        IfAddr::V6(_) => continue
      }
    }
  }
}
