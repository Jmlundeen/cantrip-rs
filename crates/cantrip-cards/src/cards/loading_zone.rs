use cantrip_api::*;
use cantrip_mechanics::{replacements, warp};
use cantrip_oracle as oracle;

/// "If one or more counters would be put on a creature, Spacecraft, or Planet you control" (CR 122.6, 614.1a): on one of this
/// source's controller's, whoever puts the counters, including those it enters with (the ruling, CR 614.12): judged as it would
/// exist on the battlefield. The subtypes are read through the words of the asking text (CR 612.2).
fn counters_on_your_creature_spacecraft_or_planet(q: &dyn Query, asker: &Asker, proposal: &Proposal) -> bool {
    let words = q.words(asker.source);
    matches!(proposal, Proposal::Counters(c)
        if c.controller == asker.controller
            && (c.entering || q.is_on_battlefield(c.object))
            && {
                let chars = q.chars(c.object);
                chars.is(CardType::Creature) || chars.has_subtype(words.subtype(Subtype::Spacecraft)) || chars.has_subtype(words.subtype(Subtype::Planet))
            })
}

/// "Twice that many of each of those kinds of counters are put on it instead" (CR 614.5): each kind doubled, each effect once,
/// so two Loading Zones make four times as many (the ruling).
fn twice_that_many(_: &dyn Query, _: &Asker, proposal: &mut Proposal) {
    replacements::double_counters_apply(proposal);
}

pub static CARD: CardDef = CardDef {
    replacements: &[replacement(ReplacementKind::Counters, counters_on_your_creature_spacecraft_or_planet, twice_that_many, "puts twice that many counters of each kind instead")],
    ways: &[warp::way(mana("{G}"), "Warp {G}")],
    end_step_riders: warp::RIDERS,
    delayed: &[warp::EXILE],
    ..oracle::LOADING_ZONE
};
