use cantrip_api::*;
use cantrip_mechanics::history;
use cantrip_oracle as oracle;

/// "An artifact spell" (CR 601.3): the card on top of the library as the spell it would be.
fn artifact_spell(q: &dyn Query, _: &Asker, card: ObjectId) -> bool {
    q.chars(card).is(CardType::Artifact)
}

/// "Once during each of your turns" (CR 601.3): it is the controller's turn, and no cast made by this permanent's rule has
/// happened this turn, asked of the history (`Cast::granted_by`) - no marker of the card's own.
fn once_during_each_of_your_turns(q: &dyn Query, asker: &Asker) -> bool {
    q.state().turn.active == asker.controller && !history::permission_used_this_turn(q, asker.source)
}

/// "Spacecraft you control enter with two additional charge counters on them" (CR 614.1c, 614.12, 122.6): a Spacecraft
/// entering under this ability's controller, whether or not Zefram is still there as it does.
fn spacecraft_you_control_enters(q: &dyn Query, asker: &Asker, proposal: &Proposal) -> bool {
    matches!(proposal, Proposal::ZoneChange(z)
        if z.to.is(ZoneKind::Battlefield)
            && z.controller == asker.controller
            && q.chars(z.object).has_subtype(Subtype::Spacecraft))
}

fn two_charge_counters(_: &dyn Query, _: &Asker, proposal: &mut Proposal) {
    if let Proposal::ZoneChange(z) = proposal {
        z.counters[CounterKind::Charge as usize] += 2;
    }
}

/// The rule "you may cast an artifact spell from the top of your library", while it has not been used this turn.
static TOP_ARTIFACT: &[PlayRule] = &[PlayRule::LibraryTopSpells(artifact_spell, None)];

pub static CARD: CardDef = CardDef {
    // "You may look at the top card of your library any time", and "once during each of your turns, you may cast an artifact
    // spell from the top of your library" (CR 401.2, 601.3).
    play_rules: &[PlayRule::LookAtLibraryTop, PlayRule::While(once_during_each_of_your_turns, TOP_ARTIFACT)],
    replacements: &[replacement(
        ReplacementKind::ZoneChange,
        spacecraft_you_control_enters,
        two_charge_counters,
        "enters with two additional charge counters",
    )],
    ..oracle::ZEFRAM_COCHRANE_WARP_PIONEER
};
