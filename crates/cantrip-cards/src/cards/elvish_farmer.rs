use cantrip_api::*;
use cantrip_mechanics::{spore, tokens};
use cantrip_oracle as oracle;

/// "Sacrifice a Saproling: You gain 2 life." (CR 608.2, 119.3).
fn gain_2_life(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.gain_life(item.controller, 2);
    Ok(())
}

/// "Sacrifice a Saproling: You gain 2 life." (CR 608.2).
fn gain_2_life_2(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    q.chars(id).has_subtype(q.words(asker.source).subtype(Subtype::Saproling))
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        spore::UPKEEP,
        spore::SAPROLING,
        ability(
            sacrifice(1, gain_2_life_2),
            gain_2_life,
            "Sacrifice a Saproling: You gain 2 life.",
        ),
    ],
    ..oracle::ELVISH_FARMER
};

pub static EXTRA: &[&CardDef] = &[&tokens::SAPROLING];
