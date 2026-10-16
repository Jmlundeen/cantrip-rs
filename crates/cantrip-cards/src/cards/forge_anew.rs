use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "Target Equipment card from your graveyard" (CR 115.1).
fn equipment_in_your_graveyard(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    cantrip_mechanics::targets::card_in_your_graveyard(q, asker, t, &[CardType::Artifact])
        && matches!(t, Target::Object(id) if q.chars(id).has_subtype(Subtype::Equipment))
}

/// When this enters, return target Equipment card from your graveyard to the battlefield.
fn return_equipment(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(card) = targets::object(g, item, 0) {
        g.put_onto_battlefield(card, item.controller, false)?;
    }
    Ok(())
}

/// Whether the ability `index` of `source` is an equip ability (CR 702.6a).
fn is_equip(q: &dyn Query, ability: AbilityRef) -> bool {
    matches!(q.card(ability.card).abilities.get(ability.index as usize), Some(AbilityDef::Activated(d)) if d.equip)
}

/// "During your turn, you may activate equip abilities any time you could cast an instant" (CR 702.6a, 602.5d).
fn instant_equip(q: &dyn Query, asker: &Asker, source: ObjectId, index: u8, activator: PlayerId) -> bool {
    activator == asker.controller
        && q.state().turn.active == asker.controller
        && q.chars(source).abilities().nth(index as usize).is_some_and(|a| is_equip(q, a))
}

/// "You may pay {0} rather than pay the equip cost of the first equip ability you activate during each of your turns"
/// (CR 118.9, 602.2b): the first counted over every equip ability they activated this turn, with or without this there.
fn free_first_equip(q: &dyn Query, asker: &Asker, activator: PlayerId, _: ObjectId, def: &ActivatedDef) -> Option<ManaCost> {
    if !def.equip || activator != asker.controller || q.state().turn.active != asker.controller {
        return None;
    }
    let activated_one = q.history().this_turn().iter().any(|r| {
        matches!(r.event, Event::AbilityActivated { ability, player, .. } if player == activator && is_equip(q, ability))
    });
    if activated_one { None } else { Some(ManaCost::ZERO) }
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, return_equipment, "When this enchantment enters, return target Equipment card from your graveyard to the battlefield.")
        .targets(&[TargetSpec { min: 1, max: 1, filter: equipment_in_your_graveyard }])],
    instant_activation: &[instant_equip],
    ability_alternatives: &[free_first_equip],
    ..oracle::FORGE_ANEW
};
