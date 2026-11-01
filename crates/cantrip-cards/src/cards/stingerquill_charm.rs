use cantrip_api::*;
use cantrip_mechanics::{effects, haste_until_end_of_turn, targets, tokens};
use cantrip_oracle as oracle;

/// "Target creature gains first strike and deathtouch until end of turn" (CR 611.2a, 613.1f): a layer 6 effect from the
/// spell, on the creature if it is still a legal target (CR 608.2b).
fn first_strike_and_deathtouch(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.add_effect(item.object, creature, EffectKind::AddKeywords(Keywords::of(&[Keyword::FirstStrike, Keyword::Deathtouch])), Duration::EndOfTurn);
    }
    Ok(())
}

/// "Create a 2/2 colorless Wizard Soldier creature token named Cadet. It gains haste until end of turn."
fn create_cadet(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let made = tokens::create(g, item.controller, &tokens::CADET, 1)?;
    g.end_sentence();
    instruction(g, |g| {
        for cadet in made {
            haste_until_end_of_turn(g, item.object, cadet);
        }
    });
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(effects::damage_first_target::<3>, "Stingerquill Charm deals 3 damage to any target.").targets(&[targets::ANY]),
    mode(first_strike_and_deathtouch, "Target creature gains first strike and deathtouch until end of turn.").targets(&[targets::CREATURE]),
    mode(create_cadet, "Create a 2/2 colorless Wizard Soldier creature token named Cadet. It gains haste until end of turn."),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::STINGERQUILL_CHARM };

pub static EXTRA: &[&CardDef] = &[&tokens::CADET];
