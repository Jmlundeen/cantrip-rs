use cantrip_api::*;
use cantrip_mechanics::{effects, haste_until_end_of_turn, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// Creatures you control gain haste until end of turn.
fn haste(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let creatures: Vec<ObjectId> =
        g.permanents_of(item.controller).into_iter().filter(|&id| g.chars(id).is(CardType::Creature)).collect();
    for id in creatures {
        haste_until_end_of_turn(g, item.object, id);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, |g, item| effects::may_discard_to_draw(g, item, 2), "When Bitter Reunion enters, you may discard a card. If you do, draw two cards."),
        ability(cost("{1}").and(SACRIFICE_THIS), haste, "{1}, Sacrifice Bitter Reunion: Creatures you control gain haste until end of turn."),
    ],
    ..oracle::BITTER_REUNION
};
