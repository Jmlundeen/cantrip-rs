use cantrip_api::*;
use cantrip_mechanics::{effects, spree};
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers::deals_combat_damage_to_player;
use cantrip_oracle as oracle;

/// Mode 0: "Put two +1/+1 counters on target creature."
fn counters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.add_counters(creature, CounterKind::PlusOne, 2);
    }
    Ok(())
}

/// Mode 1: "Target creature gains trample until end of turn."
fn trample(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.add_effect(item.object, creature, EffectKind::AddKeywords(Keywords::of(&[Keyword::Trample])), Duration::EndOfTurn);
    }
    Ok(())
}

/// Mode 2: "Until end of turn, target creature gains 'Whenever this creature deals combat damage to a player, draw two
/// cards.'" A continuous effect fixed to the creature as the spell resolves (CR 611.2c) that grants the triggered
/// ability in layer 6 (CR 613.1f), the creature's own, so its controller draws.
fn grant_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        let this = g.card_id(&CARD);
        g.add_effect(item.object, creature, EffectKind::AddAbility(AbilityRef::delayed(this, 0)), Duration::EndOfTurn);
    }
    Ok(())
}

const MODES: &[ModeDef] = &[
    ModeDef {
        text: "+ {2} \u{2014} Put two +1/+1 counters on target creature.",
        targets: &[targets::CREATURE],
        effect: counters,
        cost: mana(2, 0, 0, 0, 0, 0),
    },
    ModeDef {
        text: "+ {1} \u{2014} Target creature gains trample until end of turn.",
        targets: &[targets::CREATURE],
        effect: trample,
        cost: mana(1, 0, 0, 0, 0, 0),
    },
    ModeDef {
        text: "+ {1} \u{2014} Until end of turn, target creature gains \"Whenever this creature deals combat damage to a player, draw two cards.\"",
        targets: &[targets::CREATURE],
        effect: grant_draw,
        cost: mana(1, 0, 0, 0, 0, 0),
    },
];

static MODAL: ModalDef = spree::modal(MODES);

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef::MODAL),
    modal: Some(&MODAL),
    delayed: &[AbilityDef::Triggered(TriggeredDef {
        text: "Whenever this creature deals combat damage to a player, draw two cards.",
        listens: EventMask::DAMAGE,
        matches: deals_combat_damage_to_player,
        effect: effects::draw::<2>,
        ..TriggeredDef::BLANK
    })],
    ..oracle::TRASH_THE_TOWN
};
