use cantrip_api::*;
use cantrip_mechanics::{delayed, destroy_target, haste_until_end_of_turn, mobilize, targets, tokens};
use cantrip_oracle as oracle;

/// "Destroy target creature. Create two 1/1 red Warrior creature tokens. They gain haste until end of turn. Sacrifice them at the
/// beginning of the next end step." (CR 701.8, 111.1, 611.2a, 603.7c): the two Warriors enter together, whether or not the
/// creature was destroyed (indestructible, a regeneration shield), and a delayed ability about exactly them sacrifices them. If the
/// only target is no longer legal the spell does not resolve at all (CR 608.2b) and this is not run: nothing is made.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    g.end_sentence();
    let made = tokens::create(g, item.controller, &mobilize::WARRIOR, 2)?;
    g.end_sentence();
    instruction(g, |g| {
        for &token in &made {
            haste_until_end_of_turn(g, item.object, token);
        }
    });
    g.end_sentence();
    delayed::at_next_end_step_about(g, item, 0, &made);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], effect),
    delayed: &[delayed::SACRIFICE_CREATED],
    ..oracle::SALT_ROAD_SKIRMISH
};

pub static EXTRA: &[&CardDef] = &[&mobilize::WARRIOR];
