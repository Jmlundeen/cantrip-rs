use cantrip_api::*;
use cantrip_mechanics::{copy, prepare};
use cantrip_oracle as oracle;

/// "Whenever you cast a prepared spell" (CR 722.3): a spell its controller cast that is a copy of a permanent's prepare
/// spell, identified through its prepare-copy provenance in the event history.
fn prepared_spell_you_cast(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::SpellCast { spell, player, .. } if *player == me.controller && prepare::is_prepare_spell(q, *spell) => {
            Fired::about(Target::Object(*spell))
        }
        _ => None,
    }
}

/// "{W}{U}{B}{R}{G}, {T}: Each creature you control becomes prepared." (CR 722.2): only creatures with a prepare spell
/// become prepared; the rest are unaffected.
fn prepare_all(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    for creature in g.permanents_of(item.controller) {
        if g.chars(creature).is(CardType::Creature) {
            g.become_prepared(creature);
        }
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(when_asked(EventMask::SPELL_CAST, prepared_spell_you_cast), copy::copy_the_cast_spell, "Whenever you cast a prepared spell, copy it. You may choose new targets for the copy."),
        ability(cost("{W}{U}{B}{R}{G}").and(TAP), prepare_all, "{W}{U}{B}{R}{G}, {T}: Each creature you control becomes prepared."),
    ],
    ..oracle::CODIE_RAVENOUS_CODEX
};
