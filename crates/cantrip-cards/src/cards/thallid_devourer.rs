use cantrip_api::*;
use cantrip_mechanics::{effects, spore, tokens};
use cantrip_oracle as oracle;

/// "Sacrifice a Saproling: This creature gets +1/+2 until end of turn." (CR 118.1).
fn creature_gets_1_2_2(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    q.chars(id).has_subtype(q.words(asker.source).subtype(Subtype::Saproling))
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        spore::UPKEEP,
        spore::SAPROLING,
        ability(
            sacrifice(1, creature_gets_1_2_2),
            effects::this_gets::<1, 2>,
            "Sacrifice a Saproling: This creature gets +1/+2 until end of turn.",
        ),
    ],
    ..oracle::THALLID_DEVOURER
};

pub static EXTRA: &[&CardDef] = &[&tokens::SAPROLING];
