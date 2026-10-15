use cantrip_api::*;
use cantrip_mechanics::{effects, pump, targets, triggers::this_enters};
use cantrip_oracle as oracle;

/// {T}: Add {C}{C}. This mana can't be spent to cast nonartifact spells (CR 106.6): it pays artifact spells, and
/// abilities and everything else that is not a spell.
fn add_restricted(g: &mut dyn Game, asker: &Asker, _: Option<Color>) -> Result<(), Suspend> {
    g.add_restricted_mana(asker.controller, Mana::C, 2, ManaRestriction::NotNonartifactSpells);
    Ok(())
}

fn minus_five(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        pump(g, item.object, creature, -5, -5);
    }
    Ok(())
}

static MODAL: ModalDef = ModalDef {
    modes: &[
        ModeDef { text: "Draw two cards.", targets: &[], effect: effects::draw::<2>, cost: ManaCost::ZERO },
        ModeDef { text: "Target creature gets -5/-5 until end of turn.", targets: &[targets::CREATURE], effect: minus_five, cost: ManaCost::ZERO },
    ],
    min: 1,
    max: 1,
    escalate: None,
    repeat: false, allowed: None,
};

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "When The Mightstone and Weakstone enters, choose one \u{2014} draw two cards; or target creature gets -5/-5 until end of turn.",
            listens: EventMask::ZONE_CHANGE,
            matches: this_enters,
            modal: Some(&MODAL),
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "{T}: Add {C}{C}. This mana can't be spent to cast nonartifact spells.",
            cost: Cost { tap: true, ..Cost::NONE },
            mana: Some(add_restricted),
            adds: ManaPool([0, 0, 0, 0, 0, 2]),
            restricted: Some(ManaRestriction::NotNonartifactSpells),
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::THE_MIGHTSTONE_AND_WEAKSTONE
};
