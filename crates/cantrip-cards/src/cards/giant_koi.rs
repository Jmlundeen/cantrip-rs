use cantrip_api::*;
use cantrip_mechanics::{effects, typecycling};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(
            cost("{3}"),
            effects::this_cant_be_blocked_this_turn,
            "Waterbend {3}: This creature can't be blocked this turn. (While paying a waterbend cost, you can tap your artifacts and creatures to help. Each one pays for {1}.)",
        )
        .waterbend(),
        typecycling::ability(Subtype::Island, mana("{2}"), "Islandcycling {2} ({2}, Discard this card: Search your library for an Island card, reveal it, put it into your hand, then shuffle.)"),
    ],
    ..oracle::GIANT_KOI
};
