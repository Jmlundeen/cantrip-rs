use cantrip_api::*;
use cantrip_mechanics::{mana as mana_effects, targets::filters::creature};
use cantrip_oracle as oracle;

/// "Put this card from your hand onto the battlefield." (CR 602.2, 400.7.) The card enters tapped by its own replacement
/// effect (CR 614.1c). A card that has left the hand since the ability was activated is a new object (CR 400.7), which the
/// verb leaves where it is.
fn put_onto_battlefield(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.put_onto_battlefield(item.object, item.controller, false)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(mana_effects::enters_tapped),
    abilities: &[
        mana_ability(TAP, add_one_of(&[Color::Green, Color::White, Color::Blue]), "{T}: Add {G}, {W}, or {U}."),
        ability(
            Cost::mana(mana("{2}")).and(return_tapped_to_hand(1, creature)),
            put_onto_battlefield,
            "{2}, Return a tapped creature you control to its owner's hand: Put this card from your hand onto the battlefield. Activate only as a sorcery.",
        )
        .from(ZoneKind::Hand)
        .sorcery_speed(),
    ],
    ..oracle::URBAN_RETREAT
};
