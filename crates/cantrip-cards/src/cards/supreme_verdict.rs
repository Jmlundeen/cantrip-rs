use cantrip_api::*;
use cantrip_mechanics::effects::destroy_all_creatures;
use cantrip_oracle as oracle;

// "This spell can't be countered" (CR 101.2) is the core keyword `CantBeCountered`; "Destroy all creatures" is the
// shared `destroy_all_creatures` (CR 701.8), the same as Day of Judgment.
pub static CARD: CardDef = CardDef {
    spell: spell(&[], destroy_all_creatures),
    ..oracle::SUPREME_VERDICT
};
