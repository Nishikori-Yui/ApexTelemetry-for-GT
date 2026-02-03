// Metadata layer for car/track names and GT7Tracks geometry discovery.

mod detector;
mod geometry;
mod packet;
mod store;

pub use detector::TrackDetector;
pub use geometry::{TrackBounds, TrackSvg};
pub use packet::{parse_packet_meta, PacketMeta};
pub use store::{CarMeta, MetadataStore, TrackMeta};
