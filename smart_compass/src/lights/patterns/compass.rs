use super::{ANGLES, PHYSICAL_TO_FIBONACCI, RGB8};
use crate::arduino::*;
use crate::lights::focalintent::fade_to_black_by;
use crate::network::{NetworkData, PeerLocation};
use crate::NUM_LEDS;
use derive_more::Constructor;
use heapless::consts::*;
use heapless::FnvIndexMap;
use micromath::F32Ext;
use smart_leds::hsv::{hsv2rgb, Hsv};

#[derive(Constructor)]
pub struct Compass {
    pub background_fade: u8,
    pub max_distance: f32,
    /// if two peers are next to eachother, we cycle between their colors
    pub ms_per_color: u32,
}

impl Compass {
    pub fn buffer(
        &mut self,
        now: u32,
        leds: &mut [RGB8],
        network_data: &NetworkData,
    ) -> Option<()> {
        let my_peer_id = &network_data.my_peer_id;

        fade_to_black_by(leds, self.background_fade);

        if let Some((my_location, _)) = network_data.peer_locations[*my_peer_id].as_ref() {
            // store locations in a hashmap of vecs because multiple items might be on the same led
            // TODO: use MAX_PEERS for the size of this map
            let mut locations = FnvIndexMap::<_, _, U16>::new();

            locations
                .insert(0usize, alloc::vec![my_location])
                .ok()
                .unwrap();

            for peer_location in network_data.peer_locations.iter() {
                if let Some((peer_location, _)) = peer_location {
                    if peer_location.peer_id == *my_peer_id {
                        // we already drew ourselves at the center
                        continue;
                    }

                    // TODO: check last_updated_at to make sure it isn't too old
                    // if peer_location.last_updated_at;

                    let bearing = get_bearing(my_location, peer_location);

                    let distance = get_haversine_distance(my_location, peer_location);

                    let i = bearing_and_distance_to_id(bearing, distance, self.max_distance);

                    if let Some(location_vec) = locations.get_mut(&i) {
                        location_vec.push(peer_location);
                    } else {
                        locations
                            .insert(i, alloc::vec![peer_location])
                            .ok()
                            .unwrap();
                    }
                }
            }

            for (led_id, peer_ids) in locations.iter() {
                let drawn_peer_id = if peer_ids.len() == 1 {
                    peer_ids[0]
                } else {
                    // cycle between multiple
                    let color_id = (now / self.ms_per_color) as usize % peer_ids.len();

                    peer_ids[color_id]
                };

                // TODO: save the color in the peer_location instead of doing this conversion every time
                let color = hsv2rgb(Hsv {
                    hue: drawn_peer_id.hue,
                    sat: drawn_peer_id.sat,
                    val: 255,
                });

                // TODO: nblend
                leds[*led_id] = color;
            }
        }

        Some(())
    }
}

fn get_bearing(my_location: &PeerLocation, other_location: &PeerLocation) -> f32 {
    let d_lon = other_location.lon - my_location.lon;

    // y = math.sin(dLon) * math.cos(lat2)
    let y = d_lon.sin() * other_location.lat;
    // x = math.cos(lat1) * math.sin(lat2) - math.sin(lat1) * math.cos(lat2) * math.cos(dLon)
    let x = my_location.lat.cos() * other_location.lat.sin()
        - my_location.lat.sin() * other_location.lat.cos() * d_lon.cos();

    // brng = math.atan2(y, x)
    let bearing = y.atan2(x);

    // brng = math.degrees(brng)
    let bearing = bearing.to_degrees();

    // brng = (brng + 360) % 360
    let bearing = (bearing + 360.0) % 360.0;

    // brng = 360 - brng # count degrees clockwise - remove to make counter-clockwise
    let bearing = 360.0 - bearing;

    bearing
}

fn get_haversine_distance(my_location: &PeerLocation, other_location: &PeerLocation) -> f32 {
    // kilometer radius of Earth
    const R: f32 = 6371.0;

    let d_lat: f32 = (other_location.lat - my_location.lat).to_radians();
    let d_lon: f32 = (other_location.lon - my_location.lon).to_radians();
    let lat1: f32 = (my_location.lat).to_radians();
    let lat2: f32 = (other_location.lat).to_radians();

    let a: f32 = ((d_lat / 2.0).sin()) * ((d_lat / 2.0).sin())
        + ((d_lon / 2.0).sin()) * ((d_lon / 2.0).sin()) * (lat1.cos()) * (lat2.cos());
    let c: f32 = 2.0 * ((a.sqrt()).atan2((1.0 - a).sqrt()));

    return R * c;
}

pub fn bearing_and_distance_to_id(mut bearing: f32, mut distance: f32, max_distance: f32) -> usize {
    let mut best_gap = u16::MAX;
    let mut best_i = 0;

    if distance.is_sign_negative() {
        // flip the bearing
        bearing = (bearing + 180.0) % 360.0;
        distance = distance.abs();
    }

    // convert bearing from 0-360 to 0-255
    // TODO: off by one error here?
    let bearing = map(bearing, 0.0, 360.0, 0.0, 255.0) as i8;

    // convert distance from 0-max_distance to 0-255
    let distance = constrain(distance, 0.0, max_distance);
    let distance = map(distance, 0.0, max_distance, 0.0, 255.0) as i8;

    for i in 0..NUM_LEDS {
        let i_bearing = ANGLES[i] as i8;
        let i_distance = PHYSICAL_TO_FIBONACCI[i] as i8;

        let bearing_gap = (bearing - i_bearing).abs() as u16;
        let distance_gap = (distance - i_distance).abs() as u16;

        let gap = bearing_gap + distance_gap;

        if gap < best_gap {
            best_i = i;
            best_gap = gap;
        }
    }

    best_i
}
