use cantrip_api::*;
use cantrip_mechanics::next_creature_counter::{give_the_spell_a_counter, spell_enters};
use cantrip_mechanics::{spent, sunburst};
use cantrip_mechanics::triggers::CREATURE_SPELL_YOU_CAST;
use cantrip_oracle as oracle;

/// X, "the number of colors of mana spent to cast it", of the creature spell that is entering.
fn add_counters_per_color(q: &dyn Query, _: &Asker, affected: ObjectId, proposal: &mut Proposal) {
    if let Proposal::ZoneChange(z) = proposal {
        z.counters[CounterKind::PlusOne as usize] += sunburst::colors_spent(q, affected) as i32;
    }
}

pub static CARD: CardDef = CardDef {
    enters: Some(spent::enters_with_a_plus_one_counter_for_each_color_spent),
    abilities: &[triggered(
        CREATURE_SPELL_YOU_CAST,
        give_the_spell_a_counter,
        "Whenever you cast a creature spell, that creature enters with X additional +1/+1 counters on it, where X is the number of colors of mana spent to cast it.",
    )],
    granted_replacements: &[granted_replacement(
        ReplacementKind::ZoneChange,
        spell_enters,
        add_counters_per_color,
        "enters with X additional +1/+1 counters, where X is the number of colors of mana spent to cast it",
    )
    .text("that creature enters with X additional +1/+1 counters on it, where X is the number of colors of mana spent to cast it.")],
    ..oracle::WILDGROWTH_ARCHAIC
};
