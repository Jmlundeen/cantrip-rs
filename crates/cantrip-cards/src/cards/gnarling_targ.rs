use cantrip_api::*;
use cantrip_mechanics::{pump, trample_until_end_of_turn, triggers};
use cantrip_oracle as oracle;

/// "Klingons and Warriors you control get +2/+0 and gain trample until end of turn." (CR 611.2c, 613.1): the creatures its
/// controller controls as the ability resolves that are a Klingon, a Warrior or both - none that arrives later - each getting
/// +2/+0 and trample together as one instruction. "Klingons" and "Warriors" are the words of this card's own text, so each
/// subtype is read through them (CR 612.2); Gnarling Targ is a Boar, so it is not one of them.
fn pump_klingons_and_warriors(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let klingon = g.words(item.object).subtype(Subtype::Klingon);
    let warrior = g.words(item.object).subtype(Subtype::Warrior);
    let mut creatures: Vec<ObjectId> = Vec::new();
    for id in g.permanents_of(item.controller) {
        let chars = g.chars(id);
        if chars.is(CardType::Creature) && (chars.has_subtype(klingon) || chars.has_subtype(warrior)) {
            creatures.push(id);
        }
    }
    instruction(g, |g| {
        for id in creatures {
            pump(g, item.object, id, 2, 0);
            trample_until_end_of_turn(g, item.object, id);
        }
    });
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::YOU_ATTACK,
        pump_klingons_and_warriors,
        "Whenever you attack, Klingons and Warriors you control get +2/+0 and gain trample until end of turn.",
    )],
    ..oracle::GNARLING_TARG
};
