use cantrip_api::*;
use cantrip_mechanics::replacements::{exile_instead_apply, opponents_creature_would_die};
use cantrip_mechanics::tokens;
use cantrip_oracle as oracle;

/// "When you do, create a 2/2 green Wolf creature token" (CR 603.12): the reflexive trigger of the exile, put on the stack the
/// next time a player would receive priority, once for each creature exiled this way.
fn make_a_wolf(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::WOLF, 1)?;
    Ok(())
}

/// "If a creature an opponent controls would die, exile it instead. When you do, ...": the exile is the replacement
/// (CR 614.1a), and having done it, its controller gets the reflexive trigger (CR 603.12).
fn exiled_this_way(g: &mut dyn Game, asker: &Asker, _: &Proposal, _: &Proposal) {
    let me = g.card_id(&CARD);
    g.reflexive_trigger(asker.source, asker.controller, AbilityRef::delayed(me, 0), Target::None);
}

pub static CARD: CardDef = CardDef {
    replacements: &[replacement(ReplacementKind::ZoneChange, opponents_creature_would_die, exile_instead_apply, "exiles a creature an opponent controls instead of letting it die, and makes a Wolf")
        .then(exiled_this_way)],
    delayed: &[delayed_trigger(make_a_wolf, "When you do, create a 2/2 green Wolf creature token.")],
    ..oracle::HEAD_OF_THE_HUNT
};

pub static EXTRA: &[&CardDef] = &[&tokens::WOLF];
