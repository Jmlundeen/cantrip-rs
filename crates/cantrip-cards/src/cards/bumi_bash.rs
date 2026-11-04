use cantrip_api::*;
use cantrip_mechanics::{damage_target, destroy_target, lands_you_control, targets};
use cantrip_oracle as oracle;

/// "Bumi Bash deals damage equal to the number of lands you control to target creature": the lands its controller has as it
/// resolves (CR 608.2h), through the layers.
fn damage_by_lands(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let lands = lands_you_control(g, item.controller) as u16;
    damage_target(g, item, 0, lands);
    Ok(())
}

/// "Destroy target land creature or nonbasic land" (CR 115.1, 205.2): a land that is also a creature, or any land that is
/// not basic.
fn land_creature_or_nonbasic_land(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if targets::permanent(q, id) && { let c = q.chars(id); c.is(CardType::Land) && (c.is(CardType::Creature) || !c.supertypes.basic) })
}

fn destroy_land(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(damage_by_lands, "Bumi Bash deals damage equal to the number of lands you control to target creature.").targets(&[targets::CREATURE]),
    mode(destroy_land, "Destroy target land creature or nonbasic land.").targets(&[target(land_creature_or_nonbasic_land)]),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::BUMI_BASH };
