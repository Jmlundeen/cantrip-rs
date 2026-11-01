use cantrip_api::*;
use cantrip_mechanics::{lands, targets};
use cantrip_oracle as oracle;

/// "Target player mills four cards." (CR 701.17a): nothing if the target is gone (CR 608.2b).
fn mill_four(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(player) = targets::player(g, item, 0) else { return Ok(()) };
    g.mill(player, 4);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        mana_ability(TAP.and(life(1)), add(Mana::U), "{T}, Pay 1 life: Add {U}."),
        ability(
            cost("{1}{U}, {T}").and(sacrifice(1, targets::filters::land_type::<{ lands::sub(Subtype::Desert) }>)),
            mill_four,
            "{1}{U}, {T}, Sacrifice a Desert: Target player mills four cards.",
        )
        .targets(&[targets::PLAYER]),
    ],
    ..oracle::IPNU_RIVULET
};
