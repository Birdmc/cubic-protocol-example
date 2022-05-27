use std::marker::PhantomData;
use std::str::FromStr;
use std::sync::Arc;
use cubic_chat::component::TextComponent;
use cubic_protocol::server::server::{Connection, ProtocolServer, ProtocolServerHandler};
use cubic_protocol::p1_18_2;
use cubic_protocol::p1_18_2::{NextState, Pong, Response, ServerPacket, SHPacket, SSPacket};
use cubic_protocol::protocol::ProtocolJson;
use cubic_protocol::status::{StatusPlayers, StatusResponse, StatusSample, StatusVersion};
use cubic_protocol::version::State;
use log::{info, LevelFilter};
use simple_logger::SimpleLogger;
use uuid::Uuid;

struct Handler {}

impl ProtocolServerHandler<p1_18_2::ServerPacket> for Handler {
    fn handle_connect(&self, connection: Arc<Connection>) {
        info!("Client connected: {}", connection.get_addr());
    }

    fn handle_disconnect(&self, connection: Arc<Connection>) {
        info!("Client disconnected: {}", connection.get_addr());
    }

    fn handle_event(&self, connection: Arc<Connection>, state: &mut State, packet: ServerPacket) {
        info!("Received event: {:?}", packet);
        match packet {
            ServerPacket::Handshake(packet) => match packet {
                SHPacket::Handshaking(packet) => match packet.next_state {
                    NextState::Status => *state = State::Status,
                    NextState::Login => {
                        tokio::spawn(async move { connection.close().await.unwrap() });
                    }
                }
            }
            ServerPacket::Status(packet) => match packet {
                SSPacket::Request(request) => {
                    tokio::spawn(async move {
                        connection.write_object(&Response {
                            response: ProtocolJson {
                                value: StatusResponse {
                                    version: StatusVersion {
                                        protocol: 758,
                                        name: "1.18.2".into(),
                                    },
                                    players: StatusPlayers {
                                        max: 100,
                                        online: 1,
                                        sample: vec![
                                            StatusSample {
                                                name: "jenya705".into(),
                                                id: Uuid::from_str("b31f5079-c09d-4979-b02e-2396e5fd9afb").unwrap(),
                                            }
                                        ],
                                    },
                                    description: TextComponent::new("hi!".into()).into(),
                                    favicon: "".to_string(),
                                }
                            }
                        }).await.unwrap()
                    });
                },
                SSPacket::Ping(packet) => {
                    tokio::spawn(async move {
                        connection.write_object(&Pong {
                            payload: packet.payload,
                        }).await.unwrap()
                    });
                }
            }
            ServerPacket::Login(_) => {}
            ServerPacket::Play(_) => {}
        }
    }
}

#[tokio::main]
async fn main() {
    SimpleLogger::new().with_level(LevelFilter::Info).init().unwrap();
    let server = ProtocolServer {
        handler: Handler {},
        host: "0.0.0.0:25565".to_string(),
        packet_node: PhantomData,
    };
    server.run().await.unwrap()
}