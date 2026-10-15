use cantrip_api::*;
use cantrip_mechanics::mana;
use cantrip_oracle as oracle;

/// Draw a card, then you lose life equal to the number of cards in your hand.
fn draw_then_lose_life(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    g.draw(player, 1);
    let cards = g.state().zone(Zone::hand(player)).len() as i32;
    if cards > 0 {
        g.lose_life(player, cards);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(cantrip_mechanics::lands::enters_tapped_unless_you_control::<{ cantrip_mechanics::lands::sub(Subtype::Swamp) }>),
    abilities: &[
        mana::tap_for(mana::add_b, Mana::B, "{T}: Add {B}."),
        AbilityDef::Activated(ActivatedDef {
            text: "{1}{B}{B}, {T}: Draw a card, then you lose life equal to the number of cards in your hand.",
            cost: Cost { mana: mana(1, 0, 0, 2, 0, 0), tap: true, ..Cost::NONE },
            effect: draw_then_lose_life,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::CASTLE_LOCTHWAIN
};
