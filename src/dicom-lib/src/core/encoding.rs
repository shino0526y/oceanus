#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Encoding {
    ImplicitVrLittleEndian,
    ExplicitVrLittleEndian,
    ExplicitVrBigEndian,
}
