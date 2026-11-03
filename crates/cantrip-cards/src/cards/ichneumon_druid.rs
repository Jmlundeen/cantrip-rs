use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// The current cast is already in the history (CR 601.2i). Count earlier instants by this caster as they were cast,
/// even if the Druid wasn't on the battlefield then or those spells have since changed zones.
fn opponent_casts_later_instant(q: &dyn Query, asker: &Asker, event: &Event) -> Option<Fired> {
    let Event::SpellCast { spell, player, .. } = event else { return None };
    if q.state().opponents(asker.controller).any(|p| p == *player)
        && q.chars(*spell).is(CardType::Instant)
        && q.history().casts_by(*player).any(|cast| cast.spell != *spell && cast.is(CardType::Instant))
    {
        Fired::about(Target::Player(*player))
    } else {
        None
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::SPELL_CAST, opponent_casts_later_instant),
        |g, item| effects::damage_that_player(g, item, 4),
        "Whenever an opponent casts an instant spell other than the first instant spell that player casts each turn, this creature deals 4 damage to that player.",
    )],
    ..oracle::ICHNEUMON_DRUID
};
