use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Each Dragon you control enters with an additional +1/+1 counter on it" (CR 614.1c, 614.12, 122.6): a Dragon, card or token,
/// about to enter the battlefield under this permanent's controller. "Dragon" is a word of this card's text (CR 612). The Globe
/// is not a Dragon itself, and a Dragon entering at the same time as the Globe sees no Globe on the battlefield yet.
fn a_dragon_enters_under_you(q: &dyn Query, asker: &Asker, proposal: &Proposal) -> bool {
    let dragon = q.words(asker.source).subtype(Subtype::Dragon);
    matches!(proposal, Proposal::ZoneChange(z)
        if z.to.is(ZoneKind::Battlefield)
            && z.controller == asker.controller
            && q.chars(z.object).has_subtype(dragon))
}

fn add_a_counter(_: &dyn Query, _: &Asker, proposal: &mut Proposal) {
    if let Proposal::ZoneChange(z) = proposal {
        z.counters[CounterKind::PlusOne as usize] += 1;
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(TAP, add_any_color(1), "{T}: Add one mana of any color.")],
    replacements: &[replacement(
        ReplacementKind::ZoneChange,
        a_dragon_enters_under_you,
        add_a_counter,
        "enters with an additional +1/+1 counter",
    )],
    ..oracle::DRAGONSTORM_GLOBE
};
