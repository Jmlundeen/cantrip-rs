use cantrip_api::*;
use cantrip_mechanics::history::opponents_who_lost_life_this_turn;
use cantrip_mechanics::{damage_target, targets, triggers};
use cantrip_oracle as oracle;

/// "Other creatures you control enter with an additional +1/+1 counter on them for each opponent who lost life this turn"
/// (CR 614.1c, 122.6): a replacement of a creature entering under Gev's controller, counted when it enters from the history
/// (CR 119.3: an opponent who gained and lost life lost life). "Other" needs no test: Gev's own replacement works only while
/// he is on the battlefield, and he cannot be entering then; so a creature entering at the same time as Gev is not affected
/// (CR 614.12; the ruling). The creature is judged by the characteristics it would have on the battlefield, a token's by its
/// definition.
fn applies(q: &dyn Query, asker: &Asker, proposal: &Proposal) -> bool {
    matches!(proposal, Proposal::ZoneChange(z)
        if z.to.is(ZoneKind::Battlefield)
            && z.controller == asker.controller
            && q.chars(z.object).is(CardType::Creature)
            && opponents_who_lost_life_this_turn(q, asker.controller) > 0)
}

fn apply(q: &dyn Query, asker: &Asker, proposal: &mut Proposal) {
    if let Proposal::ZoneChange(z) = proposal {
        z.counters[CounterKind::PlusOne as usize] += opponents_who_lost_life_this_turn(q, asker.controller) as i16;
    }
}

/// "Whenever you cast a Lizard spell" (CR 603.2): "Lizard" is a word of Gev's text (CR 612), and the spell is a Lizard by its
/// own characteristics as it stands on the stack.
fn you_cast_a_lizard_spell(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::SpellCast { spell, player, .. }
            if *player == q.controller(source) && q.chars(*spell).has_subtype(q.words(source).subtype(Subtype::Lizard)) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// "Gev deals 1 damage to target opponent."
fn ping(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggers::ward_life::<2>("Ward\u{2014}Pay 2 life."),
        triggered(when(EventMask::SPELL_CAST, you_cast_a_lizard_spell), ping, "Whenever you cast a Lizard spell, Gev deals 1 damage to target opponent.")
            .targets(&[targets::OPPONENT]),
    ],
    replacements: &[replacement(ReplacementKind::ZoneChange, applies, apply, "puts a +1/+1 counter on it for each opponent who lost life this turn")],
    ..oracle::GEV_SCALED_SCORCH
};
