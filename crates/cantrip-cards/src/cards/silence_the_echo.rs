use cantrip_api::*;
use cantrip_mechanics::{destroy_target, targets};
use cantrip_oracle as oracle;

/// "As an additional cost to cast this spell, sacrifice a creature or planeswalker or pay {3}" (CR 601.2b, 118.8):
/// one of two additional costs, announced as the spell is cast, before its target is chosen (CR 601.2c). A creature
/// and a planeswalker are the same kind of choice here - either is one permanent, and its controller anywhere.
fn creature_or_planeswalker(q: &dyn Query, _: &Asker, id: ObjectId) -> bool {
    let chars = q.chars(id);
    chars.is(CardType::Creature) || chars.is(CardType::Planeswalker)
}

/// "Destroy target creature or planeswalker." (CR 701.8a, 115.1.)
fn destroy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_OR_PLANESWALKER], destroy),
    additional_choice: &[
        ExtraCost { text: "Sacrifice a creature or planeswalker", cost: Cost::sacrifice(1, creature_or_planeswalker) },
        ExtraCost { text: "Pay {3}", cost: Cost::mana(mana("{3}")) },
    ],
    ..oracle::SILENCE_THE_ECHO
};
