use cantrip_api::*;
use cantrip_mechanics::combat;
use cantrip_mechanics::tokens::FAERIE;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

const FAERIE_FOR_EACH: &str = "Whenever Worf attacks, for each attacking non-Faerie creature, create a tapped and attacking 1/1 blue Faerie creature token with flying.";

/// "for each attacking non-Faerie creature" (CR 608.2h, 508.4): the count is taken as the trigger resolves, over every
/// attacking creature whose subtypes are not "Faerie" (CR 612.2) - Worf himself among them - and the Faeries are made together.
fn faerie_for_each_attacker(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let faerie_word = g.words(item.object).subtype(Subtype::Faerie);
    let attacking = g.state().attackers.iter();
    let count = attacking.filter(|a| g.chars(a.id).is(CardType::Creature) && !g.chars(a.id).has_subtype(faerie_word)).count();
    if count == 0 { return Ok(()); }
    let faerie = g.card_id(&FAERIE);
    combat::create_attacking_of_your_choice(g, item.controller, item.object, faerie, count, true)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(triggers::THIS_ATTACKS, faerie_for_each_attacker, FAERIE_FOR_EACH)],
    ..oracle::WORF_STILL_NOT_MERRY
};
