use cantrip_api::*;
use cantrip_mechanics::{library::object_names, pump, targets};
use cantrip_oracle as oracle;

const NAME: &str = "Ancestral Anger";

/// "Target creature gains trample and gets +X/+0 until end of turn, where X is 1 plus the number of cards named
/// Ancestral Anger in your graveyard. Draw a card." X is counted as the spell resolves, so the spell itself - still on
/// the stack - is not among them (CR 608.2h, 201.2). "Named Ancestral Anger" is the fixed name, not the
/// spell's current name (a text-changing or copy effect can't change which cards count). If the creature is no longer a legal target, the spell does not
/// resolve (CR 608.2b) and nothing, the draw included, happens.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let named = g
        .state()
        .zone(Zone::graveyard(item.controller))
        .to_vec()
        .into_iter()
        .filter(|&card| object_names(g, card).contains(&NAME))
        .count();
    instruction(g, |g| {
        cantrip_mechanics::trample_until_end_of_turn(g, item.object, creature);
        pump(g, item.object, creature, 1 + named as i16, 0);
    });
    g.end_sentence();
    g.draw(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::CREATURE], effect), ..oracle::ANCESTRAL_ANGER };
