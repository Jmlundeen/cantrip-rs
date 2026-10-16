use cantrip_api::*;
use cantrip_mechanics::{effects, pump, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

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
        modal_triggered(THIS_ENTERS, &MODAL, "When The Mightstone and Weakstone enters, choose one \u{2014} draw two cards; or target creature gets -5/-5 until end of turn."),
        mana_ability(TAP, add_n(Mana::C, 2).only_for(ManaRestriction::NotNonartifactSpells), "{T}: Add {C}{C}. This mana can't be spent to cast nonartifact spells."),
    ],
    ..oracle::THE_MIGHTSTONE_AND_WEAKSTONE
};
