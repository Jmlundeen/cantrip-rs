use cantrip_api::*;
use cantrip_mechanics::{effects, lands, targets};
use cantrip_oracle as oracle;

/// "{2}, {T}, Sacrifice a Desert: Exile all graveyards." (CR 406.2, 608.2c).
fn exile_all_graveyards(g: &mut dyn Game, _: &StackItem) -> Result<(), Suspend> {
    effects::exile_all_graveyards(g);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        ability(
            cost("{2}").and(TAP).and(sacrifice(1, targets::filters::land_type::<{ lands::sub(Subtype::Desert) }>)),
            exile_all_graveyards,
            "{2}, {T}, Sacrifice a Desert: Exile all graveyards.",
        ),
    ],
    ..oracle::SCAVENGER_GROUNDS
};
