use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::seek_out::seek_out;
use cantrip_oracle as oracle;

/// "Whenever you draw your second card each turn" (CR 603.2): the trigger counts the cards its controller draws in one turn,
/// whoever's turn it is. It deals 2 damage to each opponent as it resolves (CR 120.3a) - each opponent in a
/// multiplayer game, and the damage is dealt at once, as one event.
fn sear_each_opponent(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::damage_each_opponent(g, item, 2);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        seek_out(
            sear_each_opponent,
            &[],
            "Seek out — Whenever you draw your second card each turn, Ro Laren deals 2 damage to each opponent.",
        ),
        ability(
            cost("{1}").and(TAP).and(discard_cards(1)),
            |g, item| effects::draw(g, item, 1),
            "{1}, {T}, Discard a card: Draw a card.",
        ),
    ],
    ..oracle::RO_LAREN_DOUBLE_AGENT
};
