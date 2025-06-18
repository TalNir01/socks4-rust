//! Constants for the SOCKS protocol
pub mod socks4_status {
    /// Request granted
    pub const REQUEST_GRANTED: u8 = 0x5A;

    /// Request rejected or failed
    pub const REQUEST_REJECTED: u8 = 0x5B;

    /// Request failed because client is not running identd
    pub const IDENTD_NOT_RUNNING: u8 = 0x5C;

    /// Request failed because identd could not confirm the user ID
    pub const IDENTD_CONFIRM_FAILED: u8 = 0x5D;
}
