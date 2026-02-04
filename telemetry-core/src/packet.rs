// Packet metadata helpers extracted from raw UDP payload.

pub struct PacketMeta {
    pub car_id: Option<i32>,
    pub position_xz: Option<(f32, f32)>,
}

pub fn parse_packet_meta(payload: &[u8]) -> PacketMeta {
    let car_id = read_i32(payload, 0x124);
    let pos_x = read_f32(payload, 0x04);
    let pos_z = read_f32(payload, 0x0C);
    let position_xz = match (pos_x, pos_z) {
        (Some(x), Some(z)) => Some((x, z)),
        _ => None,
    };

    PacketMeta {
        car_id,
        position_xz,
    }
}

fn read_f32(payload: &[u8], offset: usize) -> Option<f32> {
    let bytes = payload.get(offset..offset + 4)?;
    Some(f32::from_le_bytes(bytes.try_into().ok()?))
}

fn read_i32(payload: &[u8], offset: usize) -> Option<i32> {
    let bytes = payload.get(offset..offset + 4)?;
    Some(i32::from_le_bytes(bytes.try_into().ok()?))
}
