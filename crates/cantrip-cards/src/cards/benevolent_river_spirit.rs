use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    // "As an additional cost to cast this spell, waterbend {5}." The cost is paid as the spell is cast (CR 601.2f, 701.67a):
    // five generic mana, each of which an untapped artifact or creature of its caster's may pay by tapping (CR 701.67b).
    additional_cost: Some(Cost::waterbend(5, 0)),
    abilities: &[
        triggers::ward::<2>("Ward {2} (Whenever this creature becomes the target of a spell or ability an opponent controls, counter it unless that player pays {2}.)"),
        triggered(triggers::THIS_ENTERS, effects::scry_two, "When this creature enters, scry 2."),
    ],
    ..oracle::BENEVOLENT_RIVER_SPIRIT
};
