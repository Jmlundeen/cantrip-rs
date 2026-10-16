use cantrip_api::*;
use cantrip_mechanics::{tokens, targets};
use cantrip_mechanics::triggers::DEALS_COMBAT_DAMAGE_TO_PLAYER;
use cantrip_oracle as oracle;

/// "Target Wolf you control" (CR 115.1).
fn wolf_you_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id)
        if targets::permanent(q, id) && q.controller(id) == asker.controller
            && q.chars(id).has_subtype(q.words(asker.source).subtype(Subtype::Wolf)))
}

/// Mode 0: a +1/+1 counter on the Wolf (CR 700.2).
fn counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(wolf) = targets::object(g, item, 0) {
        g.add_counters(wolf, CounterKind::PlusOne, 1);
    }
    Ok(())
}

/// Mode 1: a Treasure token.
fn treasure(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create_treasure(g, item.controller)
}

static MODAL: ModalDef = ModalDef {
    modes: &[
        ModeDef {
            text: "Put a +1/+1 counter on target Wolf you control.",
            targets: &[TargetSpec { min: 1, max: 1, filter: wolf_you_control, label: "" }],
            effect: counter,
            cost: ManaCost::ZERO,
        },
        ModeDef { text: "Create a Treasure token.", targets: &[], effect: treasure, cost: ManaCost::ZERO },
    ],
    min: 1,
    max: 1,
    escalate: None,
    repeat: false, allowed: None,
};

pub static CARD: CardDef = CardDef {
    abilities: &[modal_triggered(DEALS_COMBAT_DAMAGE_TO_PLAYER, &MODAL, "Whenever this creature deals combat damage to a player, choose one \u{2014} Put a +1/+1 counter on target Wolf you control; or create a Treasure token.")],
    ..oracle::BEJEWELED_WARG
};

pub static EXTRA: &[&CardDef] = &[&tokens::TREASURE];
