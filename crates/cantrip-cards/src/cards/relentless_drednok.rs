use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{2}{B}").and(EXILE_THIS),
        effects::draw_a_card_and_lose_1_life,
        "{2}{B}, Exile this card from your graveyard: You draw a card and lose 1 life.",
    )
    .from(ZoneKind::Graveyard)],
    ..oracle::RELENTLESS_DREDNOK
};
