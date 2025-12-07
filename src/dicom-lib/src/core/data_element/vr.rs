use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vr {
    Ae,
    As,
    At,
    Cs,
    Da,
    Ds,
    Dt,
    Fl,
    Fd,
    Is,
    Lo,
    Lt,
    Ob,
    Od,
    Of,
    Ol,
    Ov,
    Ow,
    Pn,
    Sh,
    Sl,
    Sq,
    Ss,
    St,
    Sv,
    Tm,
    Uc,
    Ui,
    Ul,
    Un,
    Ur,
    Us,
    Ut,
    Uv,
}

impl Vr {
    pub fn as_str(&self) -> &str {
        match self {
            Vr::Ae => "AE",
            Vr::As => "AS",
            Vr::At => "AT",
            Vr::Cs => "CS",
            Vr::Da => "DA",
            Vr::Ds => "DS",
            Vr::Dt => "DT",
            Vr::Fl => "FL",
            Vr::Fd => "FD",
            Vr::Is => "IS",
            Vr::Lo => "LO",
            Vr::Lt => "LT",
            Vr::Ob => "OB",
            Vr::Od => "OD",
            Vr::Of => "OF",
            Vr::Ol => "OL",
            Vr::Ov => "OV",
            Vr::Ow => "OW",
            Vr::Pn => "PN",
            Vr::Sh => "SH",
            Vr::Sl => "SL",
            Vr::Sq => "SQ",
            Vr::Ss => "SS",
            Vr::St => "ST",
            Vr::Sv => "SV",
            Vr::Tm => "TM",
            Vr::Uc => "UC",
            Vr::Ui => "UI",
            Vr::Ul => "UL",
            Vr::Un => "UN",
            Vr::Ur => "UR",
            Vr::Us => "US",
            Vr::Ut => "UT",
            Vr::Uv => "UV",
        }
    }
}

#[derive(Error, Debug)]
pub enum VrParseError {
    #[error("不明なVRです (バイト列=[0x{:02X}, 0x{:02X}], 文字列=\"{}\")", .0[0], .0[1], String::from_utf8_lossy(.0))]
    UnknownVr([u8; 2]),
}

impl TryFrom<[u8; 2]> for Vr {
    type Error = VrParseError;

    fn try_from(v: [u8; 2]) -> Result<Self, Self::Error> {
        match &v {
            b"AE" => Ok(Vr::Ae),
            b"AS" => Ok(Vr::As),
            b"AT" => Ok(Vr::At),
            b"CS" => Ok(Vr::Cs),
            b"DA" => Ok(Vr::Da),
            b"DS" => Ok(Vr::Ds),
            b"DT" => Ok(Vr::Dt),
            b"FL" => Ok(Vr::Fl),
            b"FD" => Ok(Vr::Fd),
            b"IS" => Ok(Vr::Is),
            b"LO" => Ok(Vr::Lo),
            b"LT" => Ok(Vr::Lt),
            b"OB" => Ok(Vr::Ob),
            b"OD" => Ok(Vr::Od),
            b"OF" => Ok(Vr::Of),
            b"OL" => Ok(Vr::Ol),
            b"OV" => Ok(Vr::Ov),
            b"OW" => Ok(Vr::Ow),
            b"PN" => Ok(Vr::Pn),
            b"SH" => Ok(Vr::Sh),
            b"SL" => Ok(Vr::Sl),
            b"SQ" => Ok(Vr::Sq),
            b"SS" => Ok(Vr::Ss),
            b"ST" => Ok(Vr::St),
            b"SV" => Ok(Vr::Sv),
            b"TM" => Ok(Vr::Tm),
            b"UC" => Ok(Vr::Uc),
            b"UI" => Ok(Vr::Ui),
            b"UL" => Ok(Vr::Ul),
            b"UN" => Ok(Vr::Un),
            b"UR" => Ok(Vr::Ur),
            b"US" => Ok(Vr::Us),
            b"UT" => Ok(Vr::Ut),
            b"UV" => Ok(Vr::Uv),
            _ => Err(VrParseError::UnknownVr(v)),
        }
    }
}

impl From<Vr> for [u8; 2] {
    fn from(v: Vr) -> Self {
        v.as_str()
            .as_bytes()
            .try_into()
            .expect("VR文字列は2バイト固定のはず")
    }
}
