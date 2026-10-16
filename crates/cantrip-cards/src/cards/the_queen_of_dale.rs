use cantrip_api::*;
use cantrip_mechanics::{recruit, tokens};
use cantrip_oracle as oracle;

/// "Whenever an opponent casts their first noncreature spell each turn" (CR 603.2): a spell cast by an opponent of this
/// ability's controller that is not a creature (a copy is not cast), when that player has cast no other noncreature spell
/// this turn. The earlier casts are asked of the history, judged by the types they had when cast; the cast being matched is
/// left out whether or not the history already holds it. A noncreature spell cast before the Queen arrived still counts: the
/// history is of the turn, not of the Queen.
fn opponents_first_noncreature_spell(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    let Event::SpellCast { spell, player, .. } = event else { return None };
    if *player == me.controller || q.chars(*spell).is(CardType::Creature) {
        return None;
    }
    let earlier = q.history().casts_by(*player).any(|c| c.spell != *spell && !c.is(CardType::Creature));
    if earlier {
        None
    } else {
        Fired::PLAIN
    }
}

/// "You recruit" (CR 701.70a).
fn recruit_effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    recruit::recruit(g, item.controller, item.object)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::SPELL_CAST, opponents_first_noncreature_spell),
        recruit_effect,
        "Whenever an opponent casts their first noncreature spell each turn, you recruit.",
    )],
    ..oracle::THE_QUEEN_OF_DALE
};

pub static EXTRA: &[&CardDef] = &[&tokens::HUMAN_SOLDIER];
