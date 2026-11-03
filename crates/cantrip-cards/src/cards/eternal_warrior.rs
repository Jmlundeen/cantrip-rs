use cantrip_api::*;
use cantrip_mechanics::{attach, statics, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(Layer::Ability, attach::attached_object, statics::have_vigilance, "Enchanted creature has vigilance.")],
    enchant: Some(targets::CREATURE),
    ..oracle::ETERNAL_WARRIOR
};
