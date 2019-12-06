use std::convert::TryFrom;

pub struct WrongMeasure;

pub struct SingleCellChar {
    symbol: String,
}

impl TryFrom<String> for SingleCellChar {
    type Error = WrongMeasure;

    fn try_from(symbol: String) -> Result<SingleCellChar, WrongMeasure> {
        if symbol.len() == 1 {
            let ch = symbol.chars().next().unwrap();
            if ch == ' ' || ch.is_ascii_graphic() {
                Ok(SingleCellChar { symbol })
            } else {
                Err(WrongMeasure)
            }
        } else {
            Err(WrongMeasure)
        }
    }
}

impl TryFrom<&str> for SingleCellChar {
    type Error = WrongMeasure;

    fn try_from(symbol: &str) -> Result<SingleCellChar, WrongMeasure> {
        Self::try_from(symbol.to_owned())
    }
}

pub struct FreeformImage {
    ch: SingleCellChar,
}

impl FreeformImage {
    pub fn uniform(ch: SingleCellChar) -> Self {
        FreeformImage { ch }
    }
}
