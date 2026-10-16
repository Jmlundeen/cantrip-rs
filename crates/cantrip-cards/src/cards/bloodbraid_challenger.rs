use cantrip_api::*;
use cantrip_mechanics::escape;
use cantrip_mechanics::triggers::CASCADE;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[CASCADE],
    ways: &[escape::way(mana("{3}{R}{G}"), 3, "Escape—{3}{R}{G}, Exile three other cards from your graveyard")],
    ..oracle::BLOODBRAID_CHALLENGER
};
