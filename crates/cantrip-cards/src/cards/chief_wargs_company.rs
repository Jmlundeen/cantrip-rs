use cantrip_api::*;
use cantrip_mechanics::{tokens, triggers};
use cantrip_oracle as oracle;

/// "This creature can't attack unless you control two or more other Wolves" (CR 508.1c): asked of this creature as attackers are
/// declared; the other Wolves are the permanents its controller controls that are Wolves now, this creature not among them. Once
/// declared it stays attacking whatever happens to them (CR 506.4).
fn attacks_only_with_two_other_wolves(q: &dyn Query, asker: &Asker, attacker: ObjectId, _: Target) -> bool {
    let source = asker.source;
    if source != attacker {
        return true;
    }
    let me = asker.controller;
    let wolf = q.words(source).subtype(Subtype::Wolf);
    q.state().battlefield().iter().filter(|&&id| id != source && q.controller(id) == me && q.chars(id).has_subtype(wolf)).count() >= 2
}

/// "At the beginning of your upkeep, create a 2/2 green Wolf creature token."
fn make_a_wolf(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::WOLF, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(triggers::YOUR_UPKEEP, make_a_wolf, "At the beginning of your upkeep, create a 2/2 green Wolf creature token.")],
    attack_rules: &[attacks_only_with_two_other_wolves],
    ..oracle::CHIEF_WARGS_COMPANY
};

pub static EXTRA: &[&CardDef] = &[&tokens::WOLF];
