use cantrip_api::*;
use cantrip_mechanics::{choose, web_slinging};
use cantrip_oracle as oracle;

/// The card types Arachne offers, in the order of her `labels`: the card types a spell can have, but creature (CR 205.2a lists 15 types). The others
/// are never a spell's type, so choosing one would tax nothing, as Land does (ruling 2025-09-19).
const TYPES: [CardType; 8] = [
    CardType::Artifact,
    CardType::Battle,
    CardType::Enchantment,
    CardType::Instant,
    CardType::Kindred,
    CardType::Land,
    CardType::Planeswalker,
    CardType::Sorcery,
];

/// "As Arachne enters, look at an opponent's hand, then choose a card type other than creature" (CR 614.12a): the opponent
/// is chosen, their hand looked at, then the type, kept on the permanent as its chosen option.
fn enters(g: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    let opponents: Vec<PlayerId> = g.state().opponents(entering.controller).collect();
    if let Some(opponent) = choose::player(g, entering.controller, entering.object, opponents, "LookAtHand", 0)? {
        let hand = g.state().zone(Zone::hand(opponent)).to_vec();
        g.look_at(entering.controller, &hand);
    }
    choose::option_as_enters(g, entering, &CARD, 0..TYPES.len() as u8)
}

/// "Spells of the chosen type cost {1} more to cast" (CR 601.2f), whoever casts them.
fn tax(q: &dyn Query, source: ObjectId, _player: PlayerId, costed: Costed, _: &[Target]) -> CostDelta {
    let Some(index) = choose::chosen_option(q, source, &CARD) else { return CostDelta::NONE };
    match costed {
        Costed::Spell(id) if q.chars(id).is(TYPES[usize::from(index)]) => CostDelta::more(1),
        _ => CostDelta::NONE,
    }
}

pub static CARD: CardDef = CardDef {
    ways: &[web_slinging::way(mana("{W}"), "Web-slinging {W}")],
    enters: Some(enters),
    labels: &["Artifact", "Battle", "Enchantment", "Instant", "Kindred", "Land", "Planeswalker", "Sorcery"],
    cost_changes: &[tax],
    ..oracle::ARACHNE_PSIONIC_WEAVER
};
