use cantrip_api::*;
use cantrip_mechanics::destroy_target;
use cantrip_mechanics::spree;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

fn destroy_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

/// Put a +1/+1 counter on each creature target player controls: which creatures is read as the mode resolves
/// (CR 608.2c).
fn counters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(player) = targets::player(g, item, 0) else { return Ok(()) };
    let creatures: Vec<ObjectId> =
        g.permanents_of(player).into_iter().filter(|&id| g.chars(id).is(CardType::Creature)).collect();
    for creature in creatures {
        g.add_counters(creature, CounterKind::PlusOne, 1);
    }
    Ok(())
}

const MODES: &[ModeDef] = &[
    mode(destroy_it, "+ {1} — Destroy target artifact.")
        .targets(&[targets::ARTIFACT])
        .cost(mana("{1}")),
    mode(destroy_it, "+ {1} — Destroy target enchantment.")
        .targets(&[targets::ENCHANTMENT])
        .cost(mana("{1}")),
    mode(counters, "+ {1} — Put a +1/+1 counter on each creature target player controls.")
        .targets(&[targets::PLAYER])
        .cost(mana("{1}")),
];

static MODAL: ModalDef = spree::modal(MODES);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::REQUISITION_RAID };
