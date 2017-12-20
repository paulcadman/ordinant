extern crate websocket;
extern crate bus;

use std::thread;
use bus::Bus;
use std::sync::{Arc, Mutex};
use websocket::OwnedMessage;
use websocket::sync::Server;

fn main() {
	let server = Server::bind("127.0.0.1:2794").unwrap();
	let bus = Arc::new(Mutex::new(Bus::new(10)));

	for request in server.filter_map(Result::ok) {
		let mut rx = bus.lock().unwrap().add_rx();
		let bus = bus.clone();

		thread::spawn(move || {
			let mut client = request.accept().unwrap();

			let message = OwnedMessage::Text("Welcome".to_string());
			client.send_message(&message).unwrap();

			let (mut receiver, sender) = client.split().unwrap();
			let sender = Arc::new(Mutex::new(sender));

			let broadcast_sender = sender.clone();
			thread::spawn(move || {
				for message in rx.iter() {
					broadcast_sender.lock().unwrap().send_message(&message).unwrap();
				}
			});

			for message in receiver.incoming_messages() {
				let message = message.unwrap();

				match message {
					OwnedMessage::Close(_) => {
						let message = OwnedMessage::Close(None);
						{
							sender.lock().unwrap().send_message(&message).unwrap();
						}
						return;
					}
					OwnedMessage::Ping(ping) => {
						let message = OwnedMessage::Pong(ping);
						{
							sender.lock().unwrap().send_message(&message).unwrap();
						}
					}
					_ => {
						{
							bus.lock().unwrap().broadcast(message.clone());
						}
					}
				}
			}
		});
	}
}
