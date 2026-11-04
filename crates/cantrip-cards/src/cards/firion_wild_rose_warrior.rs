use cantrip_api::*;
use cantrip_mechanics::attach::equipment_you_control_enters;
use cantrip_mechanics::delayed;
use cantrip_mechanics::statics::{equipped_creatures_you_control, have_haste};
use cantrip_oracle as oracle;

/// "This Equipment's equip abilities cost {2} less to activate" (CR 601.2f, 602.2b): any equip ability of the object that
/// has this text, whoever activates it and whatever it targets, less generic mana only (CR 118.7a) and never below {0}.
fn its_equip_abilities_cost_two_less(q: &dyn Query, source: ObjectId, _: PlayerId, costed: Costed, _: &[Target]) -> CostDelta {
    match costed {
        Costed::Ability(object, ability, _) if object == source && q.activated_ability(ability).is_some_and(|d| d.equip) => CostDelta::less(2),
        _ => CostDelta::NONE,
    }
}

/// The text the token copy has by the copy's exception (CR 707.9b). Not a card anyone plays: a definition for the exception's
/// static ability to be read from. It is part of the token's copiable values, so a copy of the token has it too.
pub static EXCEPTION: CardDef = CardDef {
    name: "Firion, Wild Rose Warrior (copy exception)",
    text: "This Equipment's equip abilities cost {2} less to activate.",
    cost_changes: &[its_equip_abilities_cost_two_less],
    ..CardDef::BLANK
};

/// "Whenever a nontoken Equipment you control enters" (CR 603.6a, 111.1): the entering Equipment, a card and not a token.
fn nontoken_equipment_you_control_enters(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    let fired = equipment_you_control_enters(q, source, event)?;
    match fired.subject {
        Target::Object(equipment) if !q.obj(equipment).token => Some(fired),
        _ => None,
    }
}

/// "Create a token that's a copy of it, except it has [the exception's text]. Sacrifice that token at the beginning of the next
/// upkeep." (CR 707.2, 707.9b, 111.1, 603.7c): the Equipment's copiable values as it last existed on the battlefield, under the
/// ability's controller; the delayed ability is about the token made, so it sacrifices that token only while it is still the
/// object it was (CR 400.7) and this ability's controller controls it (CR 701.21a).
fn copy_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Object(equipment) = item.subject else { return Ok(()) };
    let values = g.copiable_values(equipment).with_text(g.card_id(&EXCEPTION));
    let tokens = g.create_token_copies_of(values, item.controller, 1)?;
    delayed::at_next_upkeep_about(g, item, 0, &tokens);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(Layer::Ability, equipped_creatures_you_control, have_haste, "Equipped creatures you control have haste.")],
    abilities: &[triggered(
        when(EventMask::ZONE_CHANGE, nontoken_equipment_you_control_enters),
        copy_it,
        "Whenever a nontoken Equipment you control enters, create a token that's a copy of it, except it has \"This Equipment's equip abilities cost {2} less to activate.\" Sacrifice that token at the beginning of the next upkeep.",
    )],
    delayed: &[delayed_trigger(delayed::sacrifice_created, "Sacrifice that token at the beginning of the next upkeep.")],
    ..oracle::FIRION_WILD_ROSE_WARRIOR
};

pub static EXTRA: &[&CardDef] = &[&EXCEPTION];
