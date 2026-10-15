use cantrip_api::*;
use cantrip_mechanics::saddle;
use cantrip_mechanics::library::reveal_top_to_hand;
use cantrip_oracle as oracle;

/// Reveal the top card of your library and put it into your hand. You lose life equal to that card's mana value if
/// this creature isn't saddled; otherwise each opponent loses that much life. (Nothing to reveal, nothing is lost.)
fn reveal_and_bleed(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some((_, value)) = reveal_top_to_hand(g, item.controller) else { return Ok(()) };
    if g.is_saddled(item.object) {
        g.each_opponent_loses_life(item.controller, value as i32);
    } else {
        g.lose_life(item.controller, value as i32);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "Whenever this creature attacks, reveal the top card of your library and put it into your hand. You lose life equal to that card's mana value if this creature isn't saddled. Otherwise, each opponent loses that much life.",
            listens: EventMask::ATTACKS,
            matches: cantrip_mechanics::triggers::this_attacks,
            effect: reveal_and_bleed,
            ..TriggeredDef::BLANK
        }),
        saddle::ability(
            3,
            "Saddle 3 (Tap any number of other creatures you control with total power 3 or more: This Mount becomes saddled until end of turn. Saddle only as a sorcery.)",
        ),
    ],
    ..oracle::CAUSTIC_BRONCO
};
