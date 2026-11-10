use cantrip_api::*;
use cantrip_mechanics::triggers::YOU_ATTACK;
use cantrip_mechanics::{effects, pump, select};
use cantrip_oracle as oracle;

/// "Whenever you attack, you may sacrifice another creature. If you do, draw a card, then Klingons and Warriors you
/// control get +2/+2 until end of turn." The sacrifice is chosen as the ability resolves and may be declined (CR
/// 608.2d, 701.21a), and the card is drawn only if a permanent was actually sacrificed (CR 608.2c); the pump is the
/// "then" that follows it (CR 608.2c), so a declined sacrifice draws nothing and pumps nothing.
fn sacrifice_to_draw_and_pump(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let rule = select::another_creature_permanents(select::up_to(1));
    if effects::may_sacrifice(g, item.controller, item.object, rule)?.is_empty() {
        return Ok(());
    }
    effects::draw(g, item, 1)?;
    // "Klingons and Warriors you control" (CR 611.2c, 613.1): the creatures its controller controls as the ability
    // resolves that are a Klingon, a Warrior or both, each getting +2/+2 together as one instruction. Both are the
    // words of this card's own text, so each subtype is read through them (CR 612.2).
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
            pump(g, item.object, id, 2, 2);
        }
    });
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        YOU_ATTACK,
        sacrifice_to_draw_and_pump,
        "Whenever you attack, you may sacrifice another creature. If you do, draw a card, then Klingons and Warriors you control get +2/+2 until end of turn.",
    )],
    ..oracle::KOR_CAUSE_OF_VENGEANCE
};
