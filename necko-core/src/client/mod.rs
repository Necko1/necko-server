use std::collections::VecDeque;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tokio::sync::{Mutex, RwLock};
use tokio::net::TcpStream;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crossbeam::atomic::AtomicCell;
use necko_protocol::decoder::Decoder;
use necko_protocol::encoder::Encoder;
use necko_protocol::packets::{ClientboundPacket, UnsignedPacket};
use necko_protocol::packets::serverbound::intention::NextState;
use crate::server::Server;

pub mod handlers;

pub struct Client {
    pub address: RwLock<SocketAddr>,
    pub packets_queue: Arc<Mutex<VecDeque<UnsignedPacket>>>,
    pub state: AtomicCell<NextState>,
    pub closed: AtomicBool,
    pub decoder: Arc<Mutex<Decoder>>,
    pub reader: Arc<Mutex<OwnedReadHalf>>,
    pub encoder: Arc<Mutex<Encoder>>,
    pub writer: Arc<Mutex<OwnedWriteHalf>>,
    
    pub done: AtomicBool,
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
            
            done: AtomicBool::new(false),
        }
    }

    pub async fn add_packet(&self, packet: UnsignedPacket) {
        let mut packets_queue = self.packets_queue.lock().await;
        packets_queue.push_back(packet);
    }

    pub async fn process_packets(&self, server: &Server) {
        let mut packet_queue = self.packets_queue.lock().await;
        while let Some(packet) = packet_queue.pop_front() {
            if let Err(_error) = self.handle_packet(packet, server).await {
                todo!("error while reading packet")
            }
        }
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