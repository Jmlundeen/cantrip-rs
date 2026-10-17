use cantrip_api::*;
use cantrip_mechanics::targets::{self, filters};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "Up to one target card from an opponent's graveyard" (CR 115.1, 404.1): a card in the graveyard of a player other than the
/// ability's controller, still that card.
fn card_in_an_opponents_graveyard(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if {
        let o = q.obj(id);
        o.zone.is(ZoneKind::Graveyard) && o.zone.player != asker.controller && o.became.is_none()
    })
}

/// "Exile up to one target card from an opponent's graveyard. Each opponent loses 2 life." The life is lost whether or not a
/// target was chosen; with a target chosen that has become illegal the whole ability does not resolve (CR 608.2b, the ruling).
fn exile_and_drain(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(card) = targets::object(g, item, 0) {
        g.exile_all(&[card]);
    }
    g.each_opponent_loses_life(item.controller, 2);
    Ok(())
}

/// "Return this card from your graveyard to your hand": nothing if it has left the graveyard by then (CR 400.7).
fn return_to_hand(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.return_to_hand(&[item.object]);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    keywords: Keywords::of(&[Keyword::CantBlock]),
    abilities: &[
        triggered(
            THIS_ENTERS,
            exile_and_drain,
            "When Gollum enters, exile up to one target card from an opponent's graveyard. Each opponent loses 2 life.",
        )
        .targets(&[up_to_one(card_in_an_opponents_graveyard)]),
        ability(
            cost("{2}").and(sacrifice(1, filters::artifact_or_creature)),
            return_to_hand,
            "{2}, Sacrifice an artifact or creature: Return this card from your graveyard to your hand. Activate only as a sorcery.",
        )
        .from(ZoneKind::Graveyard)
        .sorcery_speed(),
    ],
    ..oracle::GOLLUM_THE_ABANDONED
};
