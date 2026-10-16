use cantrip_api::*;
use cantrip_mechanics::triggers::NONCREATURE_SPELL_YOU_CAST;
use cantrip_mechanics::{discard_chosen, pump};
use cantrip_oracle as oracle;

/// Creatures you control get +1/+1 until end of turn, and untap those creatures. The creatures are fixed as the ability
/// resolves (CR 611.2c): "those creatures" is exactly that set.
fn pump_untap(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let creatures: Vec<ObjectId> =
        g.permanents_of(item.controller).into_iter().filter(|&id| g.chars(id).is(CardType::Creature)).collect();
    for &creature in &creatures {
        pump(g, item.object, creature, 1, 1);
    }
    for &creature in &creatures {
        g.untap(creature);
    }
    Ok(())
}

/// You may draw a card. If you do, discard a card (CR 608.2c): "if you do" is whether a card was drawn, not whether
/// the player said yes.
fn loot(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    if !cantrip_mechanics::effects::ask_may(g, player, item.object, Purpose::Optional)? {
        return Ok(());
    }
    let before = g.state().zone(Zone::hand(player)).len();
    g.draw(player, 1);
    if g.state().zone(Zone::hand(player)).len() > before {
        discard_chosen(g, player, item.object, 1)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(NONCREATURE_SPELL_YOU_CAST, pump_untap, "Whenever you cast a noncreature spell, creatures you control get +1/+1 until end of turn. Untap those creatures."),
        triggered(NONCREATURE_SPELL_YOU_CAST, loot, "Whenever you cast a noncreature spell, you may draw a card. If you do, discard a card."),
    ],
    ..oracle::JESKAI_ASCENDANCY
};
