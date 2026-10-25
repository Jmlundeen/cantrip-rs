use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Lands on the battlefield and land cards in graveyards ...": every land permanent, whoever controls it, and every land
/// card in any graveyard. A land card anywhere else - a hand, a library, exile, the stack - is not named.
fn lands_and_land_cards_in_graveyards(q: &dyn LayerQuery, _: ObjectId, subject: ObjectId) -> bool {
    matches!(q.state().obj(subject).zone.kind(), ZoneKind::Battlefield | ZoneKind::Graveyard) && q.chars(subject).is(CardType::Land)
}

/// "... can't be the targets of spells or abilities your opponents control" (CR 601.2c, 602.2b, 603.3d, 608.2b): any spell or
/// ability an opponent of Tomik's controller controls, whatever its source. No hexproof (the card's ruling), so what ignores
/// hexproof does not lift it.
fn controlled_by_an_opponent(q: &dyn LayerQuery, source: ObjectId, by: &Asker, _: &Chars) -> bool {
    by.controller != q.controller(source)
}

/// "Your opponents can't play land cards from graveyards" (CR 101.2, 305.1): any graveyard, by whatever permission (the card's
/// ruling: this takes precedence over it), as whichever face. Playing a land from a hand, a library or exile is not touched,
/// nor is a land card put onto the battlefield from a graveyard, which is not played.
fn opponents_not_from_graveyards(q: &dyn Query, asker: &Asker, player: PlayerId, land: ObjectId, _: CardId) -> bool {
    player == asker.controller || !q.obj(land).zone.is(ZoneKind::Graveyard)
}

pub static CARD: CardDef = CardDef {
    targeting_rules: &[TargetingRule { kind: TargetingRuleKind::Forbid(controlled_by_an_opponent), applies: lands_and_land_cards_in_graveyards }],
    land_play_rules: &[opponents_not_from_graveyards],
    ..oracle::TOMIK_DISTINGUISHED_ADVOKIST
};
