use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::triggers::{DRAGON_YOU_CONTROL_ENTERS, THIS_ENTERS};
use cantrip_mechanics::{behold, tokens};
use cantrip_oracle as oracle;

/// "When Sarkhan enters, you may behold a Dragon. If you do, create a Treasure token." (CR 701.4a, 608.2d): the choice of a
/// Dragon permanent you control or a Dragon card revealed from your hand is made as the ability resolves, and the Treasure
/// follows only if one was beheld (nothing is asked, and no Treasure made, with no Dragon to behold).
fn behold_a_dragon(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if behold::on_resolution(g, item.controller, item.object, behold::dragon, "Dragon")? {
        tokens::create_treasure(g, item.controller)?;
    }
    Ok(())
}

/// "Put a +1/+1 counter on Sarkhan. Until end of turn, Sarkhan becomes a Dragon in addition to his other types and gains
/// flying." (CR 611.2, 205.1b, 613.1): he gets the counter, the Dragon type (a layer 4
/// effect that adds to his types, read through his text's creature type word) and flying (layer 6) until the turn ends.
fn grow(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let sarkhan = item.object;
    g.add_counters(sarkhan, CounterKind::PlusOne, 1);
    g.end_sentence();
    let dragon = g.words(sarkhan).subtype(Subtype::Dragon);
    instruction(g, |g| {
        g.add_effect(sarkhan, sarkhan, EffectKind::AddSubtype(dragon), Duration::EndOfTurn);
        effects::gain_until_end_of_turn(g, sarkhan, sarkhan, Keyword::Flying);
    });
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, behold_a_dragon, "When Sarkhan enters, you may behold a Dragon. If you do, create a Treasure token."),
        triggered(DRAGON_YOU_CONTROL_ENTERS, grow, "Whenever a Dragon you control enters, put a +1/+1 counter on Sarkhan. Until end of turn, Sarkhan becomes a Dragon in addition to his other types and gains flying."),
    ],
    ..oracle::SARKHAN_DRAGON_ASCENDANT
};
