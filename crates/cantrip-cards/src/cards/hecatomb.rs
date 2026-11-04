use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers};
use cantrip_oracle as oracle;

/// "When this enchantment enters, sacrifice this enchantment unless you sacrifice four creatures." (CR 118.12, 701.21a).
fn sacrifice_unless_four(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::sacrifice_this_unless_pay(g, item, sacrifice(4, targets::filters::creature))
}

/// "Tap an untapped Swamp you control" (CR 118.3): the land type word follows text-changing effects.
fn untapped_swamp(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    !q.obj(id).tapped
        && q.chars(id).has_subtype(q.words(asker.source).subtype(Subtype::Swamp))
        && asker.controller == q.controller(id)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::THIS_ENTERS, sacrifice_unless_four,
            "When this enchantment enters, sacrifice this enchantment unless you sacrifice four creatures."),
        ability(tap_untapped(1, untapped_swamp), effects::damage_first_target::<1>,
            "Tap an untapped Swamp you control: This enchantment deals 1 damage to any target.")
            .targets(&[targets::ANY]),
    ],
    ..oracle::HECATOMB
};
