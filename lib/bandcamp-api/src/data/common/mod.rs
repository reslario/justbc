pub mod parse;

#[derive(Debug, Clone)]
pub struct Date {
    pub day: u8,
    pub month: String,
    pub year: u16
}
