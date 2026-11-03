use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "Whenever an opponent casts an artifact spell" (CR 603.2).
fn when_opponent_casts_artifact_spell(
    q: &dyn Query,
    asker: &Asker,
    event: &Event,
) -> Option<Fired> {
    let Event::SpellCast { spell, player, .. } = event else { return None };
    let id = *spell;
    let subject = Target::Object(id);
    if q
        .state()
        .opponents(asker.controller)
        .collect::<Vec<PlayerId>>()
        .contains(&*player) && q.chars(id).is(CardType::Artifact)
    {
        return Fired::about(subject);
    }
    None
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            when_asked(EventMask::SPELL_CAST, when_opponent_casts_artifact_spell),
            effects::plus_one_counter_on_this,
            "Whenever an opponent casts an artifact spell, put a +1/+1 counter on this creature.",
        ),
    ],
    ..oracle::CITANUL_DRUID
};
