use cantrip_api::*;
use cantrip_mechanics::{effects, untapped_creature_you_control};
use cantrip_oracle as oracle;

/// "Tap three untapped artifact creatures you control" (CR 118.3, 602.2b; not the {T} symbol, CR 302.6).
fn untapped_artifact_creature_you_control(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    untapped_creature_you_control(q, asker, id) && q.chars(id).is(CardType::Artifact)
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        tap_untapped(3, untapped_artifact_creature_you_control),
        effects::draw_a_card_and_lose_1_life,
        "Tap three untapped artifact creatures you control: You draw a card and lose 1 life.",
    )],
    ..oracle::COLLATION_TRIMATRIX
};
