use cantrip_api::*;
use cantrip_oracle as oracle;

/// "This creature has base power and toughness 5/2 until end of turn" (CR 613.4b, 611.2): a layer 7b effect on this creature,
/// whatever it is paid with - the Waterbender may itself be one of the creatures tapped for the waterbend cost (CR 701.67b).
fn set_five_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_effect(item.object, item.object, EffectKind::SetPt(5, 2), Duration::EndOfTurn);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{3}"),
        set_five_two,
        "Waterbend {3}: This creature has base power and toughness 5/2 until end of turn. (While paying a waterbend cost, you can tap your artifacts and creatures to help. Each one pays for {1}.)",
    )
    .waterbend()],
    ..oracle::FLEXIBLE_WATERBENDER
};
