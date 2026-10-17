use cantrip_api::*;
use cantrip_mechanics::{counter_target, targets};
use cantrip_oracle as oracle;

/// "Target creature or enchantment spell" (CR 115.1): a spell on the stack with the card type creature or enchantment, an
/// enchantment creature or an Aura included.
fn creature_or_enchantment_spell(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::SPELL.filter)(q, asker, t) && matches!(t, Target::Object(id) if {
        let chars = q.chars(id);
        chars.is(CardType::Creature) || chars.is(CardType::Enchantment)
    })
}

/// "Counter target creature or enchantment spell" (CR 701.6). Nothing happens if it is no longer a legal target
/// (CR 608.2b).
fn counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    counter_target(g, item, 0);
    Ok(())
}

/// "Target creatures and/or enchantments you own" (CR 108.3, 115.1): on the battlefield and owned by the caster, whoever
/// controls them.
fn creature_or_enchantment_you_own(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    targets::creature_or_enchantment(q, asker, t) && matches!(t, Target::Object(id) if q.obj(id).owner == asker.controller)
}

/// "Return one or two target creatures and/or enchantments you own to your hand": those still legal as it resolves
/// (CR 608.2b) are returned together (CR 400.3), to their owner's hand, which is the caster's.
fn return_to_hand(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let permanents = targets::legal_objects(g, item);
    g.return_to_hand(&permanents);
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(counter, "Counter target creature or enchantment spell.").targets(&[target(creature_or_enchantment_spell)]),
    mode(return_to_hand, "Return one or two target creatures and/or enchantments you own to your hand.")
        .targets(&[target(creature_or_enchantment_you_own).between(1, 2)]),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::GET_OUT };
