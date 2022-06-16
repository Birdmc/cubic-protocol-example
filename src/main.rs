use std::sync::Arc;
use cubic_chat::color::DefaultColor;
use cubic_chat::component::TextComponent;
use cubic_protocol::packet::PacketState;
use cubic_protocol::packet_default::{ClientHandshakePacket, ClientLoginPacket, ClientStatusPacket, StatusPong, StatusResponse, StatusResponseDescription, StatusResponseObject, StatusResponsePlayers, StatusResponseVersion};
use cubic_protocol_server::connection::Connection;
use cubic_protocol_server::handler::{ConnectionHandler, ContainerReadHandler, PacketHandler, ReadHandler};
use cubic_protocol_server::server::{ProtocolServerDeclare, run_server};
use log::{LevelFilter, Record};
use simple_logger::SimpleLogger;

struct MyConnectionHandler;

impl ConnectionHandler for MyConnectionHandler {
    fn handle_connection(&self, connection: Arc<Connection>) {
        println!("Connected: {}", connection.get_addr());
    }

    fn handle_disconnect(&self, connection: Arc<Connection>) {
        println!("Disconnected: {}", connection.get_addr());
    }
}

struct MyHandshakeReadHandler;

struct MyStatusReadHandler;

struct MyLoginReadHandler;

struct MyPlayReadHandler;

impl PacketHandler<ClientHandshakePacket> for MyHandshakeReadHandler {
    fn handle_packet(
        &self,
        connection: Arc<Connection>,
        state: &mut PacketState,
        packet: ClientHandshakePacket,
    ) {
        println!("{:?}", packet);
        match packet {
            ClientHandshakePacket::Handshaking(handshaking) => match handshaking.next_state {
                1 => *state = PacketState::Status,
                2 => *state = PacketState::Login,
                _ => {
                    tokio::spawn(async move {
                        connection.close().await
                    });
                },
            }
        };
    }
}

impl PacketHandler<ClientStatusPacket> for MyStatusReadHandler {
    fn handle_packet(
        &self,
        connection: Arc<Connection>,
        _: &mut PacketState,
        packet: ClientStatusPacket
    ) {
        println!("{:?}", packet);
        match packet {
            ClientStatusPacket::StatusRequest(_) => tokio::spawn(async move {
                connection.write_object(StatusResponse {
                    response: StatusResponseObject {
                        favicon: "".into(),
                        players: StatusResponsePlayers {
                            online: 5,
                            max: 5,
                            sample: Vec::new(),
                        },
                        description: StatusResponseDescription::Component({
                            let mut component = TextComponent::new(
                                "Cubic-Protocol Server!".into()
                            );
                            component.base.color = Some(DefaultColor::Red.into());
                            component.base.bold = Some(true);
                            component.into()
                        }),
                        version: StatusResponseVersion {
                            protocol: 757,
                            name: "1.18.2".into(),
                        }
                    }
                }).await
            }),
            ClientStatusPacket::StatusPing(ping) => tokio::spawn(async move {
                connection.write_object(StatusPong {
                    payload: ping.payload
                }).await
            })
        };
    }
}

impl PacketHandler<ClientLoginPacket> for MyLoginReadHandler {
    fn handle_packet(
        &self,
        connection: Arc<Connection>,
        _: &mut PacketState,
        packet: ClientLoginPacket
    ) {
        println!("{:?}", packet);
        match packet {
            ClientLoginPacket::LoginStart(start) => {
                println!("Player {} trying to login", start.name);
                tokio::spawn(async move { connection.close().await });
            }
            _ => {}
        };
    }
}

// it is doesn't matter
impl PacketHandler<ClientStatusPacket> for MyPlayReadHandler {
    fn handle_packet(&self, _: Arc<Connection>, _: &mut PacketState, _: ClientStatusPacket) {
        unreachable!()
    }
}

#[tokio::main]
async fn main() {
    SimpleLogger::new().with_level(LevelFilter::Debug).init();
    let task = run_server(ProtocolServerDeclare {
        host: "0.0.0.0:25565".into(),
        connection_handler: MyConnectionHandler,
        read_handler: ContainerReadHandler::new(
            MyHandshakeReadHandler,
            MyStatusReadHandler,
            MyLoginReadHandler,
            MyPlayReadHandler
        ),
    });
    task.task.await.unwrap();
}