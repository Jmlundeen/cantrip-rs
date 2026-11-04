use cantrip_api::*;
use cantrip_mechanics::{history, replacements};
use cantrip_oracle as oracle;

/// "Once during each of your turns, you may cast an artifact creature spell from your graveyard" (CR 601.3, 604.1).
fn cast_artifact_creature(q: &dyn Query, asker: &Asker, card: ObjectId, player: PlayerId) -> Option<Cost> {
    if player != asker.controller || q.state().turn.active != player {
        return None;
    }
    let c = q.chars(card);
    if !c.is(CardType::Artifact) || !c.is(CardType::Creature) {
        return None;
    }
    (!history::permission_used_this_turn(q, asker.source)).then(|| Cost::mana(q.card(c.card).cost))
}

pub static CARD: CardDef = CardDef {
    granted_ways: &[GrantedWay {
        text: "Once during each of your turns, you may cast an artifact creature spell from your graveyard.",
        cost: cast_artifact_creature,
        permission: true,
        spell_replacement: Some(0),
        ..GrantedWay::CAST
    }],
    granted_replacements: &[replacements::SPELL_ENTERS_WITH_FINALITY
        .text("If you cast a spell this way, that artifact creature enters with a finality counter on it.")],
    ..oracle::OVERFLOW_SUBMATRIX
};
