use cantrip_api::*;
use cantrip_mechanics::{behold, counter_target, empower, targets};
use cantrip_oracle as oracle;

/// A Jace, whether a permanent or a card in a hand: a planeswalker type is not a word a text-changing effect changes.
fn jace(q: &dyn Query, _: &Asker, id: ObjectId) -> bool {
    q.chars(id).has_subtype(Subtype::Jace)
}

/// "Counter target spell. Empower Jace 1." A target that is no longer legal stops the whole spell (CR 608.2b), so
/// nothing is empowered; a spell that can't be countered stays on the stack and Jace is still empowered (CR 701.6).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    counter_target(g, item, 0);
    empower::jace(g, item.controller, item.object, 1)
}

/// "As an additional cost to cast this spell, behold a Jace or pay {1}" (CR 601.2b, 701.4a): one of the two, chosen
/// before the target, which is chosen after it.
pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::SPELL], effect),
    late_targets: true,
    additional_choice: &[
        ExtraCost { text: "Behold a Jace", cost: behold::cost(jace, "Jace") },
        ExtraCost { text: "Pay {1}", cost: Cost { mana: mana("{1}"), ..Cost::NONE } },
    ],
    ..oracle::COUNTERSCULPT
};

pub static EXTRA: &[&CardDef] = &[&empower::JACE];
