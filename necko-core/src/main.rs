use std::cmp::Ordering;
use std::collections::VecDeque;
use std::io::{Error, ErrorKind, Write};
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tokio::sync::{Mutex, RwLock};
use tokio::net::{TcpListener, TcpStream};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use bytes::{Buf, BytesMut};
use crossbeam::atomic::AtomicCell;
use necko_protocol::buffer::PacketByteBuffer;
use necko_protocol::decoder::Decoder;
use necko_protocol::encoder::Encoder;
use necko_protocol::packets::serverbound::intention;
use necko_protocol::packets::{ClientboundPacket, UnsignedPacket};
use necko_protocol::types::VarInt;
use necko_protocol::packets::{Packet, ServerboundPacket};
use necko_protocol::packets::clientbound::status::pong_response::PongResponseClientbound;
use necko_protocol::packets::clientbound::status::status_response::StatusResponseClientbound;
use necko_protocol::packets::serverbound::intention::NextState;
use necko_protocol::packets::serverbound::status::ping_request::PingRequestServerbound;
use necko_protocol::packets::serverbound::status::status_request::StatusRequestServerbound;

const PROTOCOL_VERSION: i32 = 769;
const MINECRAFT_VERSION: &str = "1.21.4";

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:25565").await?;
    
    loop {
        let (stream, addr) = listener.accept().await?;
        stream.set_nodelay(true)?;

        let client = Arc::new(Client::new(stream, addr));

        tokio::spawn(async move {
            while !client.closed.load(std::sync::atomic::Ordering::Relaxed) {
                let successful = client.poll().await;
                if successful {
                    client.process_packets().await;
                }
            }
        });
    }
}


pub struct Client {
    pub address: RwLock<SocketAddr>,
    pub packets_queue: Arc<Mutex<VecDeque<UnsignedPacket>>>,
    pub state: AtomicCell<NextState>,
    pub closed: AtomicBool,
    pub decoder: Arc<Mutex<Decoder>>,
    pub reader: Arc<Mutex<OwnedReadHalf>>,
    pub encoder: Arc<Mutex<Encoder>>,
    pub writer: Arc<Mutex<OwnedWriteHalf>>
}


impl Client {
    pub async fn handle_intention(&self, packet: intention::IntentionServerbound) {
        println!("Handling Intention packet");
        self.state.store(packet.next_state);

        println!("Got state: {:?}", self.state.load());
        let protocol = packet.protocol_version.0;
        println!("Protocol version: {}", protocol);
        if self.state.load() != NextState::Status {
            match protocol.cmp(&PROTOCOL_VERSION) {
                Ordering::Less => {
                    todo!("client outdated")
                }
                Ordering::Equal => println!("Client connected"),
                Ordering::Greater => {
                    todo!("server outdated")
                }
            }
        }
    }

    pub async fn handle_ping_request(&self, packet: PingRequestServerbound) {
        println!("Handling Ping Request packet");
        self.send_packet(&PongResponseClientbound::new(
            packet.timestamp
        )).await;
        self.close().await
    }

    pub async fn handle_status_request(&self) {
        println!("Handling Status Request packet");
        let status = "{\
            \"version\":{\
                \"name\":\"1.21.4\",\
                \"protocol\":769\
            },\
            \"players\":{\
                \"max\":-52,\
                \"online\":52\
            },\
            \"description\":{\
                \"text\":\"Hello, world!\",\"color\":\"red\"\
            }\
        }";

        self.send_packet(&StatusResponseClientbound::new(status)).await
    }
}

impl Client {
    pub fn new(tcp_stream: TcpStream, socket_addr: SocketAddr) -> Self {
        let (reader, writer) = tcp_stream.into_split();
        Client {
            address: RwLock::new(socket_addr),
            packets_queue: Arc::new(Mutex::new(VecDeque::new())),
            state: AtomicCell::new(NextState::None),
            closed: AtomicBool::new(false),
            decoder: Arc::new(Mutex::new(Decoder::new())),
            encoder: Arc::new(Mutex::new(Encoder::new())),
            reader: Arc::new(Mutex::new(reader)),
            writer: Arc::new(Mutex::new(writer)),
        }
    }

    pub async fn add_packet(&self, packet: UnsignedPacket) {
        let mut packets_queue = self.packets_queue.lock().await;
        packets_queue.push_back(packet);
    }

    pub async fn process_packets(&self) {
        let mut packet_queue = self.packets_queue.lock().await;
        while let Some(packet) = packet_queue.pop_front() {
            if let Err(error) = self.handle_packet(packet).await {
                todo!("error while reading packet")
            }
        }
    }

    pub async fn handle_packet(&self, packet: UnsignedPacket) -> Result<(), Error> {
        println!("handling {:?}", packet);
        match self.state.load() {
            NextState::None => {
                self.handle_intention_packet(packet).await
            }
            NextState::Status => {
                self.handle_status_packet(packet).await
            }
            NextState::Login => {unimplemented!()}
            NextState::Transfer => {unimplemented!()}
        }
    }

    pub async fn handle_intention_packet(&self, mut packet: UnsignedPacket) -> Result<(), Error> {
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

    pub async fn handle_status_packet(&self, mut packet: UnsignedPacket) -> Result<(), Error> {
        let buffer = &mut packet.data;
        match packet.id.0 {
            StatusRequestServerbound::PACKET_ID => self.handle_status_request().await,
            PingRequestServerbound::PACKET_ID => self
                .handle_ping_request(PingRequestServerbound::read(buffer)?).await,
            id => {
                unimplemented!("failed to handle status packet ({id})")
            }
        }
        Ok(())
    }

    pub async fn poll(&self) -> bool {
        loop {
            let mut decoder = self.decoder.lock().await;

            match decoder.decode() {
                Ok(Some(packet)) => {
                    println!("from polling (Some(Packet)): {:?}, {:?}, {:?}", self.state.load(), decoder, packet);
                    self.add_packet(packet).await;
                    return true
                } Ok(None) => (),
                Err(e) => {
                    println!("failed to decode packet for: {}", e);
                    self.close().await;
                    return false
                }
            }

            // println!("{:?}", decoder);

            decoder.reserve(4096);
            let mut buffer = decoder.take_capacity();

            match self.reader.lock().await.read_buf(&mut buffer).await {
                Ok(0) => {
                    println!("buffer: {:?}, decoder {:?}", buffer, decoder);
                    self.close().await;
                    return false
                }
                Err(e) => {
                    println!("error while reading packet: {}", e);
                    self.close().await;
                    return false
                }
                _ => ()
            }

            decoder.append_bytes(buffer);
            println!("from polling (end): {:?}, {:?}", self.state.load(), decoder);
        }
    }

    pub async fn send_packet<P: ClientboundPacket>(&self, packet: &P) {
        let mut encoder = self.encoder.lock().await;
        if let Err(e) = encoder.append(packet) {
            unimplemented!("failed to encode packet: {}", e)
        }
        
        let mut writer = self.writer.lock().await;
        if let Err(e) = writer.write_all(&encoder.take()).await {
            unimplemented!("failed to write packet: {}", e)
        }
    }

    pub async fn close(&self) {
        self.closed.store(true, std::sync::atomic::Ordering::Relaxed);
        println!("closing connection from {}", self.address.read().await.to_string())
    }

}







