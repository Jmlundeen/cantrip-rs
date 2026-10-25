use cantrip_api::*;
use cantrip_mechanics::{effects, empower, triggers::{THIS_ENTERS, YOU_GAIN_LIFE}};
use cantrip_oracle as oracle;

/// "When Way of the Mentor enters, empower Jace 5."
fn empower_jace(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    empower::jace(g, item.controller, item.object, 5)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, empower_jace, "When Way of the Mentor enters, empower Jace 5."),
        triggered(
            YOU_GAIN_LIFE,
            effects::put_loyalty_counter_on_each_planeswalker_you_control,
            "Whenever you gain life, put a loyalty counter on each planeswalker you control.",
        ),
    ],
    ..oracle::WAY_OF_THE_MENTOR
};

pub static EXTRA: &[&CardDef] = &[&empower::JACE];
