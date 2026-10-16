use cantrip_api::*;
use cantrip_oracle as oracle;

/// "If Blightsteel Colossus would be put into a graveyard from anywhere, reveal Blightsteel Colossus and shuffle it
/// into its owner's library instead." (CR 614.1a, 701.20a, 701.24a.) An ability that says where it functions functions
/// there (CR 113.6b), so this works from a hand, a library, the stack and the battlefield alike, and only on the
/// card's own move.
fn applies(_: &dyn Query, asker: &Asker, proposal: &Proposal) -> bool {
    matches!(proposal, Proposal::ZoneChange(z) if z.to.is(ZoneKind::Graveyard) && z.object == asker.source)
}

fn apply(q: &dyn Query, _: &Asker, proposal: &mut Proposal) {
    if let Proposal::ZoneChange(z) = proposal {
        // A card only ever goes to its owner's graveyard (CR 400.3), so that player's library is its owner's.
        z.to = Zone::library(q.obj(z.object).owner);
        z.reveal = true;
        z.shuffle = true;
    }
}

pub static CARD: CardDef = CardDef {
    replacements: &[replacement(ReplacementKind::ZoneChange, applies, apply, "shuffles it into its owner's library")
        .own_everywhere()],
    ..oracle::BLIGHTSTEEL_COLOSSUS
};
