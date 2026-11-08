use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "{T}: Add {C}{C}." (CR 605.1a, 107.5): a mana ability - it has no target, it could add mana and it is not a loyalty
/// ability - whose only cost is the tap, and which puts the two colorless mana in the pool together.
///
/// "{2}, {T}, Sacrifice this artifact: Draw two cards." (CR 107.5, 701.21a, 121.1): an activated ability paid with two
/// generic mana, the tap and the sacrifice of the artifact itself, drawing two cards once it resolves.
pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add_n(Mana::C, 2), "{T}: Add {C}{C}."),
        ability(cost("{2}").and(TAP).and(SACRIFICE_THIS), |g, item| effects::draw(g, item, 2),
            "{2}, {T}, Sacrifice this artifact: Draw two cards."),
    ],
    ..oracle::HEDRON_ARCHIVE
};
