use std::cmp::Ordering;
use std::io::Error;
use necko_protocol::packets::serverbound::intention;
use necko_protocol::packets::clientbound::status::pong_response::PongResponseClientbound;
use necko_protocol::packets::clientbound::status::status_response::StatusResponseClientbound;
use necko_protocol::packets::serverbound::intention::NextState;
use necko_protocol::packets::serverbound::login::hello::HelloServerbound;
use necko_protocol::packets::serverbound::status::ping_request::PingRequestServerbound;
use necko_protocol::packets::serverbound::status::status_request::StatusRequestServerbound;
use necko_protocol::packets::{Packet, ServerboundPacket, UnsignedPacket};
use crate::client::Client;
use crate::server::Server;
use crate::server::status::PROTOCOL_VERSION;

impl Client {
    pub async fn handle_packet(&self, packet: UnsignedPacket, server: &Server) -> Result<(), Error> {
        println!("handling {:?}", packet);
        match self.state.load() {
            NextState::None => {
                self.handle_intention_packet(packet).await
            }
            NextState::Status => {
                self.handle_status_packet(packet, server).await
            }
            NextState::Login => {
                self.handle_login_packet(packet, server).await
            }
            NextState::Transfer => {unimplemented!("got transfer packet ({})", packet.id.0)}
        }
    }
    
    // HANDSHAKE
    
    async fn handle_intention_packet(&self, mut packet: UnsignedPacket) -> Result<(), Error> {
        let buffer = &mut packet.data;
        match packet.id.0 {
            intention::IntentionServerbound::PACKET_ID => {
                self.handle_intention(intention::IntentionServerbound::read(buffer)?).await
            }
            id => {
                unimplemented!("failed to handle intention packet ({id})")
            }
        }
        println!("from intention: {:?}, {:?}", self.state.load(), self.decoder.lock().await);
        Ok(())
    }
    
    async fn handle_intention(&self, packet: intention::IntentionServerbound) {
        log::debug!("Handling Intention packet");
        self.state.store(packet.next_state);

        log::debug!("Got state: {:?}", self.state.load());
        let protocol = packet.protocol_version.0;
        log::debug!("Protocol version: {}", protocol);
        if self.state.load() != NextState::Status {
            match protocol.cmp(&PROTOCOL_VERSION) {
                Ordering::Less => {
                    todo!("client outdated")
                }
                Ordering::Equal => log::debug!("OK."),
                Ordering::Greater => {
                    todo!("server outdated")
                }
            }
        }
    }
    
    // STATUS
    
    async fn handle_status_packet(&self, mut packet: UnsignedPacket, server: &Server) -> Result<(), Error> {
        let buffer = &mut packet.data;
        match packet.id.0 {
            StatusRequestServerbound::PACKET_ID => self.handle_status_request(server).await,
            PingRequestServerbound::PACKET_ID => self
                .handle_ping_request(PingRequestServerbound::read(buffer)?).await,
            id => {
                unimplemented!("failed to handle status packet ({id})")
            }
        }
        Ok(())
    }
    
    async fn handle_ping_request(&self, packet: PingRequestServerbound) {
        log::debug!("Handling Ping Request packet");
        self.send_packet(&PongResponseClientbound::new(
            packet.timestamp)).await;
        self.close().await
    }

    async fn handle_status_request(&self, server: &Server) {
        log::debug!("Handling Status Request packet");
        self.send_packet(&StatusResponseClientbound::new(
            server.cached_status.read().await.json.as_str())).await
    }

    // LOGIN

    async fn handle_login_packet(&self, mut packet: UnsignedPacket, server: &Server) -> Result<(), Error> {
        let buffer = &mut packet.data;
        match packet.id.0 {
            HelloServerbound::PACKET_ID => self
                .handle_hello(HelloServerbound::read(buffer)?, server).await,
            id => {
                unimplemented!("failed to handle status packet ({id})")
            }
        }
        Ok(())
    }
    
    async fn handle_hello(&self, _packet: HelloServerbound, _server: &Server) {
        log::debug!("Handling Hello (login start) packet");
        
        unimplemented!("actually")
    }
}