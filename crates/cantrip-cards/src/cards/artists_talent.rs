use cantrip_api::*;
use cantrip_mechanics::{class, effects, triggers};
use cantrip_oracle as oracle;

/// Level 2: "Noncreature spells you cast cost {1} less to cast" (CR 601.2f, 716.2): asked of the spell's own characteristics as
/// it is cast, only its controller's, and only once the Class has reached level 2; a generic reduction takes only the generic
/// part of the cost (CR 118.7c).
fn noncreature_spells_cost_one_less(q: &dyn Query, source: ObjectId, player: PlayerId, costed: Costed, _: &[Target]) -> CostDelta {
    let Costed::Spell(spell) = costed else { return CostDelta::NONE };
    if player != q.controller(source) || class::level(q, source) < 2 {
        return CostDelta::NONE;
    }
    if q.chars(spell).is(CardType::Creature) { CostDelta::NONE } else { CostDelta::less(1) }
}

/// Level 3: "If a source you control would deal noncombat damage to an opponent or a permanent an opponent controls, it deals
/// that much damage plus 2 instead" (CR 614.1, 120.3): the damage of any source its controller controls (a spell or an ability
/// on the stack, a permanent), once the Class has reached level 3. It adds to damage that is still going to be dealt: damage
/// already prevented to nothing is no damage (CR 615.6), so there is none to add to.
fn damage_to_an_opponent_or_their_permanent(q: &dyn Query, asker: &Asker, p: &Proposal) -> bool {
    let Proposal::Damage(d) = p else { return false };
    d.amount > 0
        && !d.combat
        && class::level(q, asker.source) >= 3
        && q.controller(d.source) == asker.controller
        && match d.target {
            Target::Player(player) => q.state().opponents(asker.controller).any(|opponent| opponent == player),
            recipient => recipient.object().is_some_and(|id| q.controller(id) != asker.controller),
        }
}

fn plus_two(_: &dyn Query, _: &Asker, p: &mut Proposal) {
    if let Proposal::Damage(d) = p {
        d.amount = d.amount.saturating_add(2);
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::NONCREATURE_SPELL_YOU_CAST, |g, item| effects::may_discard_to_draw(g, item, 1), "Whenever you cast a noncreature spell, you may discard a card. If you do, draw a card."),
        class::level_two("{2}{R}", "{2}{R}: Level 2"),
        class::level_three("{2}{R}", "{2}{R}: Level 3"),
    ],
    cost_changes: &[noncreature_spells_cost_one_less],
    replacements: &[replacement(ReplacementKind::Damage, damage_to_an_opponent_or_their_permanent, plus_two, "deals that much damage plus 2 instead")],
    ..oracle::ARTISTS_TALENT
};
