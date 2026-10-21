use cantrip_api::*;
use cantrip_mechanics::{choose, targets};
use cantrip_oracle as oracle;

/// "Each other creature you control enters with an additional vigilance counter on it" (CR 614.1c, 122.6): another permanent
/// about to enter the battlefield under this permanent's controller that is a creature as it would exist there (CR 614.12),
/// card or token alike. Tayam itself is not "other", and is not on the battlefield as it enters, so this static ability does
/// not apply to it; nor to a creature entering at the same time as it.
fn another_creature_enters_under_you(q: &dyn Query, asker: &Asker, proposal: &Proposal) -> bool {
    matches!(proposal, Proposal::ZoneChange(z)
        if z.to.is(ZoneKind::Battlefield)
            && z.object != asker.source
            && z.controller == asker.controller
            && q.chars(z.object).is(CardType::Creature))
}

fn add_a_vigilance_counter(_: &dyn Query, _: &Asker, proposal: &mut Proposal) {
    if let Proposal::ZoneChange(z) = proposal {
        z.counters[CounterKind::Vigilance as usize] += 1;
    }
}

/// "Mill three cards, then return a permanent card with mana value 3 or less from your graveyard to the battlefield." The
/// card is chosen as the ability resolves, after the mill, and is not targeted: it may be one just milled (the rulings), and
/// an {X} in its cost is 0 there (CR 202.3e). One is returned if there is one (no "up to").
fn mill_and_return(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    g.mill(you, 3);
    let graveyard = g.state().zone(Zone::graveyard(you)).to_vec();
    let eligible: Vec<ObjectId> =
        graveyard.iter().copied().filter(|&c| targets::is_permanent_card(&g.chars(c)) && g.mana_value(c) <= 3).collect();
    if let Some(card) = choose::card_from(g, you, item.object, Purpose::PutOntoBattlefield, graveyard, eligible)? {
        g.put_onto_battlefield(card, you, false)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{3}").and(remove_counters_among(3, targets::filters::creature)),
        mill_and_return,
        "{3}, Remove three counters from among creatures you control: Mill three cards, then return a permanent card with mana value 3 or less from your graveyard to the battlefield.",
    )],
    replacements: &[replacement(
        ReplacementKind::ZoneChange,
        another_creature_enters_under_you,
        add_a_vigilance_counter,
        "puts an additional vigilance counter on it",
    )],
    ..oracle::TAYAM_LUMINOUS_ENIGMA
};
