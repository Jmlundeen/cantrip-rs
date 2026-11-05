use cantrip_api::*;
use cantrip_mechanics::battle_cry;
use cantrip_mechanics::triggers::SQUAD;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    // Squad {2} (CR 702.157a): the additional cost, paid any number of times, is the card's multikicker.
    kicker: multikicker(mana("{2}")),
    abilities: &[SQUAD, battle_cry::TRIGGER],
    ..oracle::IMPERIAL_HULLBREACHERS
};
