use cantrip_api::*;
use cantrip_mechanics::{effects, prepare, targets};
use cantrip_oracle as oracle;

/// Blossom-Blessed Angel's prepare spell (CR 722): not a card, only ever a copy.
pub static SEED_SUTURE: CardDef = CardDef {
    name: "Seed Suture",
    cost: mana("{G/W}"),
    types: TypeSet::of(&[CardType::Sorcery]),
    spell: spell(&[targets::CREATURE], seed_suture),
    text: "Put a +1/+1 counter on target creature. You gain 1 life.",
    ..CardDef::BLANK
};

/// "Put a +1/+1 counter on target creature. You gain 1 life." (CR 122.1, 119.3.)
fn seed_suture(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.add_counters(creature, CounterKind::PlusOne, 1);
    }
    effects::gain_life(g, item, 1)
}

pub static CARD: CardDef = CardDef {
    enters: Some(prepare::enters_prepared),
    prepare: Some(&SEED_SUTURE),
    ..oracle::BLOSSOM_BLESSED_ANGEL
};

pub static EXTRA: &[&CardDef] = &[&SEED_SUTURE];
