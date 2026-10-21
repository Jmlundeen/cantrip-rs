use cantrip_api::*;
use cantrip_mechanics::gift;
use cantrip_oracle as oracle;

/// You and permanents you control gain hexproof until end of turn. If the gift was promised, permanents you control
/// also gain indestructible until end of turn. The permanents are those you control as it resolves, fixed then
/// (CR 611.2c); the gift itself was given by the rules before this runs (CR 702.174j).
fn protect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    g.add_effect(item.object, item.object, EffectKind::PlayerHexproof, Duration::EndOfTurn);
    for permanent in g.permanents_of(you) {
        g.add_effect(item.object, permanent, EffectKind::AddKeywords(Keywords::of(&[Keyword::Hexproof])), Duration::EndOfTurn);
    }
    if item.gift.is_some() {
        // A sentence ends (CR 608.2c): the state triggers judge what it did before the next (`Game::end_sentence`).
        g.end_sentence();
        for permanent in g.permanents_of(you) {
            g.add_effect(item.object, permanent, EffectKind::AddKeywords(Keywords::of(&[Keyword::Indestructible])), Duration::EndOfTurn);
        }
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], protect),
    gift: Some(gift::CARD),
    ..oracle::DAWNS_TRUCE
};
