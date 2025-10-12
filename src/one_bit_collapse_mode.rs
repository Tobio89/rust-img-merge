use clap::{builder::PossibleValue, ValueEnum};

use crate::app;

#[derive(Clone)]
pub enum OneBitDestinationChannel {
    Red,
    Green,
    Blue,
}

impl ValueEnum for OneBitDestinationChannel {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            OneBitDestinationChannel::Red,
            OneBitDestinationChannel::Green,
            OneBitDestinationChannel::Blue,
        ]
    }
    fn to_possible_value(&self) -> Option<PossibleValue> {
        match self {
            OneBitDestinationChannel::Red => Some(PossibleValue::new("red")),
            OneBitDestinationChannel::Green => Some(PossibleValue::new("green")),
            OneBitDestinationChannel::Blue => Some(PossibleValue::new("blue")),
        }
    }
}



pub fn do_one_bit_collapse_mode(cli: app::OneBitCollapseModeArgs) {
    println!("Doing One Bit Collapse mode...");

    println!("Pass.")


}
