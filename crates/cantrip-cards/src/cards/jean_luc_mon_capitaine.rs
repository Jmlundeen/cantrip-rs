use cantrip_api::*;
use cantrip_mechanics::replacements;
use cantrip_mechanics::creature_types_among_non_borg;
use cantrip_oracle as oracle;

/// "for each creature type among non-Borg creatures you already control" (CR 205.3m, 614.12): the types of the creatures
/// Jean-Luc's controller controls as the Faerie would enter, Jean-Luc's own Human and Officer among them and no creature
/// entering at the same time, which is not on the battlefield yet.
fn creature_types_already_controlled(q: &dyn Query, asker: &Asker) -> i32 {
    creature_types_among_non_borg(q, asker.source, asker.controller) as i32
}

/// "Federation — Faeries you control enter with an additional +1/+1 counter on them for each creature type among non-Borg
/// creatures you already control" (CR 614.1c, 614.12, 122.6): a Faerie about to enter the battlefield under Jean-Luc's
/// controller, card or token alike, and only a Faerie's entering. A Faerie of another player's that enters does not see this
/// ability at all (CR 614.12), and one entering at the same time as another Faerie counts no type of it.
fn a_faerie_enters_under_you(q: &dyn Query, asker: &Asker, proposal: &Proposal) -> bool {
    let faerie = q.words(asker.source).subtype(Subtype::Faerie);
    replacements::a_subtype_you_control_enters_with_a_counter_for_each(q, asker, proposal, faerie, creature_types_already_controlled)
}

fn add_a_counter_for_each_creature_type(q: &dyn Query, asker: &Asker, proposal: &mut Proposal) {
    replacements::add_a_counter_for_each(q, asker, proposal, creature_types_already_controlled)
}

pub static CARD: CardDef = CardDef {
    replacements: &[replacement(
        ReplacementKind::ZoneChange,
        a_faerie_enters_under_you,
        add_a_counter_for_each_creature_type,
        "enter with an additional +1/+1 counter on them for each creature type among non-Borg creatures you already control",
    )],
    ..oracle::JEAN_LUC_MON_CAPITAINE
};
