use cantrip_api::*;
use cantrip_mechanics::{attach::attached_object, targets};
use cantrip_oracle as oracle;

/// "Enchanted creature gets +2/+2" (layer 7c).
fn plus_two_plus_two(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.power = chars.power.map(|v| v + 2);
    chars.toughness = chars.toughness.map(|v| v + 2);
}

/// "Enchanted creature has first strike and vigilance" (layer 6).
fn first_strike_and_vigilance(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::FirstStrike).with(Keyword::Vigilance);
}

/// "Enchanted creature is a legendary Soldier in addition to its other types" (CR 205.3, layer 4): the supertype and the
/// creature type are added to what it is. Soldier is a word of this card's rules text, read through its words (CR 612).
fn legendary_soldier_too(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    chars.supertypes.legendary = true;
    chars.subtypes = chars.subtypes.with(view.words(source).subtype(Subtype::Soldier));
}

/// "Whenever enchanted creature attacks or blocks" (CR 508.1m, 509.1): the creature this Aura is attached to is declared
/// as an attacker, or as a blocker. The creature is the trigger's subject, which "it" is.
fn enchanted_creature_attacks_or_blocks(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Attacks { attacker: creature, .. } | Event::Blocks { blocker: creature, declared: true, .. } if q.attached_to(source) == Some(*creature) => {
            Fired::about(Target::Object(*creature))
        }
        _ => None,
    }
}

/// "Attach any number of target Equipment you control to it" (CR 701.3a): "it" is the creature that attacked or blocked,
/// as it is now; attaching to a creature that has left the battlefield does nothing (CR 701.3b). Each Equipment still a legal target is attached
/// in turn (CR 608.2b); one that cannot be attached to it (CR 701.3b) stays where it is.
fn attach_equipment(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Object(creature) = item.subject else { return Ok(()) };
    for slot in 0..item.target_count as usize {
        if let Some(equipment) = targets::object(g, item, slot) {
            g.attach(equipment, creature);
        }
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enchant: Some(targets::CREATURE),
    abilities: &[triggered(
        when(EventMask(EventMask::ATTACKS.0 | EventMask::BLOCKS.0), enchanted_creature_attacks_or_blocks),
        attach_equipment,
        "Whenever enchanted creature attacks or blocks, attach any number of target Equipment you control to it.",
    )
    .targets(&[targets::EQUIPMENT_YOU_CONTROL.between(0, TargetSpec::ANY_NUMBER)])],
    statics: &[
        static_ability(Layer::Type, attached_object, legendary_soldier_too, "Enchanted creature is a legendary Soldier in addition to its other types."),
        static_ability(Layer::Ability, attached_object, first_strike_and_vigilance, "Enchanted creature has first strike and vigilance.").continues(),
        static_ability(Layer::PtModify, attached_object, plus_two_plus_two, "Enchanted creature gets +2/+2.").continues(),
    ],
    ..oracle::SUPER_SOLDIER_SERUM
};
