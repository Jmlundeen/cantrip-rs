use cantrip_api::*;
use cantrip_mechanics::{effects, replacements, tokens};
use cantrip_oracle as oracle;

/// "... and create a 2/2 black Zombie creature token." (CR 608.2, 111.1). A replacement's `then` has no resolving cause, so the
/// token is made with the words of Kalitas's text (CR 612), as `tokens::create` would lose them.
fn create_a_zombie(g: &mut dyn Game, asker: &Asker, _: &Proposal, _: &Proposal) {
    let zombie = g.card_id(&tokens::ZOMBIE);
    let words = g.words(asker.source);
    let _ = g.create_tokens_worded(&[(zombie, asker.controller)], false, None, words);
}

/// "{2}{B}, Sacrifice another Vampire or Zombie: Put two +1/+1 counters on Kalitas." (CR 608.2).
fn put_two_1_1_counters(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    !(asker.source == id)
        && (q.chars(id).has_subtype(q.words(asker.source).subtype(Subtype::Vampire))
            || q.chars(id).has_subtype(q.words(asker.source).subtype(Subtype::Zombie)))
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(
            cost("{2}{B}").and(sacrifice(1, put_two_1_1_counters)),
            effects::two_plus_one_counters_on_this,
            "{2}{B}, Sacrifice another Vampire or Zombie: Put two +1/+1 counters on Kalitas.",
        ),
    ],
    replacements: &[
        replacement(
            ReplacementKind::ZoneChange,
            replacements::opponents_nontoken_creature_would_die,
            replacements::exile_instead_apply,
            "If a nontoken creature an opponent controls would die, instead exile that card and create a 2/2 black Zombie creature token.",
        )
        .then(create_a_zombie),
    ],
    ..oracle::KALITAS_TRAITOR_OF_GHET
};

pub static EXTRA: &[&CardDef] = &[&tokens::ZOMBIE];
