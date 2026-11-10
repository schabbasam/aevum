
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    // beautiful rgb color values from 0 to 255
    red:    u8,
    green:  u8,
    blue:   u8,
}
impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        // fortunately we don't need validation because u8 can't be out of range
        Self {red: r, green: g, blue: b}
    }

    pub fn red(&self) -> u8 { self.red }
    pub fn green(&self) -> u8 { self.green }
    pub fn blue(&self) -> u8 { self.blue }

    pub fn to_hex(&self) -> String {
        // formatting it as an uppercase hexadecimal string
        format!("#{:02X}{:02X}{:02X}", self.red, self.green, self.blue)
    }

    pub fn from_hex(hex: &str) -> Result<Self, String> {
        // strip the leading # if it exists
        let hex = hex.strip_prefix('#').unwrap_or(hex);
        
        // check if the hex is ascii
        if !hex.is_ascii() {
            return Err("hex color must contain only ASCII characters".to_string());
        }

        if hex.len() != 6 {
            return Err("invalid hex length, expected 6 characters".to_string());
        }

        // helper function
        fn parse_pair(pair: &str) -> Result<u8, String> {
            // from_str_radix converts pair from a base 16 (aka hex) to a u8 as a Result<>
            // if it is an Err, it gets converted to a String with map_err()
            u8::from_str_radix(pair, 16).map_err(|e| format!("failed to parse '{}': {}", pair, e))
        }

        Ok(Color {
            // string slicing: extracting 2 bytes at a time and checking if it is an Err or Ok
            red:    parse_pair(&hex[0..2])?,
            green:  parse_pair(&hex[2..4])?,
            blue:   parse_pair(&hex[4..6])?,
        })
    }

}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color() {
        let c = Color::new(200, 123, 9);

        assert_eq!(c.red(), 200);
        assert_eq!(c.green(), 123);
        assert_eq!(c.blue(), 9);

        // test maximum and minimum of u8 thingies
        Color::new(0, 0, 0);
        Color::new(255, 255, 255);

        // ---- hex conversion ----
        assert_eq!(c.to_hex(), "#C87B09");
        
        // check if the same color from hex is converted correctly
        // while we're at it lets check if lowercase works too
        assert_eq!(Color::from_hex("#C87b09").unwrap(), c);

        // ---- validation ----
        assert!(Color::from_hex("").is_err());
        assert!(Color::from_hex("#").is_err());
        assert!(Color::from_hex("12345").is_err());
        assert!(Color::from_hex("1234567").is_err());
        
        // whithout # should be fine too
        assert!(Color::from_hex("123456").is_ok());
        
        // should not work because G is invalid
        assert!(Color::from_hex("G349b8").is_err());
        
    }
}
