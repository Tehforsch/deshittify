use anyhow::anyhow;
use std::str::FromStr;

#[derive(Debug)]
pub enum Period {
    Day,
    Week,
    Month,
}

impl FromStr for Period {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "d" => Ok(Self::Day),
            "day" => Ok(Self::Day),
            "w" => Ok(Self::Week),
            "week" => Ok(Self::Week),
            "m" => Ok(Self::Month),
            "month" => Ok(Self::Month),
            _ => Err(anyhow!("Wrong period specifier.")),
        }
    }
}

impl Period {
    pub fn to_string(&self) -> String {
        match self {
            Period::Day => "d",
            Period::Week => "w",
            Period::Month => "m",
        }
        .to_owned()
    }
}
