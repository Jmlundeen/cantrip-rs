use cantrip_api::*;
use cantrip_mechanics::{effects, haste_until_end_of_turn, targets, tokens};
use cantrip_oracle as oracle;

/// "Artistic Process deals 2 damage to each creature you don't control": every creature an opponent controls (CR 608.2c), all
/// dealt together.
fn deal_2_to_each(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::damage_each_creature_opponents_control(g, item, 2);
    Ok(())
}

/// "Create a 3/3 blue and red Elemental creature token with flying. It gains haste until end of turn" (CR 611.2a): a layer 6
/// effect from the spell, on the token alone and lasting the turn.
fn create_elemental(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let made = tokens::create(g, item.controller, &tokens::ELEMENTAL_3_3_BLUE_RED, 1)?;
    g.end_sentence();
    instruction(g, |g| {
        for elemental in made {
            haste_until_end_of_turn(g, item.object, elemental);
        }
    });
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(effects::damage_first_target::<6>, "Artistic Process deals 6 damage to target creature.").targets(&[targets::CREATURE]),
    mode(deal_2_to_each, "Artistic Process deals 2 damage to each creature you don't control."),
    mode(create_elemental, "Create a 3/3 blue and red Elemental creature token with flying. It gains haste until end of turn."),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::ARTISTIC_PROCESS };

pub static EXTRA: &[&CardDef] = &[&tokens::ELEMENTAL_3_3_BLUE_RED];
