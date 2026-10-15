use cantrip_api::*;
use cantrip_mechanics::typecycling;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    keywords: Keywords::of(&[Keyword::CantBeBlockedExceptByThreeOrMore]),
    abilities: &[typecycling::ability(Subtype::Swamp, mana(1, 0, 0, 0, 0, 0), "Swampcycling {1} ({1}, Discard this card: Search your library for a Swamp card, reveal it, put it into your hand, then shuffle.)")],
    ..oracle::TROLL_OF_KHAZAD_D_M
};
