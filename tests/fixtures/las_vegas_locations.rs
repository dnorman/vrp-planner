//! Real Las Vegas / Henderson locations for realistic test fixtures.
//!
//! Coordinates sourced from OpenStreetMap via Overpass API.
//! These are real, routable locations that work with OSRM Nevada data.

/// A named location with coordinates.
#[derive(Debug, Clone)]
pub struct Location {
    pub name: &'static str,
    pub lat: f64,
    pub lng: f64,
}

impl Location {
    pub const fn new(name: &'static str, lat: f64, lng: f64) -> Self {
        Self { name, lat, lng }
    }

    pub fn coords(&self) -> (f64, f64) {
        (self.lat, self.lng)
    }
}

// ============================================================================
// Major Casinos / Hotels (good for depot/start locations)
// ============================================================================

pub const CASINOS: &[Location] = &[
    Location::new("Wynn Las Vegas", 36.1263781, -115.1658180),
    Location::new("Encore at Wynn", 36.1289345, -115.1653620),
    Location::new("MGM Grand", 36.1023654, -115.1688720),
    Location::new("Bellagio", 36.1126, -115.1767),
    Location::new("Caesars Palace", 36.1162, -115.1745),
    Location::new("Longhorn Casino", 36.1070664, -115.0591256),
];

// ============================================================================
// Las Vegas Strip Area Restaurants
// ============================================================================

pub const STRIP_RESTAURANTS: &[Location] = &[
    Location::new("Hard Rock Cafe", 36.1041592, -115.1722166),
    Location::new("SW Steakhouse", 36.1262145, -115.1669146),
    Location::new("Sinatra", 36.1300035, -115.1654850),
    Location::new("Public House", 36.1219193, -115.1689317),
    Location::new("Outback Steakhouse Strip", 36.1037287, -115.1724577),
    Location::new("The Crack Shack", 36.1050709, -115.1735287),
    Location::new("Brooklyn Bowl", 36.1175388, -115.1695094),
    Location::new("Flour & Barley", 36.1173688, -115.1702674),
    Location::new("Yard House", 36.1177147, -115.1691992),
    Location::new("Gordon Ramsay BurGR", 36.1107195, -115.1720818),
    Location::new("P.F. Chang's", 36.1103352, -115.1723830),
    Location::new("Earl of Sandwich Planet Hollywood", 36.1093912, -115.1720087),
    Location::new("Lobster Me Planet Hollywood", 36.1094857, -115.1708325),
    Location::new("Gordon Ramsay Steak", 36.1127744, -115.1712029),
    Location::new("Spago by Wolfgang Puck", 36.1139368, -115.1741462),
    Location::new("Le Cirque", 36.1135689, -115.1749763),
    Location::new("BLT Steakhouse", 36.1135528, -115.1690095),
    Location::new("Guy Fieri's Vegas Kitchen", 36.1184064, -115.1722088),
    Location::new("Hash House A Go Go", 36.1181377, -115.1710989),
    Location::new("Ruth's Chris Steak House", 36.1193113, -115.1722630),
    Location::new("Otto Pizzeria", 36.1231219, -115.1684514),
    Location::new("Canaletto Ristorante", 36.1230743, -115.1688618),
    Location::new("Buddy V's", 36.1231133, -115.1697093),
    Location::new("Carnevino Italian Steakhouse", 36.1249521, -115.1687357),
    Location::new("Grand Lux Cafe", 36.1216416, -115.1685024),
    Location::new("Delmonico Steakhouse", 36.1231561, -115.1686955),
    Location::new("CUT", 36.1233879, -115.1682073),
    Location::new("Rao's", 36.1163982, -115.1763053),
    Location::new("Beijing Noodle No. 9", 36.1158277, -115.1758038),
    Location::new("Bacchanal Buffet", 36.1159581, -115.1762929),
    Location::new("Mr Chow", 36.1161158, -115.1761223),
    Location::new("Best Friend", 36.1049396, -115.1739173),
    Location::new("Il Fornaio", 36.1024474, -115.1740110),
    Location::new("America", 36.1021028, -115.1750921),
    Location::new("Emeril's New Orleans Fish House", 36.1028578, -115.1688386),
    Location::new("Wolfgang Puck Bar & Grill", 36.1022046, -115.1696020),
    Location::new("L'Atelier De Joel Robuchon", 36.1026401, -115.1695890),
    Location::new("Charlie Palmer Steak", 36.0910624, -115.1743364),
    Location::new("Strip Steak", 36.0908722, -115.1776176),
];

// ============================================================================
// Henderson / East Las Vegas Area
// ============================================================================

pub const HENDERSON_LOCATIONS: &[Location] = &[
    Location::new("I Love Sushi Henderson", 35.9916660, -115.1028343),
    Location::new("Extended Stay America Henderson", 36.1283949, -115.0826989),
    Location::new("Islander's Grill", 36.0335058, -114.9856162),
    Location::new("Naga", 36.0137634, -114.9928676),
    Location::new("RibCage", 35.9949754, -115.0999810),
    Location::new("Buffalo Wild Wings Henderson", 36.0090449, -114.9917034),
    Location::new("Green Valley Ranch Area", 36.0308, -115.0825),
    Location::new("Sunset Station Area", 36.0614, -115.0631),
];

// ============================================================================
// North Las Vegas / Outlying Areas
// ============================================================================

pub const NORTH_VEGAS_LOCATIONS: &[Location] = &[
    Location::new("Rivas Mexican Grill North", 36.1450055, -115.0482587),
    Location::new("Roberto's Taco Shop", 36.1452953, -115.0478347),
    Location::new("Monarca Mexican Restaurant", 36.1440711, -115.0634197),
    Location::new("Pizza Hut North", 36.1443292, -115.0624966),
    Location::new("La Costa del Sol", 36.1470458, -115.0644345),
    Location::new("Beers and Bets", 36.1428945, -115.1573836),
];

// ============================================================================
// South Strip / Airport Area
// ============================================================================

pub const SOUTH_STRIP_LOCATIONS: &[Location] = &[
    Location::new("Buffalo Wild Wings South", 36.0543044, -115.1714860),
    Location::new("Dickey's Barbecue Pit", 36.0544257, -115.1715391),
    Location::new("Bootlegger Bistro", 36.0492047, -115.1715744),
    Location::new("Denny's South", 36.0591086, -115.1717250),
    Location::new("Tahiti Joe's Restaurant", 36.0592855, -115.1716402),
    Location::new("Matryoshka", 36.0492150, -115.1710486),
    Location::new("kabuki Japanese", 36.0675472, -115.1779391),
    Location::new("Mikos Izakaya", 36.0429503, -115.1527627),
    Location::new("Budget Suites South", 36.0366259, -115.1713361),
    Location::new("TENDER Steak & Seafood", 36.0949700, -115.1761289),
    Location::new("Pyramid Cafe", 36.0956586, -115.1761902),
    Location::new("Diablo's Cantina", 36.0955372, -115.1751818),
    Location::new("Burger Bar", 36.0943773, -115.1760142),
];

// ============================================================================
// Central / Mid-Strip
// ============================================================================

pub const MID_STRIP_LOCATIONS: &[Location] = &[
    Location::new("Marakesh", 36.1177772, -115.1546882),
    Location::new("Musashi", 36.1177743, -115.1545417),
    Location::new("Satay", 36.1182162, -115.1542982),
    Location::new("Coco's Bakery", 36.1004202, -115.1652380),
    Location::new("Gallagher's", 36.1025514, -115.1742518),
    Location::new("Denny's Mid Strip", 36.1209774, -115.1717620),
    Location::new("Tilted Kilt", 36.1174596, -115.1705837),
    Location::new("Slice of Vegas Pizza", 36.0944330, -115.1759954),
    Location::new("La La Noodle", 36.1041828, -115.1740723),
    Location::new("PBR Grill", 36.1090017, -115.1724402),
    Location::new("Outback Steakhouse Mid", 36.1207197, -115.1718418),
    Location::new("mon ami Gabi", 36.1128554, -115.1724137),
    Location::new("Gold mine Bar & Grill", 36.1181288, -115.1629389),
    Location::new("Bonanno's NY Pizzeria", 36.1165192, -115.1719357),
    Location::new("La Subs & Salads", 36.1162678, -115.1719679),
    Location::new("Center Cut Steakhouse", 36.1162981, -115.1717185),
    Location::new("Paradise Garden Buffet", 36.1170543, -115.1710748),
    Location::new("Oyster Bar", 36.1194951, -115.1715059),
    Location::new("Fulton Street Food Hall", 36.1193256, -115.1710575),
    Location::new("Toby Keith's Bar & Grill", 36.1190579, -115.1703190),
    Location::new("Social Life Pizza", 36.1210052, -115.1684273),
    Location::new("Lobster Me Venetian", 36.1217851, -115.1687545),
    Location::new("Le Macaron", 36.1217180, -115.1689230),
    Location::new("Morels French Steakhouse", 36.1249500, -115.1690442),
    Location::new("Grimaldi's Pizzeria", 36.1248850, -115.1683540),
    Location::new("Zeffirino", 36.1219228, -115.1695013),
    Location::new("Trattoria Reggiano", 36.1218938, -115.1684071),
    Location::new("Canonita", 36.1218701, -115.1687311),
    Location::new("Earl of Sandwich Miracle Mile", 36.1178304, -115.1756160),
    Location::new("Payard Patisserie", 36.1166582, -115.1759111),
];

// ============================================================================
// East Side Locations
// ============================================================================

pub const EAST_SIDE_LOCATIONS: &[Location] = &[
    Location::new("Pei Wei Town Square", 36.0810469, -115.1472694),
    Location::new("Jose Cuervo Tequileria", 36.0806515, -115.1465380),
    Location::new("Sammy's", 36.0826447, -115.1483055),
    Location::new("Villa", 36.0825032, -115.1481717),
    Location::new("Pei Wei East", 36.0861327, -115.1387345),
    Location::new("The Local", 36.0861274, -115.1388524),
    Location::new("Hello Tokyo", 36.1161627, -115.0902096),
    Location::new("Golden China", 36.1171166, -115.0904647),
    Location::new("Original Lindo Michoacan", 36.1294005, -115.1135106),
    Location::new("Rivas Mexican Grill East", 36.1295175, -115.1087980),
    Location::new("Pizza Hut East", 36.1305215, -115.1093500),
    Location::new("Tomo Sushi", 36.0992464, -115.1142123),
    Location::new("Pizza Hut Boulder", 36.1287535, -115.0931625),
    Location::new("Wo Fat Chinese", 36.1298523, -115.0936239),
    Location::new("Sushi Twister", 36.1007300, -115.0526259),
    Location::new("Thai Food To Go", 36.1302738, -115.1037355),
    Location::new("Chuck Wagon Restaurant", 36.1072491, -115.0593482),
    Location::new("Denny's Boulder", 36.1061288, -115.0578247),
    Location::new("Viva El Salvador", 36.1013492, -115.0646473),
    Location::new("Roma Pizza", 36.1012461, -115.0753039),
];

// ============================================================================
// All Locations Combined
// ============================================================================

/// Returns all locations as a single slice.
pub fn all_locations() -> Vec<Location> {
    let mut all = Vec::with_capacity(150);
    all.extend_from_slice(CASINOS);
    all.extend_from_slice(STRIP_RESTAURANTS);
    all.extend_from_slice(HENDERSON_LOCATIONS);
    all.extend_from_slice(NORTH_VEGAS_LOCATIONS);
    all.extend_from_slice(SOUTH_STRIP_LOCATIONS);
    all.extend_from_slice(MID_STRIP_LOCATIONS);
    all.extend_from_slice(EAST_SIDE_LOCATIONS);
    all
}

/// Returns a subset of locations for smaller tests.
pub fn sample_locations(count: usize) -> Vec<Location> {
    all_locations().into_iter().take(count).collect()
}

/// Returns locations spread across the metro area (good for multi-route tests).
pub fn geographically_diverse_locations() -> Vec<Location> {
    vec![
        // North
        Location::new("Rivas Mexican Grill North", 36.1450055, -115.0482587),
        Location::new("Beers and Bets", 36.1428945, -115.1573836),
        // Central Strip
        Location::new("Wynn Las Vegas", 36.1263781, -115.1658180),
        Location::new("Bellagio", 36.1126, -115.1767),
        Location::new("MGM Grand", 36.1023654, -115.1688720),
        // South
        Location::new("Bootlegger Bistro", 36.0492047, -115.1715744),
        Location::new("Budget Suites South", 36.0366259, -115.1713361),
        // East / Henderson
        Location::new("Green Valley Ranch Area", 36.0308, -115.0825),
        Location::new("Sunset Station Area", 36.0614, -115.0631),
        Location::new("Longhorn Casino", 36.1070664, -115.0591256),
        Location::new("I Love Sushi Henderson", 35.9916660, -115.1028343),
        Location::new("Islander's Grill", 36.0335058, -114.9856162),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_locations_count() {
        let all = all_locations();
        assert!(all.len() >= 100, "should have at least 100 locations, got {}", all.len());
    }

    #[test]
    fn test_coordinates_in_vegas_area() {
        for loc in all_locations() {
            assert!(loc.lat > 35.9 && loc.lat < 36.3, "{} lat out of range: {}", loc.name, loc.lat);
            assert!(loc.lng > -115.4 && loc.lng < -114.8, "{} lng out of range: {}", loc.name, loc.lng);
        }
    }
}
