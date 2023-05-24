/// See <https://wiki.vg/Classic_Protocol#Client_.E2.86.92_Server_packets>
pub mod client;
/// See <https://wiki.vg/Classic_Protocol#Server_.E2.86.92_Client_packets>
pub mod server;
use classicl_serde::FixedSize;

pub trait Packet: FixedSize {
    const ID: u8;
}
