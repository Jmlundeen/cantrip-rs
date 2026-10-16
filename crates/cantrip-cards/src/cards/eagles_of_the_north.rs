use cantrip_api::*;
use cantrip_mechanics::typecycling;
use cantrip_mechanics::{pump, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// Creatures you control get +1/+0 and gain first strike until end of turn: the creatures there are as it resolves
/// (CR 611.2c).
fn pump_creatures(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let creatures: Vec<ObjectId> =
        g.permanents_of(item.controller).into_iter().filter(|&id| g.chars(id).is(CardType::Creature)).collect();
    for id in creatures {
        pump(g, item.object, id, 1, 0);
        g.add_effect(item.object, id, EffectKind::AddKeywords(Keywords::of(&[Keyword::FirstStrike])), Duration::EndOfTurn);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, pump_creatures, "When Eagles of the North enters, creatures you control get +1/+0 and gain first strike until end of turn."),
        typecycling::ability(Subtype::Plains, mana("{1}"), "Plainscycling {1} ({1}, Discard this card: Search your library for a Plains card, reveal it, put it into your hand, then shuffle.)"),
    ],
    ..oracle::EAGLES_OF_THE_NORTH
};
